use ed25519_dalek::{Signer, SigningKey};
use licence::*;
use time::macros::date;
use uuid::uuid;

const INSTANCE: &str = "0b7e6c2a-1d3f-4a5b-9c8d-7e6f5a4b3c2d";

fn key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn sample() -> Licence {
    Licence {
        licence_id: LicenceId::from_uuid(uuid!("11111111-2222-4333-8444-555555555555")),
        licensee_id: LicenseeId::from_uuid(uuid!("66666666-7777-4888-8999-aaaaaaaaaaaa")),
        licensee_name: "Koperasi Maju".into(),
        instance_id: InstanceId::parse(INSTANCE).unwrap(),
        instance_address: "bank.example".into(),
        issued: date!(2026 - 10 - 08),
        expires: date!(2027 - 10 - 08),
        active_user_cap: 50,
        licensed_features: vec!["investing".into()],
        key_id: KeyId::new("key-1").unwrap(),
        format_version: FORMAT_VERSION,
    }
}

fn signed_file(licence: &Licence) -> Vec<u8> {
    let payload = licence.payload_bytes();
    let signature = key().sign(&payload).to_bytes();
    encode_envelope(&payload, &signature)
}

#[test]
fn envelope_round_trips_and_verifies() {
    let file = signed_file(&sample());
    let env = read_envelope(&file).unwrap();
    assert!(verify_signature(
        &env.payload,
        &env.signature,
        &key().verifying_key()
    ));
}

#[test]
fn payload_round_trips_to_the_same_licence() {
    let licence = sample();
    let parsed = parse_payload(&licence.payload_bytes()).unwrap();
    assert_eq!(parsed, licence);
}

#[test]
fn a_changed_payload_byte_fails_verification() {
    let file = signed_file(&sample());
    let mut env = read_envelope(&file).unwrap();
    let last = env.payload.len() - 2;
    env.payload[last] ^= 1;
    assert!(!verify_signature(
        &env.payload,
        &env.signature,
        &key().verifying_key()
    ));
}

#[test]
fn a_wrong_key_fails_verification() {
    let file = signed_file(&sample());
    let env = read_envelope(&file).unwrap();
    let other = SigningKey::from_bytes(&[9u8; 32]).verifying_key();
    assert!(!verify_signature(&env.payload, &env.signature, &other));
}

#[test]
fn a_short_signature_fails_verification() {
    let file = signed_file(&sample());
    let env = read_envelope(&file).unwrap();
    assert!(!verify_signature(
        &env.payload,
        &env.signature[..63],
        &key().verifying_key()
    ));
}

#[test]
fn instance_id_accepts_only_canonical_lowercase() {
    assert!(InstanceId::parse(INSTANCE).is_ok());
    assert_eq!(
        InstanceId::parse(&INSTANCE.to_uppercase()),
        Err(FormatError::Malformed)
    );
    assert_eq!(
        InstanceId::parse("0b7e6c2a1d3f4a5b9c8d7e6f5a4b3c2d"),
        Err(FormatError::Malformed)
    );
}

#[test]
fn unknown_payload_fields_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_slice(&sample().payload_bytes()).unwrap();
    value["surprise"] = serde_json::json!(true);
    let bytes = serde_json::to_vec(&value).unwrap();
    assert_eq!(parse_payload(&bytes), Err(FormatError::Malformed));
}

#[test]
fn zero_active_user_cap_is_malformed() {
    let mut licence = sample();
    licence.active_user_cap = 0;
    assert_eq!(
        parse_payload(&licence.payload_bytes()),
        Err(FormatError::Malformed)
    );
}

#[test]
fn empty_features_are_valid() {
    let mut licence = sample();
    licence.licensed_features.clear();
    assert!(parse_payload(&licence.payload_bytes()).is_ok());
}

#[test]
fn unknown_feature_names_are_kept_not_rejected() {
    let mut licence = sample();
    licence.licensed_features = vec!["future-feature".into()];
    assert!(parse_payload(&licence.payload_bytes()).is_ok());
}

#[test]
fn version_is_readable_even_when_key_id_is_gone() {
    let mut value: serde_json::Value = serde_json::from_slice(&sample().payload_bytes()).unwrap();
    value["format_version"] = serde_json::json!(2);
    value.as_object_mut().unwrap().remove("key_id");
    let bytes = serde_json::to_vec(&value).unwrap();
    let version = read_format_version(&bytes).unwrap();
    assert_eq!(
        check_format_version(version),
        Err(FormatError::UnsupportedFormatVersion)
    );
}

#[test]
fn key_id_is_read_from_the_payload() {
    let key_id = read_key_id(&sample().payload_bytes()).unwrap();
    assert_eq!(key_id, KeyId::new("key-1").unwrap());
}

#[test]
fn non_json_envelope_is_malformed() {
    assert_eq!(read_envelope(b"not json"), Err(FormatError::Malformed));
}
