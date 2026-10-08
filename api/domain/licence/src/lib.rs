//! Licence document format (ADR-0013).
//!
//! A Licence file is `{"payload": ..., "signature": ...}`, both base64url
//! without padding. `payload` is the exact JSON bytes that were signed; they
//! are verified as-is and never re-encoded.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;
use time::Date;

/// The only format version this crate reads or writes.
pub const FORMAT_VERSION: u32 = 1;

/// Why a file cannot be read as a Licence. Carries no detail: callers map it
/// to one reason each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    Malformed,
    UnsupportedFormatVersion,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Malformed => f.write_str("malformed Licence"),
            FormatError::UnsupportedFormatVersion => f.write_str("unsupported format version"),
        }
    }
}

impl std::error::Error for FormatError {}

macro_rules! uuid_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Parses the canonical form only: lowercase, hyphenated.
            pub fn parse(s: &str) -> Result<Self, FormatError> {
                let uuid = uuid::Uuid::parse_str(s).map_err(|_| FormatError::Malformed)?;
                let parsed = Self(uuid);
                if uuid.get_version_num() == 4 && parsed.to_string() == s {
                    Ok(parsed)
                } else {
                    Err(FormatError::Malformed)
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.hyphenated().fmt(f)
            }
        }

        impl TryFrom<String> for $name {
            type Error = FormatError;
            fn try_from(s: String) -> Result<Self, FormatError> {
                Self::parse(&s)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.to_string()
            }
        }
    };
}

uuid_id!(
    /// Identifies one Instance. Every copy of a Licence for it shares this ID.
    InstanceId
);
uuid_id!(
    /// Identifies one issued Licence.
    LicenceId
);
uuid_id!(
    /// Identifies a Licensee: the organisation that holds the Licence.
    LicenseeId
);

/// Names the signing key. Non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyId(String);

impl KeyId {
    pub fn new(s: &str) -> Result<Self, FormatError> {
        if s.is_empty() {
            Err(FormatError::Malformed)
        } else {
            Ok(Self(s.to_owned()))
        }
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for KeyId {
    type Error = FormatError;
    fn try_from(s: String) -> Result<Self, FormatError> {
        Self::new(&s)
    }
}

impl From<KeyId> for String {
    fn from(id: KeyId) -> String {
        id.0
    }
}

/// The most Active Users an Instance may have. At least 1: a cap of 0 would
/// lock every User out at once (ADR-0013). The only constructor is `new`, so
/// the rule lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct ActiveUserCap(NonZeroU32);

impl ActiveUserCap {
    pub fn new(cap: u32) -> Option<Self> {
        NonZeroU32::new(cap).map(Self)
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<u32> for ActiveUserCap {
    type Error = FormatError;
    fn try_from(cap: u32) -> Result<Self, FormatError> {
        Self::new(cap).ok_or(FormatError::Malformed)
    }
}

impl From<ActiveUserCap> for u32 {
    fn from(cap: ActiveUserCap) -> u32 {
        cap.get()
    }
}

/// The signed content of a Licence. Unknown fields are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Licence {
    pub licence_id: LicenceId,
    pub licensee_id: LicenseeId,
    pub licensee_name: String,
    pub instance_id: InstanceId,
    /// The Instance address the Licensee configures (ADR-0011).
    pub instance_address: String,
    #[serde(with = "calendar_date")]
    pub issued: Date,
    #[serde(with = "calendar_date")]
    pub expires: Date,
    pub active_user_cap: ActiveUserCap,
    /// Open set: names an Instance does not know are ignored (ADR-0013).
    pub licensed_features: Vec<String>,
    pub key_id: KeyId,
    pub format_version: u32,
}

impl Licence {
    /// The exact bytes that get signed.
    pub fn payload_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("a Licence always serialises")
    }
}

/// Parses a payload strictly. Does not check the format version; callers
/// check it first (ADR-0013 order). A zero cap fails in `ActiveUserCap`.
pub fn parse_payload(payload: &[u8]) -> Result<Licence, FormatError> {
    serde_json::from_slice(payload).map_err(|_| FormatError::Malformed)
}

#[derive(Deserialize)]
struct VersionOnly {
    format_version: u32,
}

#[derive(Deserialize)]
struct KeyIdOnly {
    key_id: KeyId,
}

/// Reads only `format_version` from unverified payload bytes. Unknown fields
/// are ignored, so a newer format can still be recognised as unsupported.
pub fn read_format_version(payload: &[u8]) -> Result<u32, FormatError> {
    serde_json::from_slice::<VersionOnly>(payload)
        .map(|v| v.format_version)
        .map_err(|_| FormatError::Malformed)
}

/// Reads only `key_id` from unverified payload bytes. Call after the format
/// version check. Grants nothing until the signature verifies.
pub fn read_key_id(payload: &[u8]) -> Result<KeyId, FormatError> {
    serde_json::from_slice::<KeyIdOnly>(payload)
        .map(|k| k.key_id)
        .map_err(|_| FormatError::Malformed)
}

pub fn check_format_version(found: u32) -> Result<(), FormatError> {
    if found == FORMAT_VERSION {
        Ok(())
    } else {
        Err(FormatError::UnsupportedFormatVersion)
    }
}

/// The outer envelope after base64url decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    payload: String,
    signature: String,
}

/// Reads the envelope shape only. No version, key or signature checks.
pub fn read_envelope(file: &[u8]) -> Result<Envelope, FormatError> {
    let wire: Wire = serde_json::from_slice(file).map_err(|_| FormatError::Malformed)?;
    let payload = URL_SAFE_NO_PAD
        .decode(wire.payload)
        .map_err(|_| FormatError::Malformed)?;
    let signature = URL_SAFE_NO_PAD
        .decode(wire.signature)
        .map_err(|_| FormatError::Malformed)?;
    Ok(Envelope { payload, signature })
}

/// Wraps signed payload bytes and their signature into a Licence file.
pub fn encode_envelope(payload: &[u8], signature: &[u8]) -> Vec<u8> {
    let wire = Wire {
        payload: URL_SAFE_NO_PAD.encode(payload),
        signature: URL_SAFE_NO_PAD.encode(signature),
    };
    serde_json::to_vec(&wire).expect("an envelope always serialises")
}

/// True only if `signature` is a valid Ed25519 signature of `payload` by `key`.
pub fn verify_signature(payload: &[u8], signature: &[u8], key: &VerifyingKey) -> bool {
    match Signature::from_slice(signature) {
        Ok(sig) => key.verify(payload, &sig).is_ok(),
        Err(_) => false,
    }
}

/// `YYYY-MM-DD`, written by hand so `time` needs no `formatting` or `parsing`
/// feature (both pull in `std`; see scripts/check-domain-deps.sh).
mod calendar_date {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::{Date, Month};

    pub fn serialize<S: Serializer>(date: &Date, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!(
            "{:04}-{:02}-{:02}",
            date.year(),
            u8::from(date.month()),
            date.day()
        ))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Date, D::Error> {
        let text = String::deserialize(d)?;
        parse(&text).ok_or_else(|| serde::de::Error::custom("expected a YYYY-MM-DD date"))
    }

    fn parse(text: &str) -> Option<Date> {
        let b = text.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let digits = |range: std::ops::Range<usize>| -> Option<u32> {
            let part = &text[range];
            part.bytes()
                .all(|c| c.is_ascii_digit())
                .then(|| part.parse().ok())
                .flatten()
        };
        let year = digits(0..4)? as i32;
        let month = Month::try_from(digits(5..7)? as u8).ok()?;
        let day = digits(8..10)? as u8;
        Date::from_calendar_date(year, month, day).ok()
    }
}
