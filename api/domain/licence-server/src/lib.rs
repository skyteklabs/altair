//! Licence server core (ADR-0013): the `issue` command.
//!
//! Infrastructure sits behind ports: `Signer` (production: Cloud KMS, ADR-0010),
//! `IdSource` and `LicenceStore`. The store saves the Licence and its Audit
//! entry together, so an issued Licence always has its Audit entry.

#[cfg(feature = "test-support")]
pub mod test_support;

use ed25519_dalek::VerifyingKey;
use licence::{
    ActiveUserCap, FORMAT_VERSION, InstanceId, KeyId, Licence, LicenceId, LicenseeId,
    encode_envelope, verify_signature,
};
pub use licence::{InstanceAddress, LicenseeName};
use std::fmt;
use std::future::Future;
use time::{Date, OffsetDateTime, UtcOffset};
use uuid::Uuid;

/// Staff roles, from the Backoffice glossary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffRole {
    Admin,
    Support,
    Growth,
    Sales,
}

/// Identifies a Staff member in the Backoffice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffId(String);

impl StaffId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffMember {
    pub staff_id: StaffId,
    pub role: StaffRole,
}

/// A business rule the command refused. Each variant names one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleViolation {
    OnlySalesOrAdminMayIssue { role: StaffRole },
    ActiveUserCapMustBePositive,
    ExpiryBeforeIssue,
    LicenseeNameEmpty,
    InstanceAddressEmpty,
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleViolation::OnlySalesOrAdminMayIssue { role } => write!(
                f,
                "only Sales or Admin may issue a Licence; this Staff member is {role:?}"
            ),
            RuleViolation::ActiveUserCapMustBePositive => {
                f.write_str("the Active User cap must be at least 1")
            }
            RuleViolation::ExpiryBeforeIssue => {
                f.write_str("the expiry date is before the issue date")
            }
            RuleViolation::LicenseeNameEmpty => f.write_str("the Licensee name is empty"),
            RuleViolation::InstanceAddressEmpty => f.write_str("the Instance address is empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueError {
    Rule(RuleViolation),
    Signer(SignerError),
    Store(StoreError),
    /// The `IdSource` gave a UUID that is not v4.
    IdNotV4(Uuid),
    /// The signature does not verify with the Signer's `verifying_key`.
    SignatureDoesNotVerify,
}

impl fmt::Display for IssueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueError::Rule(rule) => write!(f, "rule violation: {rule}"),
            IssueError::Signer(SignerError(e)) => write!(f, "signer failed: {e}"),
            IssueError::Store(StoreError(e)) => write!(f, "store failed: {e}"),
            IssueError::IdNotV4(id) => write!(f, "id source gave a non-v4 UUID: {id}"),
            IssueError::SignatureDoesNotVerify => {
                f.write_str("signature does not verify with the signer's key")
            }
        }
    }
}

impl std::error::Error for IssueError {}

/// Signs payload bytes with the Licence signing key.
pub trait Signer {
    fn key_id(&self) -> KeyId;
    /// The key Instances trust for `key_id`, taken from the trusted-keys config
    /// they are built with. Never fetch it from the signing backend: its own
    /// public key always verifies its own signature, so `issue` would no longer
    /// catch a backend key that does not match `key_id`.
    fn verifying_key(&self) -> VerifyingKey;
    fn sign(&self, payload: &[u8]) -> impl Future<Output = Result<Vec<u8>, SignerError>> + Send;
}

/// Produces fresh UUIDv4s for Instance and Licence IDs. `issue` refuses any
/// other version, since an Instance cannot read it back.
pub trait IdSource {
    fn next_id(&self) -> Uuid;
}

/// Saves an issued Licence and its Audit entry together, or neither.
pub trait LicenceStore {
    fn record_issue(
        &self,
        licence: Licence,
        audit: AuditEntry,
    ) -> impl Future<Output = Result<(), StoreError>> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    IssueLicence,
}

/// One Staff action on a Licensee or Licence. Never edited or deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub staff_id: StaffId,
    pub role: StaffRole,
    pub action: AuditAction,
    pub at: OffsetDateTime,
    pub licensee_id: LicenseeId,
    pub licensee_name: LicenseeName,
    pub licence_id: LicenceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueRequest {
    pub staff_member: StaffMember,
    /// Time of the action. Its UTC date is the Licence's issue date.
    pub at: OffsetDateTime,
    pub licensee_id: LicenseeId,
    pub licensee_name: String,
    pub instance_address: String,
    pub expires: Date,
    pub active_user_cap: u32,
    pub licensed_features: Vec<String>,
}

/// `bytes` is the Licence file exactly as signed (ADR-0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedLicence {
    pub licence_id: LicenceId,
    pub instance_id: InstanceId,
    pub bytes: Vec<u8>,
}

/// Issues a Licence. Sales and Admin only. Nothing is stored or signed if a
/// rule fails, and nothing is stored if signing fails or the signature does
/// not verify with the Signer's `verifying_key`.
pub async fn issue<S: Signer, I: IdSource, L: LicenceStore>(
    signer: &S,
    ids: &I,
    store: &L,
    req: IssueRequest,
) -> Result<IssuedLicence, IssueError> {
    match req.staff_member.role {
        StaffRole::Sales | StaffRole::Admin => {}
        role => {
            return Err(IssueError::Rule(RuleViolation::OnlySalesOrAdminMayIssue {
                role,
            }));
        }
    }
    let active_user_cap = ActiveUserCap::new(req.active_user_cap)
        .ok_or(IssueError::Rule(RuleViolation::ActiveUserCapMustBePositive))?;
    let licensee_name = LicenseeName::new(req.licensee_name)
        .ok_or(IssueError::Rule(RuleViolation::LicenseeNameEmpty))?;
    let instance_address = InstanceAddress::new(req.instance_address)
        .ok_or(IssueError::Rule(RuleViolation::InstanceAddressEmpty))?;
    let issued = req.at.to_offset(UtcOffset::UTC).date();
    if req.expires < issued {
        return Err(IssueError::Rule(RuleViolation::ExpiryBeforeIssue));
    }

    let id = ids.next_id();
    let instance_id = InstanceId::from_uuid(id).ok_or(IssueError::IdNotV4(id))?;
    let id = ids.next_id();
    let licence_id = LicenceId::from_uuid(id).ok_or(IssueError::IdNotV4(id))?;
    let licence = Licence {
        licence_id,
        licensee_id: req.licensee_id,
        licensee_name: licensee_name.clone(),
        instance_id,
        instance_address,
        issued,
        expires: req.expires,
        active_user_cap,
        licensed_features: req.licensed_features,
        key_id: signer.key_id(),
        format_version: FORMAT_VERSION,
    };

    let payload = licence.payload_bytes();
    let signature = signer.sign(&payload).await.map_err(IssueError::Signer)?;
    if !verify_signature(&payload, &signature, &signer.verifying_key()) {
        return Err(IssueError::SignatureDoesNotVerify);
    }
    let bytes = encode_envelope(&payload, &signature);

    let audit = AuditEntry {
        staff_id: req.staff_member.staff_id,
        role: req.staff_member.role,
        action: AuditAction::IssueLicence,
        at: req.at,
        licensee_id: licence.licensee_id,
        licensee_name,
        licence_id,
    };
    store
        .record_issue(licence, audit)
        .await
        .map_err(IssueError::Store)?;

    Ok(IssuedLicence {
        licence_id,
        instance_id,
        bytes,
    })
}
