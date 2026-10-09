//! Licence fixtures shared by the Instance evaluation tests.
#![allow(dead_code)]

use ed25519_dalek::{SigningKey, VerifyingKey};
use licence::*;
use time::macros::date;
use uuid::uuid;

pub const THIS: &str = "0b7e6c2a-1d3f-4a5b-9c8d-7e6f5a4b3c2d";
pub const OTHER: &str = "c4d3e2f1-0a9b-4c8d-9e7f-6a5b4c3d2e1f";

pub fn seed_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

pub fn trusted(entries: &[(&str, u8)]) -> Vec<(KeyId, VerifyingKey)> {
    entries
        .iter()
        .map(|(id, seed)| (KeyId::new(id).unwrap(), seed_key(*seed).verifying_key()))
        .collect()
}

pub fn licence_for(instance: &str, key_id: &str) -> Licence {
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
pub fn file_signed_by(payload: &[u8], seed: u8) -> Vec<u8> {
    use ed25519_dalek::Signer;
    encode_envelope(payload, &seed_key(seed).sign(payload).to_bytes())
}
