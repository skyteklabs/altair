//! Licence server core (ADR-0013): the `issue` command.
//!
//! Infrastructure sits behind ports: `Signer` (production: Cloud KMS, ADR-0010),
//! `IdSource` and `LicenceStore`. The store saves the Licence and its Audit
//! entry together, so an issued Licence always has its Audit entry.

use licence::{FORMAT_VERSION, InstanceId, KeyId, Licence, LicenceId, LicenseeId, encode_envelope};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub staff_id: String,
    pub role: StaffRole,
}

/// A business rule the command refused. Each variant names one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleViolation {
    OnlySalesOrAdminMayIssue { role: StaffRole },
    ActiveUserCapMustBePositive,
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
}

impl fmt::Display for IssueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueError::Rule(rule) => write!(f, "rule violation: {rule:?}"),
            IssueError::Signer(SignerError(e)) => write!(f, "signer failed: {e}"),
            IssueError::Store(StoreError(e)) => write!(f, "store failed: {e}"),
        }
    }
}

impl std::error::Error for IssueError {}

/// Signs payload bytes with the Licence signing key.
pub trait Signer {
    fn key_id(&self) -> KeyId;
    fn sign(&self, payload: &[u8]) -> impl Future<Output = Result<Vec<u8>, SignerError>> + Send;
}

/// Produces fresh UUIDs for Instance and Licence IDs.
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
pub enum Action {
    IssueLicence,
}

/// One Staff action on a Licensee or Licence. Never edited or deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub staff_id: String,
    pub role: StaffRole,
    pub action: Action,
    pub at: OffsetDateTime,
    pub licensee_id: LicenseeId,
    pub licence_id: LicenceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueRequest {
    pub actor: Actor,
    /// Time of the action. Its UTC date is the Licence's issue date.
    pub at: OffsetDateTime,
    pub licensee_id: LicenseeId,
    pub licensee_name: String,
    pub instance_address: String,
    pub expires: Date,
    pub active_user_cap: u32,
    pub features: Vec<String>,
}

/// `bytes` is the Licence file exactly as signed (ADR-0013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedLicence {
    pub licence_id: LicenceId,
    pub instance_id: InstanceId,
    pub bytes: Vec<u8>,
}

/// Issues a Licence. Sales and Admin only. Nothing is stored or signed if a
/// rule fails, and nothing is stored if signing fails.
pub async fn issue<S: Signer, I: IdSource, L: LicenceStore>(
    signer: &S,
    ids: &I,
    store: &L,
    req: IssueRequest,
) -> Result<IssuedLicence, IssueError> {
    match req.actor.role {
        StaffRole::Sales | StaffRole::Admin => {}
        role => {
            return Err(IssueError::Rule(RuleViolation::OnlySalesOrAdminMayIssue {
                role,
            }));
        }
    }
    if req.active_user_cap == 0 {
        return Err(IssueError::Rule(RuleViolation::ActiveUserCapMustBePositive));
    }

    let instance_id = InstanceId::from_uuid(ids.next_id());
    let licence_id = LicenceId::from_uuid(ids.next_id());
    let licence = Licence {
        licence_id,
        licensee_id: req.licensee_id,
        licensee_name: req.licensee_name,
        instance_id,
        instance_address: req.instance_address,
        issued: req.at.to_offset(UtcOffset::UTC).date(),
        expires: req.expires,
        active_user_cap: req.active_user_cap,
        features: req.features,
        key_id: signer.key_id(),
        format_version: FORMAT_VERSION,
    };

    let payload = licence.payload_bytes();
    let signature = signer.sign(&payload).await.map_err(IssueError::Signer)?;
    let bytes = encode_envelope(&payload, &signature);

    let audit = AuditEntry {
        staff_id: req.actor.staff_id,
        role: req.actor.role,
        action: Action::IssueLicence,
        at: req.at,
        licensee_id: licence.licensee_id,
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
