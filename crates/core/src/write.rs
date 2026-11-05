//! Writes and batch validation (ST-01 design, data model, A-1, A-11; T-4).
//!
//! Authors outside the closed list and device ≠ `stamp.a` are not representable (deviation E-1 of the design).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hlc::{DeviceId, L_MAX, Stamp};

/// Record identifier. Contractual order `(module, category, id)`, UTF-8 bytes (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId {
    pub module: String,
    pub category: String,
    pub id: String,
}

/// Field name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field(pub String);

/// Field value serialized by the module; the core only compares its bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldValue(pub Vec<u8>);

/// Closed list of authors (RG-ST01-4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Author {
    CliUser,
    UiUser,
    Assistant,
    System,
}

/// What a write does: set a field or delete the record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Field { name: Field, value: FieldValue },
    Delete,
}

impl Change {
    /// `Field { name, value }` from a name and raw bytes.
    pub fn field(name: &str, value: &[u8]) -> Change {
        Change::Field {
            name: Field(name.into()),
            value: FieldValue(value.to_vec()),
        }
    }
}

/// Which part of its record a write targets. Contractual order: delete before fields (A-1, D-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetKind {
    Delete,
    Field(Field),
}

/// Target `(record, kind)`. Contractual order (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub record: RecordId,
    pub kind: TargetKind,
}

/// Largest stamp known to the author, per target kind of the same record (A-1). Missing entry = `⊥`.
pub type Seen = BTreeMap<TargetKind, Stamp>;

/// Write `(record, change, stamp, seen, author)`. Its derived `Ord` is not contractual (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Write {
    pub record: RecordId,
    pub change: Change,
    pub stamp: Stamp,
    pub seen: Seen,
    pub author: Author,
}

impl Write {
    /// `(record, Field(name))` or `(record, Delete)`.
    pub fn target(&self) -> Target {
        let kind = match &self.change {
            Change::Field { name, .. } => TargetKind::Field(name.clone()),
            Change::Delete => TargetKind::Delete,
        };
        Target {
            record: self.record.clone(),
            kind,
        }
    }

    /// Authoring device, `= stamp.a` by construction (RG-ST01-4.5).
    pub fn device(&self) -> DeviceId {
        self.stamp.a
    }
}

/// Why a write is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A `seen` entry is not `< stamp` (E-2, A-1).
    SeenNotBefore,
    /// Empty module, category, id or field name.
    EmptyIdentifier,
    /// `stamp.l ≥ L_MAX` (A-11).
    StampOutOfBounds,
}

/// Batch rejection: first faulty write, in batch order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchError {
    pub index: usize,
    pub reason: Reason,
}

/// Validates a whole batch (RG-ST01-4.5, INV-ST01-7).
///
/// Post: `Ok` ⇔ every write has non-empty identifiers, `l < L_MAX`
/// and every `seen` entry `< stamp`. Otherwise the first faulty write.
/// Extra `seen` entries are not rejected (A-1).
pub fn validate(batch: &[Write]) -> Result<(), BatchError> {
    for (index, w) in batch.iter().enumerate() {
        let reason = if !identifiers_non_empty(&w.record, &w.change) {
            Reason::EmptyIdentifier
        } else if w.stamp.l >= L_MAX {
            Reason::StampOutOfBounds
        } else if w.seen.values().any(|m| *m >= w.stamp) {
            Reason::SeenNotBefore
        } else {
            continue;
        };
        return Err(BatchError { index, reason });
    }
    Ok(())
}

/// Non-empty module, category, id and field name.
pub(crate) fn identifiers_non_empty(record: &RecordId, change: &Change) -> bool {
    let name_non_empty = match change {
        Change::Field { name, .. } => !name.0.is_empty(),
        Change::Delete => true,
    };
    name_non_empty
        && !record.module.is_empty()
        && !record.category.is_empty()
        && !record.id.is_empty()
}
