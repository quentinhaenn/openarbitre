//! Action descriptors, schema subset, catalogue build, conformance, text parsers
//! (ST-03 specs RG-ST03-1.2, 4.1 to 4.6).

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::value::Value;

/// What an action declares about itself (RG-ST03-4.1, frozen after ST-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDescriptor {
    /// Dot-separated path, first segment = module id (RG-ST03-4.2).
    pub name: String,
    /// One non-empty line.
    pub summary: String,
    /// `Schema::Object` (input restrictions of RG-ST03-4.3).
    pub input: Schema,
    /// `Schema::Object`, free nesting.
    pub output: Schema,
    pub mutating: bool,
    /// `≠ None ⇒ mutating`.
    pub logged_as: LoggedAs,
    /// Destinations the action cannot complete without; empty = offline (Q-8 (a)).
    pub requires_network: BTreeSet<Network>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggedAs {
    None,
    Delete,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    Calendar,
    Relay,
    Ai,
}

/// The schema subset of RG-ST03-4.3 (frozen after ST-04). String lengths count characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schema {
    String {
        min_length: Option<u64>,
        max_length: Option<u64>,
    },
    Integer {
        minimum: Option<i64>,
        maximum: Option<i64>,
    },
    Boolean,
    /// Accepted values, in declaration order.
    Enum(Vec<String>),
    /// `YYYY-MM-DD`, years 0001–9999, an existing day; carried as `Value::Str`.
    Date,
    List(alloc::boxed::Box<Schema>),
    /// Properties in declaration order.
    Object(Vec<Property>),
}

/// A named field of an object schema; in an input, one `--` option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    /// Help text, non-empty.
    pub description: String,
    pub required: bool,
    pub schema: Schema,
}

/// Validated descriptors of every wired module (RG-ST03-4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    actions: Vec<ActionDescriptor>,
}

/// Why a catalogue was refused; the message names the action(s) and module(s).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError {
    pub message: String,
}

impl Catalog {
    /// Builds from `(module id, descriptors)` pairs; refuses every case of RG-ST03-4.6.
    /// A path may be both an action and a group (Q-B (a)). Module id `arbitre` = foundation.
    pub fn build(modules: &[(&str, Vec<ActionDescriptor>)]) -> Result<Catalog, CatalogError> {
        let _ = modules;
        todo!("T-3: Catalog::build — RG-ST03-4.2, 4.3, 4.6, INV-ST03-7")
    }

    /// Descriptors, sorted by name.
    pub fn actions(&self) -> &[ActionDescriptor] {
        &self.actions
    }
}

/// Command path: the name's segments, minus a leading `arbitre` (RG-ST03-4.2).
pub fn command_path(name: &str) -> Vec<&str> {
    let _ = name;
    todo!("T-3: command_path — RG-ST03-4.2")
}

/// MCP tool name: `.` → `_` (RG-ST03-4.2, spike S5).
pub fn mcp_name(name: &str) -> String {
    let _ = name;
    todo!("T-3: mcp_name — RG-ST03-4.2")
}

/// Parses one option text against a scalar schema (string, integer, enum, date);
/// `Err` carries the message shown with the usage (RG-ST03-1.2, 4.4).
pub fn parse_text(schema: &Schema, text: &str) -> Result<Value, String> {
    let _ = (schema, text);
    todo!("T-3: parse_text — RG-ST03-1.2, 4.4")
}

/// Input conformance (RG-ST03-4.4): exactly the declared keys (booleans and lists always
/// present), each of its type and within bounds. `Err` = first failure.
pub fn conforms_input(schema: &Schema, value: &Value) -> Result<(), String> {
    let _ = (schema, value);
    todo!("T-3: conforms_input — RG-ST03-4.4, INV-ST03-8")
}

/// Output conformance (RG-ST03-4.5): required keys present, no undeclared key, types and bounds.
pub fn conforms_output(schema: &Schema, value: &Value) -> Result<(), String> {
    let _ = (schema, value);
    todo!("T-3: conforms_output — RG-ST03-4.5, INV-ST03-8")
}

impl ActionDescriptor {
    /// Input schema in the JSON Schema form of RG-ST03-4.3 (handed to MCP as is, ST-07).
    pub fn input_json_schema(&self) -> Value {
        todo!("T-3: input_json_schema — RG-ST03-4.3")
    }

    /// Output schema in the JSON Schema form of RG-ST03-4.3.
    pub fn output_json_schema(&self) -> Value {
        todo!("T-3: output_json_schema — RG-ST03-4.3")
    }
}
