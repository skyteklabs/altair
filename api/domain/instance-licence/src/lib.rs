//! Instance-side Licence evaluation (ADR-0013).
//!
//! Checks run in a fixed order, and each `Invalid` reason has one trigger:
//! no file, envelope shape, format version, strict parse, key ID, signature,
//! Instance ID. Only then is the Licence term checked against `today`
//! (ADR-0010): expiry is the end of the expiry date, the next
//! [`GRACE_PERIOD_DAYS`] are the Grace period, and after that the Instance is
//! an Expired Instance. Every date comes in as an input; nothing reads a clock.

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

/// Days after the expiry date during which writes still work (ADR-0010).
pub const GRACE_PERIOD_DAYS: i64 = 14;

/// Days before expiry on which Instance administrators are warned (ADR-0010).
pub const EXPIRY_WARNING_DAYS: [i64; 3] = [30, 14, 7];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenceStatus {
    Valid(Licence),
    /// Past expiry but within the Grace period: writes still work.
    GracePeriod(Licence),
    /// Past the Grace period: an Expired Instance, read-only.
    Expired(Licence),
    /// No valid Licence: an Expired Instance, read-only (ADR-0011).
    Invalid(InstanceLicenceReason),
}

/// What the Instance shows its Instance administrators on a given day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    ExpiryWarning { days_left: i64 },
    GracePeriodBanner,
}

impl LicenceStatus {
    /// Whether the Instance may record anything new. No status ever hides or
    /// deletes data; the rest only make the Instance read-only.
    pub fn may_write(&self) -> bool {
        matches!(
            self,
            LicenceStatus::Valid(_) | LicenceStatus::GracePeriod(_)
        )
    }

    /// The notice due on `today`. Worked out from the Licence's own term, so
    /// it stays right even if `today` differs from the day of evaluation.
    pub fn notice_due(&self, today: Date) -> Option<Notice> {
        let (LicenceStatus::Valid(licence)
        | LicenceStatus::GracePeriod(licence)
        | LicenceStatus::Expired(licence)) = self
        else {
            return None;
        };
        match term(licence, today) {
            Term::Running { days_left } => EXPIRY_WARNING_DAYS
                .contains(&days_left)
                .then_some(Notice::ExpiryWarning { days_left }),
            Term::GracePeriod => Some(Notice::GracePeriodBanner),
            Term::Over => None,
        }
    }
}

/// Where `today` falls in a Licence's term. The one place the expiry and
/// Grace period boundaries are drawn.
enum Term {
    /// On or before the expiry date: the Licence runs to the end of that day.
    Running {
        days_left: i64,
    },
    GracePeriod,
    Over,
}

fn term(licence: &Licence, today: Date) -> Term {
    match (today - licence.expires).whole_days() {
        days_past @ ..=0 => Term::Running {
            days_left: -days_past,
        },
        1..=GRACE_PERIOD_DAYS => Term::GracePeriod,
        _ => Term::Over,
    }
}

/// Evaluates an installed Licence file for this Instance.
///
/// `trusted` is the set of public keys this build accepts, by key ID.
/// `today` is the Instance's current date; the caller supplies it.
pub fn evaluate(
    file: Option<&[u8]>,
    this_instance: InstanceId,
    trusted: &[(KeyId, VerifyingKey)],
    today: Date,
) -> LicenceStatus {
    use InstanceLicenceReason::*;

    let Some(file) = file else {
        return LicenceStatus::Invalid(NoLicence);
    };
    let Ok(envelope) = read_envelope(file) else {
        return LicenceStatus::Invalid(Malformed);
    };
    let Ok(version) = read_format_version(&envelope.payload) else {
        return LicenceStatus::Invalid(Malformed);
    };
    if check_format_version(version).is_err() {
        return LicenceStatus::Invalid(UnsupportedFormatVersion);
    }
    let Ok(key_id) = read_key_id(&envelope.payload) else {
        return LicenceStatus::Invalid(Malformed);
    };
    let Ok(licence) = parse_payload(&envelope.payload) else {
        return LicenceStatus::Invalid(Malformed);
    };
    let Some((_, key)) = trusted.iter().find(|(id, _)| *id == key_id) else {
        return LicenceStatus::Invalid(UnknownKeyId);
    };
    if !verify_signature(&envelope.payload, &envelope.signature, key) {
        return LicenceStatus::Invalid(BadSignature);
    }
    if licence.instance_id != this_instance {
        return LicenceStatus::Invalid(WrongInstanceId);
    }
    match term(&licence, today) {
        Term::Running { .. } => LicenceStatus::Valid(licence),
        Term::GracePeriod => LicenceStatus::GracePeriod(licence),
        Term::Over => LicenceStatus::Expired(licence),
    }
}
