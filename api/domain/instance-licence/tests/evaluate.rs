use ed25519_dalek::{SigningKey, VerifyingKey};
use instance_licence::{InstanceLicenceReason::*, LicenceStatus, evaluate};
use licence::*;
use time::macros::date;
use uuid::uuid;

const THIS: &str = "0b7e6c2a-1d3f-4a5b-9c8d-7e6f5a4b3c2d";
const OTHER: &str = "c4d3e2f1-0a9b-4c8d-9e7f-6a5b4c3d2e1f";

fn seed_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn trusted(entries: &[(&str, u8)]) -> Vec<(KeyId, VerifyingKey)> {
    entries
        .iter()
        .map(|(id, seed)| (KeyId::new(id).unwrap(), seed_key(*seed).verifying_key()))
        .collect()
}

fn licence_for(instance: &str, key_id: &str) -> Licence {
    Licence {
        licence_id: LicenceId::from_uuid(uuid!("11111111-2222-4333-8444-555555555555")).unwrap(),
        licensee_id: LicenseeId::from_uuid(uuid!("66666666-7777-4888-8999-aaaaaaaaaaaa")).unwrap(),
        licensee_name: LicenseeName::new("Koperasi Maju").unwrap(),
        instance_id: InstanceId::parse(instance).unwrap(),
        instance_address: InstanceAddress::new("bank.example").unwrap(),
        issued: date!(2026 - 10 - 08),
        expires: date!(2027 - 10 - 08),
        active_user_cap: ActiveUserCap::new(50).unwrap(),
        licensed_features: vec!["investing".into()],
        key_id: KeyId::new(key_id).unwrap(),
        format_version: FORMAT_VERSION,
    }
}

/// Signs `payload` with `seed`, returning a Licence file.
fn file_signed_by(payload: &[u8], seed: u8) -> Vec<u8> {
    use ed25519_dalek::Signer;
    encode_envelope(payload, &seed_key(seed).sign(payload).to_bytes())
}

fn valid_file() -> Vec<u8> {
    file_signed_by(&licence_for(THIS, "key-a").payload_bytes(), 7)
}

fn with_payload_field(licence: &Licence, field: &str, value: serde_json::Value) -> Vec<u8> {
    let mut json: serde_json::Value = serde_json::from_slice(&licence.payload_bytes()).unwrap();
    json[field] = value;
    serde_json::to_vec(&json).unwrap()
}

fn evaluate_file(file: Option<&[u8]>, keys: &[(&str, u8)]) -> LicenceStatus {
    evaluate(
        file,
        InstanceId::parse(THIS).unwrap(),
        &trusted(keys),
        date!(2026 - 10 - 08),
    )
}

#[test]
fn a_licence_issued_for_this_instance_is_valid() {
    let status = evaluate_file(Some(&valid_file()), &[("key-a", 7)]);
    assert_eq!(status, LicenceStatus::Valid(licence_for(THIS, "key-a")));
}

#[test]
fn no_file_is_no_licence() {
    assert_eq!(
        evaluate_file(None, &[("key-a", 7)]),
        LicenceStatus::Invalid(NoLicence)
    );
}

#[test]
fn non_json_file_is_malformed() {
    assert_eq!(
        evaluate_file(Some(b"garbage"), &[("key-a", 7)]),
        LicenceStatus::Invalid(Malformed)
    );
}

#[test]
fn non_json_payload_is_malformed() {
    let file = file_signed_by(b"not a licence", 7);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(Malformed)
    );
}

#[test]
fn zero_cap_payload_is_malformed() {
    let payload = with_payload_field(&licence_for(THIS, "key-a"), "active_user_cap", 0.into());
    let file = file_signed_by(&payload, 7);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(Malformed)
    );
}

#[test]
fn newer_format_version_is_unsupported_even_with_unknown_fields() {
    let payload = with_payload_field(&licence_for(THIS, "key-a"), "format_version", 2.into());
    let mut json: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    json["future_field"] = true.into();
    let file = file_signed_by(&serde_json::to_vec(&json).unwrap(), 7);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(UnsupportedFormatVersion)
    );
}

#[test]
fn unknown_key_id_is_rejected() {
    let file = file_signed_by(&licence_for(THIS, "key-z").payload_bytes(), 7);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(UnknownKeyId)
    );
}

#[test]
fn tampered_payload_is_a_bad_signature() {
    let payload = with_payload_field(
        &licence_for(THIS, "key-a"),
        "licensee_name",
        "Mallory".into(),
    );
    let signed_payload = licence_for(THIS, "key-a").payload_bytes();
    let file = {
        use ed25519_dalek::Signer;
        let sig = seed_key(7).sign(&signed_payload).to_bytes();
        encode_envelope(&payload, &sig)
    };
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(BadSignature)
    );
}

#[test]
fn tampered_signature_is_a_bad_signature() {
    let payload = licence_for(THIS, "key-a").payload_bytes();
    let mut sig = {
        use ed25519_dalek::Signer;
        seed_key(7).sign(&payload).to_bytes()
    };
    sig[0] ^= 1;
    let file = encode_envelope(&payload, &sig);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(BadSignature)
    );
}

#[test]
fn licence_for_another_instance_is_wrong_instance_id() {
    let file = file_signed_by(&licence_for(OTHER, "key-a").payload_bytes(), 7);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7)]),
        LicenceStatus::Invalid(WrongInstanceId)
    );
}

#[test]
fn key_rotation_accepts_either_trusted_key_and_rejects_a_third() {
    let trust = [("key-a", 7), ("key-b", 8)];
    let by_a = file_signed_by(&licence_for(THIS, "key-a").payload_bytes(), 7);
    let by_b = file_signed_by(&licence_for(THIS, "key-b").payload_bytes(), 8);
    let by_c = file_signed_by(&licence_for(THIS, "key-c").payload_bytes(), 9);
    assert!(matches!(
        evaluate_file(Some(&by_a), &trust),
        LicenceStatus::Valid(_)
    ));
    assert!(matches!(
        evaluate_file(Some(&by_b), &trust),
        LicenceStatus::Valid(_)
    ));
    assert_eq!(
        evaluate_file(Some(&by_c), &trust),
        LicenceStatus::Invalid(UnknownKeyId)
    );
}

#[test]
fn a_third_key_claiming_a_trusted_id_is_a_bad_signature() {
    let forged = file_signed_by(&licence_for(THIS, "key-a").payload_bytes(), 9);
    assert_eq!(
        evaluate_file(Some(&forged), &[("key-a", 7), ("key-b", 8)]),
        LicenceStatus::Invalid(BadSignature)
    );
}

#[test]
fn a_removed_key_gives_unknown_key_id() {
    let file = file_signed_by(&licence_for(THIS, "key-old").payload_bytes(), 9);
    assert_eq!(
        evaluate_file(Some(&file), &[("key-a", 7), ("key-b", 8)]),
        LicenceStatus::Invalid(UnknownKeyId)
    );
}
