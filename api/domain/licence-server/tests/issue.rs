use ed25519_dalek::{SigningKey, VerifyingKey};
use licence::{LicenseeId, parse_payload, read_envelope, verify_signature};
use licence_server::test_support::{MemStore, SeqIds, TestSigner};
use licence_server::*;
use std::future::Future;
use time::macros::{date, datetime};
use uuid::Uuid;

fn signer() -> TestSigner {
    TestSigner {
        key: SigningKey::from_bytes(&[7u8; 32]),
        fail: false,
    }
}

fn request(role: StaffRole) -> IssueRequest {
    IssueRequest {
        staff_member: StaffMember {
            staff_id: StaffId::new("staff-42"),
            role,
        },
        at: datetime!(2026-10-08 23:30:00 -05:00),
        licensee_id: LicenseeId::from_uuid(
            Uuid::parse_str("66666666-7777-4888-8999-aaaaaaaaaaaa").unwrap(),
        )
        .unwrap(),
        licensee_name: "Koperasi Maju".into(),
        instance_address: "bank.example".into(),
        expires: date!(2027 - 10 - 08),
        active_user_cap: 50,
        licensed_features: vec!["investing".into()],
    }
}

fn run(
    signer: &TestSigner,
    store: &MemStore,
    req: IssueRequest,
) -> Result<IssuedLicence, IssueError> {
    pollster::block_on(issue(signer, &SeqIds::new(), store, req))
}

#[test]
fn sales_and_admin_can_issue() {
    for role in [StaffRole::Sales, StaffRole::Admin] {
        let store = MemStore::default();
        assert!(run(&signer(), &store, request(role)).is_ok(), "{role:?}");
    }
}

#[test]
fn support_and_growth_are_refused_with_a_named_rule_and_no_audit_entry() {
    for role in [StaffRole::Support, StaffRole::Growth] {
        let store = MemStore::default();
        assert_eq!(
            run(&signer(), &store, request(role)),
            Err(IssueError::Rule(RuleViolation::OnlySalesOrAdminMayIssue {
                role
            }))
        );
        assert!(store.rows.lock().unwrap().is_empty());
    }
}

