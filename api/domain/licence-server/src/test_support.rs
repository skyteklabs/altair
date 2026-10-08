//! In-memory test doubles for the licence server ports. Enabled by the
//! `test-support` feature, so no production build carries them.

use crate::{AuditEntry, IdSource, LicenceStore, Signer, SignerError, StoreError};
use ed25519_dalek::{Signer as _, SigningKey};
use licence::{KeyId, Licence};
use std::cell::Cell;
use std::future::Future;
use std::sync::Mutex;
use uuid::{Builder, Uuid};

/// Signs with an Ed25519 key. `fail` makes every call return an error.
pub struct TestSigner {
    pub key: SigningKey,
    pub fail: bool,
}

impl TestSigner {
    pub fn new(key: SigningKey) -> Self {
        Self { key, fail: false }
    }

    pub fn failing(key: SigningKey) -> Self {
        Self { key, fail: true }
    }
}

impl Signer for TestSigner {
    fn key_id(&self) -> KeyId {
        KeyId::new("key-1").unwrap()
    }

    fn sign(&self, payload: &[u8]) -> impl Future<Output = Result<Vec<u8>, SignerError>> + Send {
        let result = if self.fail {
            Err(SignerError("kms unavailable".into()))
        } else {
            Ok(self.key.sign(payload).to_bytes().to_vec())
        };
        async move { result }
    }
}

/// Sequential UUIDv4 values, so issued IDs are unique and parse back.
#[derive(Default)]
pub struct SeqIds(Cell<u128>);

impl SeqIds {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IdSource for SeqIds {
    fn next_id(&self) -> Uuid {
        let next = self.0.get() + 1;
        self.0.set(next);
        Builder::from_random_bytes(next.to_be_bytes()).into_uuid()
    }
}

/// Keeps every stored Licence with its Audit entry. `fail` makes stores error.
#[derive(Default)]
pub struct MemStore {
    pub rows: Mutex<Vec<(Licence, AuditEntry)>>,
    pub fail: bool,
}

impl LicenceStore for MemStore {
    fn record_issue(
        &self,
        licence: Licence,
        audit: AuditEntry,
    ) -> impl Future<Output = Result<(), StoreError>> + Send {
        let result = if self.fail {
            Err(StoreError("disk full".into()))
        } else {
            self.rows.lock().unwrap().push((licence, audit));
            Ok(())
        };
        async move { result }
    }
}
