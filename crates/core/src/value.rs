//! Structured value, canonical JSON, human rendering (ST-03 specs RG-ST03-2.3, 2.4).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A structured value: no null, no float. Objects are sorted by key (UTF-8 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

/// The success result of an action (glossary `Output`, plan P-11).
pub type Output = Value;

/// Canonical JSON of `v` (RG-ST03-2.4): sorted keys, no whitespace, shortest decimal
/// integers, the escaping table of RG-ST03-2.4. Same value ⇒ same bytes.
pub fn encode(v: &Value) -> String {
    let _ = v;
    todo!("T-2: encode — RG-ST03-2.4, INV-ST03-5")
}

/// Human rendering of `v` (RG-ST03-2.3): deterministic, key order, every leaf on a line
/// carrying its key, strings verbatim; an empty object renders as empty text.
pub fn render(v: &Value) -> String {
    let _ = v;
    todo!("T-2: render — RG-ST03-2.3, INV-ST03-2")
}