#[test]
fn a_zero_cap_is_refused_and_nothing_is_stored() {
    let store = MemStore::default();
    let mut req = request(StaffRole::Sales);
    req.active_user_cap = 0;
    assert_eq!(
        run(&signer(), &store, req),
        Err(IssueError::Rule(RuleViolation::ActiveUserCapMustBePositive))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn a_successful_issue_writes_exactly_one_audit_entry() {
    let store = MemStore::default();
    let issued = run(&signer(), &store, request(StaffRole::Sales)).unwrap();
    let rows = store.rows.lock().unwrap();
    assert_eq!(rows.len(), 1);
    let (licence, audit) = &rows[0];
    assert_eq!(audit.staff_id, StaffId::new("staff-42"));
    assert_eq!(audit.role, StaffRole::Sales);
    assert_eq!(audit.action, AuditAction::IssueLicence);
    assert_eq!(audit.at, datetime!(2026-10-08 23:30:00 -05:00));
    assert_eq!(
        audit.licensee_id,
        LicenseeId::from_uuid(Uuid::parse_str("66666666-7777-4888-8999-aaaaaaaaaaaa").unwrap())
            .unwrap()
    );
    assert_eq!(audit.licensee_name.as_str(), "Koperasi Maju");
    assert_eq!(audit.licence_id, issued.licence_id);
    assert_eq!(licence.licence_id, issued.licence_id);
}

#[test]
fn each_issued_licence_gets_a_new_instance_id() {
    let ids = SeqIds::new();
    let store = MemStore::default();
    let first =
        pollster::block_on(issue(&signer(), &ids, &store, request(StaffRole::Sales))).unwrap();
    let second =
        pollster::block_on(issue(&signer(), &ids, &store, request(StaffRole::Sales))).unwrap();
    assert_ne!(first.instance_id, second.instance_id);
}

#[test]
fn issue_date_is_the_utc_date_of_the_action() {
    // 23:30 on 8 October at UTC-5 is 04:30 on 9 October in UTC.
    let store = MemStore::default();
    run(&signer(), &store, request(StaffRole::Sales)).unwrap();
    let rows = store.rows.lock().unwrap();
    assert_eq!(rows[0].0.issued, date!(2026 - 10 - 09));
}

#[test]
fn the_returned_bytes_verify_and_carry_the_stored_licence() {
    let store = MemStore::default();
    let issued = run(&signer(), &store, request(StaffRole::Sales)).unwrap();
    let env = read_envelope(&issued.bytes).unwrap();
    let key: VerifyingKey = SigningKey::from_bytes(&[7u8; 32]).verifying_key();
    assert!(verify_signature(&env.payload, &env.signature, &key));
    let parsed = parse_payload(&env.payload).unwrap();
    assert_eq!(parsed.instance_id, issued.instance_id);
    assert_eq!(parsed, store.rows.lock().unwrap()[0].0);
}

#[test]
fn a_signing_failure_stores_nothing() {
    let store = MemStore::default();
    let failing = TestSigner {
        fail: true,
        ..signer()
    };
    assert_eq!(
        run(&failing, &store, request(StaffRole::Sales)),
        Err(IssueError::Signer(SignerError("kms unavailable".into())))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn a_store_failure_is_reported() {
    let store = MemStore {
        fail: true,
        ..MemStore::default()
    };
    assert_eq!(
        run(&signer(), &store, request(StaffRole::Sales)),
        Err(IssueError::Store(StoreError("disk full".into())))
    );
}

#[test]
fn an_expiry_before_the_issue_date_is_refused() {
    let store = MemStore::default();
    let mut req = request(StaffRole::Sales);
    req.expires = date!(2026 - 10 - 08); // issue date is 2026-10-09 (UTC)
    assert_eq!(
        run(&signer(), &store, req),
        Err(IssueError::Rule(RuleViolation::ExpiryBeforeIssue))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn an_expiry_on_the_issue_date_is_allowed() {
    let store = MemStore::default();
    let mut req = request(StaffRole::Sales);
    req.expires = date!(2026 - 10 - 09);
    assert!(run(&signer(), &store, req).is_ok());
}

#[test]
fn an_empty_licensee_name_is_refused() {
    let store = MemStore::default();
    let mut req = request(StaffRole::Sales);
    req.licensee_name = "   ".into();
    assert_eq!(
        run(&signer(), &store, req),
        Err(IssueError::Rule(RuleViolation::LicenseeNameEmpty))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn an_empty_instance_address_is_refused() {
    let store = MemStore::default();
    let mut req = request(StaffRole::Sales);
    req.instance_address = String::new();
    assert_eq!(
        run(&signer(), &store, req),
        Err(IssueError::Rule(RuleViolation::InstanceAddressEmpty))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn an_instance_address_that_is_not_a_bare_domain_is_refused_as_invalid() {
    for address in [
        "https://bank.example",
        "bank.example:8443",
        "bank.example/app",
        "admin@bank.example",
        "bank example",
    ] {
        let store = MemStore::default();
        let mut req = request(StaffRole::Sales);
        req.instance_address = address.into();
        assert_eq!(
            run(&signer(), &store, req),
            Err(IssueError::Rule(RuleViolation::InstanceAddressInvalid)),
            "{address}"
        );
        assert!(store.rows.lock().unwrap().is_empty(), "{address}");
    }
}

/// UUIDv7 IDs, as `Uuid::now_v7` would give.
struct V7Ids;

impl IdSource for V7Ids {
    fn next_id(&self) -> Uuid {
        uuid::Builder::from_random_bytes([9u8; 16])
            .with_version(uuid::Version::SortRand)
            .into_uuid()
    }
}

#[test]
fn a_non_v4_id_source_issues_nothing() {
    let store = MemStore::default();
    let id = V7Ids.next_id();
    assert_eq!(
        pollster::block_on(issue(&signer(), &V7Ids, &store, request(StaffRole::Sales))),
        Err(IssueError::IdNotV4(id))
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

/// Signs with one key but reports another, like a KMS key that does not
/// match the configured verifying key.
struct MismatchedSigner {
    signs_with: SigningKey,
    reports: SigningKey,
}

impl Signer for MismatchedSigner {
    fn key_id(&self) -> licence::KeyId {
        licence::KeyId::new("key-1").unwrap()
    }

    fn verifying_key(&self) -> VerifyingKey {
        self.reports.verifying_key()
    }

    fn sign(&self, payload: &[u8]) -> impl Future<Output = Result<Vec<u8>, SignerError>> + Send {
        use ed25519_dalek::Signer as _;
        let signature = self.signs_with.sign(payload).to_bytes().to_vec();
        async move { Ok(signature) }
    }
}

#[test]
fn a_signature_that_does_not_verify_is_not_stored() {
    let store = MemStore::default();
    let signer = MismatchedSigner {
        signs_with: SigningKey::from_bytes(&[8u8; 32]),
        reports: SigningKey::from_bytes(&[7u8; 32]),
    };
    assert_eq!(
        pollster::block_on(issue(
            &signer,
            &SeqIds::new(),
            &store,
            request(StaffRole::Sales)
        )),
        Err(IssueError::SignatureDoesNotVerify)
    );
    assert!(store.rows.lock().unwrap().is_empty());
}

#[test]
fn rule_violations_display_in_glossary_words() {
    let cases = [
        (
            RuleViolation::OnlySalesOrAdminMayIssue {
                role: StaffRole::Support,
            },
            "only Sales or Admin may issue a Licence; this Staff member is Support",
        ),
        (
            RuleViolation::ActiveUserCapMustBePositive,
            "the Active User cap must be at least 1",
        ),
        (
            RuleViolation::ExpiryBeforeIssue,
            "the expiry date is before the issue date",
        ),
        (
            RuleViolation::LicenseeNameEmpty,
            "the Licensee name is empty",
        ),
        (
            RuleViolation::InstanceAddressEmpty,
            "the Instance address is empty",
        ),
        (
            RuleViolation::InstanceAddressInvalid,
            "the Instance address is not a bare domain",
        ),
    ];
    for (rule, text) in cases {
        assert_eq!(rule.to_string(), text);
        assert_eq!(
            IssueError::Rule(rule).to_string(),
            format!("rule violation: {text}")
        );
    }
}
