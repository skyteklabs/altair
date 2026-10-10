use ed25519_dalek::SigningKey;
use instance_licence::{LicenceStatus, evaluate};
use licence::{InstanceAddress, KeyId, LicenseeId};
use licence_server::test_support::{MemStore, SeqIds, TestSigner};
use licence_server::*;
use time::macros::{date, datetime};
use uuid::Uuid;

#[test]
fn a_licence_issued_by_the_licence_server_is_valid_on_the_instance() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let issued = pollster::block_on(issue(
        &TestSigner::new(key.clone()),
        &SeqIds::new(),
        &MemStore::default(),
        IssueRequest {
            staff_member: StaffMember {
                staff_id: StaffId::new("staff-1"),
                role: StaffRole::Sales,
            },
            at: datetime!(2026-10-08 09:00:00 UTC),
            licensee_id: LicenseeId::from_uuid(
                Uuid::parse_str("66666666-7777-4888-8999-aaaaaaaaaaaa").unwrap(),
            )
            .unwrap(),
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
        &InstanceAddress::new("bank.example").unwrap(),
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
