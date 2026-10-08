use ed25519_dalek::{Signer as _, SigningKey};
use instance_licence::{LicenceStatus, evaluate};
use licence::{KeyId, LicenseeId};
use licence_server::*;
use std::cell::Cell;
use std::future::Future;
use std::sync::Mutex;
use time::macros::{date, datetime};
use uuid::Uuid;

struct TestSigner(SigningKey);

impl Signer for TestSigner {
    fn key_id(&self) -> KeyId {
        KeyId::new("key-1").unwrap()
    }

    fn sign(&self, payload: &[u8]) -> impl Future<Output = Result<Vec<u8>, SignerError>> + Send {
        let result = Ok(self.0.sign(payload).to_bytes().to_vec());
        async move { result }
    }
}

struct SeqIds(Cell<u128>);

impl IdSource for SeqIds {
    fn next_id(&self) -> Uuid {
        let next = self.0.get() + 1;
        self.0.set(next);
        uuid::Builder::from_random_bytes(u128::to_be_bytes(next)).into_uuid()
    }
}

#[derive(Default)]
struct MemStore(Mutex<usize>);

impl LicenceStore for MemStore {
    fn record_issue(
        &self,
        _licence: licence::Licence,
        _audit: AuditEntry,
    ) -> impl Future<Output = Result<(), StoreError>> + Send {
        *self.0.lock().unwrap() += 1;
        async { Ok(()) }
    }
}

#[test]
fn a_licence_issued_by_the_licence_server_is_valid_on_the_instance() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let issued = pollster::block_on(issue(
        &TestSigner(key.clone()),
        &SeqIds(Cell::new(0)),
        &MemStore::default(),
        IssueRequest {
            actor: StaffMember {
                staff_id: "staff-1".into(),
                role: StaffRole::Sales,
            },
            at: datetime!(2026-10-08 09:00:00 UTC),
            licensee_id: LicenseeId::from_uuid(
                Uuid::parse_str("66666666-7777-4888-8999-aaaaaaaaaaaa").unwrap(),
            ),
            licensee_name: "Koperasi Maju".into(),
            instance_address: "bank.example".into(),
            expires: date!(2027 - 10 - 08),
            active_user_cap: 50,
            licensed_features: vec!["investing".into()],
        },
    ))
    .unwrap();

    let trusted = [(KeyId::new("key-1").unwrap(), key.verifying_key())];
    let status = evaluate(
        Some(&issued.bytes),
        issued.instance_id,
        &trusted,
        date!(2026 - 10 - 08),
    );
    match status {
        LicenceStatus::Valid(licence) => {
            assert_eq!(licence.licence_id, issued.licence_id);
            assert_eq!(licence.instance_id, issued.instance_id);
        }
        other => panic!("expected Valid, got {other:?}"),
    }
}
