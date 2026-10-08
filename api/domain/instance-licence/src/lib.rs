//! Instance-side Licence evaluation (ADR-0013).
//!
//! Checks run in a fixed order, and each `Invalid` reason has one trigger:
//! no file, envelope shape, format version, strict parse, key ID, signature,
//! Instance ID. The term check comes in SKY-6, so `today` is accepted now and
//! not yet used.

use ed25519_dalek::VerifyingKey;
use licence::{
    InstanceId, KeyId, Licence, check_format_version, parse_payload, read_envelope,
    read_format_version, read_key_id, verify_signature,
};
use time::Date;

/// Why an Instance does not hold a valid Licence. Closed; carries no detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceLicenceReason {
    NoLicence,
    Malformed,
    UnsupportedFormatVersion,
    UnknownKeyId,
    BadSignature,
    WrongInstanceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Valid(Licence),
    Invalid(InstanceLicenceReason),
}

/// Evaluates an installed Licence file for this Instance.
///
/// `trusted` is the set of public keys this build accepts, by key ID.
/// `_today` is unused until SKY-6 adds the term and expiry checks.
pub fn evaluate(
    file: Option<&[u8]>,
    this_instance: InstanceId,
    trusted: &[(KeyId, VerifyingKey)],
    _today: Date,
) -> Status {
    use InstanceLicenceReason::*;

    let Some(file) = file else {
        return Status::Invalid(NoLicence);
    };
    let Ok(envelope) = read_envelope(file) else {
        return Status::Invalid(Malformed);
    };
    let Ok(version) = read_format_version(&envelope.payload) else {
        return Status::Invalid(Malformed);
    };
    if check_format_version(version).is_err() {
        return Status::Invalid(UnsupportedFormatVersion);
    }
    let Ok(key_id) = read_key_id(&envelope.payload) else {
        return Status::Invalid(Malformed);
    };
    let Ok(licence) = parse_payload(&envelope.payload) else {
        return Status::Invalid(Malformed);
    };
    let Some((_, key)) = trusted.iter().find(|(id, _)| *id == key_id) else {
        return Status::Invalid(UnknownKeyId);
    };
    if !verify_signature(&envelope.payload, &envelope.signature, key) {
        return Status::Invalid(BadSignature);
    }
    if licence.instance_id != this_instance {
        return Status::Invalid(WrongInstanceId);
    }
    Status::Valid(licence)
}
