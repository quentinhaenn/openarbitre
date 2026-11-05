//! Action descriptors, schema subset, catalogue build, conformance, text parsers
//! (ST-03 specs RG-ST03-1.2, 4.1 to 4.6).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
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

impl LoggedAs {
    /// Public name: `none`, `delete`, `move`.
    pub fn as_str(self) -> &'static str {
        match self {
            LoggedAs::None => "none",
            LoggedAs::Delete => "delete",
            LoggedAs::Move => "move",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    Calendar,
    Relay,
    Ai,
}

impl Network {
    /// Public name: `calendar`, `relay`, `ai`.
    pub fn as_str(self) -> &'static str {
        match self {
            Network::Calendar => "calendar",
            Network::Relay => "relay",
            Network::Ai => "ai",
        }
    }
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
    List(Box<Schema>),
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

/// `word(sep word)*`, words `[a-z0-9]+`; `lead_alpha`: the first character is a letter.
fn dashed(s: &str, sep: char, lead_alpha: bool) -> bool {
    let lead = s
        .chars()
        .next()
        .is_some_and(|c| !lead_alpha || c.is_ascii_lowercase());
    lead && s.split(sep).all(|w| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

fn check_name(name: &str, module: &str) -> Result<(), String> {
    if name.len() > 64 || !name.split('.').all(|s| dashed(s, '-', true)) {
        return Err(format!("ill-formed name {name:?}"));
    }
    if name.split('.').next() != Some(module) {
        return Err(format!("first segment is not the module id {module:?}"));
    }
    if name == "arbitre" {
        return Err("name `arbitre` alone is reserved".into());
    }
    Ok(())
}

/// `input`: restrictions of RG-ST03-4.3; an output nests freely.
fn check_schema(s: &Schema, input: bool) -> Result<(), String> {
    match s {
        Schema::String {
            min_length: Some(a),
            max_length: Some(b),
        } if a > b => Err("minLength > maxLength".into()),
        Schema::Integer {
            minimum: Some(a),
            maximum: Some(b),
        } if a > b => Err("minimum > maximum".into()),
        Schema::Enum(vs) => {
            let distinct: BTreeSet<&String> = vs.iter().collect();
            if vs.is_empty() || distinct.len() != vs.len() {
                return Err("empty or duplicate enum".into());
            }
            if vs.iter().any(|v| !dashed(v, '-', false)) {
                return Err("ill-formed enum value".into());
            }
            Ok(())
        }
        Schema::List(inner) => {
            let scalar = matches!(
                **inner,
                Schema::String { .. } | Schema::Integer { .. } | Schema::Enum(_) | Schema::Date
            );
            if input && !scalar {
                return Err("input list items must be string, integer, enum or date".into());
            }
            check_schema(inner, input)
        }
        Schema::Object(_) if input => Err("nested object in an input".into()),
        Schema::Object(props) => check_props(props, input),
        _ => Ok(()),
    }
}

fn check_props(props: &[Property], input: bool) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for p in props {
        let n = &p.name;
        if !dashed(n, '_', true) || matches!(n.as_str(), "json" | "store" | "help" | "version") {
            return Err(format!("reserved or ill-formed property name {n:?}"));
        }
        if !seen.insert(n) {
            return Err(format!("duplicate property {n:?}"));
        }
        if p.description.is_empty() {
            return Err(format!("property {n:?} has no description"));
        }
        if input && !p.required && matches!(p.schema, Schema::Boolean | Schema::List(_)) {
            return Err(format!("optional boolean or list {n:?}"));
        }
        check_schema(&p.schema, input).map_err(|e| format!("property {n:?}: {e}"))?;
    }
    Ok(())
}

fn check_top(s: &Schema, input: bool) -> Result<(), String> {
    match s {
        Schema::Object(props) => check_props(props, input),
        _ => Err("not an object schema".into()),
    }
}

fn check_descriptor(a: &ActionDescriptor, module: &str) -> Result<(), String> {
    check_name(&a.name, module)?;
    if a.summary.is_empty() || a.summary.contains(['\n', '\r']) {
        return Err("summary must be one non-empty line".into());
    }
    if a.logged_as != LoggedAs::None && !a.mutating {
        return Err("logged_as requires mutating".into());
    }
    check_top(&a.input, true).map_err(|e| format!("input: {e}"))?;
    check_top(&a.output, false).map_err(|e| format!("output: {e}"))
}

impl Catalog {
    /// Builds from the foundation's descriptors (module `arbitre`) and `(module id,
    /// descriptors)` pairs; refuses every case of RG-ST03-4.6 and RG-ST04-3.4.
    /// A path may be both an action and a group (Q-B (a)).
    pub fn build(
        foundation: &[ActionDescriptor],
        modules: &[(&str, Vec<ActionDescriptor>)],
    ) -> Result<Catalog, CatalogError> {
        let mut names: BTreeMap<&str, &str> = BTreeMap::new();
        let mut paths: BTreeMap<Vec<&str>, &str> = BTreeMap::new();
        let mut ids = BTreeSet::new();
        let mut actions = Vec::new();
        let all = core::iter::once(("arbitre", foundation))
            .chain(modules.iter().map(|(m, list)| (*m, list.as_slice())));
        // Foundation first command words, each with one action using it (RG-ST04-3.4, F-8).
        let reserved: BTreeMap<&str, &str> = foundation
            .iter()
            .filter_map(|a| Some((*command_path(&a.name).first()?, a.name.as_str())))
            .collect();
        for (module, _) in modules {
            let why = if *module == "arbitre" {
                "owned by the foundation".into()
            } else if !dashed(module, '-', true) {
                "ill-formed".into()
            } else if let Some(a) = reserved.get(module) {
                format!("reserved by foundation action {a:?}")
            } else {
                continue;
            };
            return Err(CatalogError {
                message: format!("module id {module:?}: {why}"),
            });
        }
        for (module, list) in all {
            if !ids.insert(module) {
                return Err(CatalogError {
                    message: format!("module id {module:?} given twice"),
                });
            }
            for a in list {
                let err = |m: String| CatalogError {
                    message: format!("action {:?} (module {module}): {m}", a.name),
                };
                check_descriptor(a, module).map_err(err)?;
                if let Some(other) = names.insert(&a.name, module) {
                    return Err(err(format!("duplicate name, also in module {other}")));
                }
                if let Some(other) = paths.insert(command_path(&a.name), &a.name) {
                    return Err(err(format!("same command path as {other:?}")));
                }
                actions.push(a.clone());
            }
        }
        actions.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Catalog { actions })
    }

    /// Descriptors, sorted by name.
    pub fn actions(&self) -> &[ActionDescriptor] {
        &self.actions
    }
}

/// Command path: the name's segments, minus a leading `arbitre` (RG-ST03-4.2).
pub fn command_path(name: &str) -> Vec<&str> {
    name.split('.')
        .enumerate()
        .filter(|&(i, s)| i > 0 || s != "arbitre")
        .map(|(_, s)| s)
        .collect()
}

/// MCP tool name: `.` → `_` (RG-ST03-4.2, spike S5).
pub fn mcp_name(name: &str) -> String {
    name.replace('.', "_")
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: core::ops::Range<usize>| {
        let d = &s[r];
        if d.bytes().all(|c| c.is_ascii_digit()) {
            d.parse::<u32>().ok()
        } else {
            None
        }
    };
    let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return false,
    };
    y >= 1 && (1..=days).contains(&d)
}

/// Parses one option text against a scalar schema (string, integer, enum, date);
/// `Err` carries the message shown with the usage (RG-ST03-1.2, 4.4).
pub fn parse_text(schema: &Schema, text: &str) -> Result<Value, String> {
    let v = match schema {
        Schema::Integer { .. } => Value::Int(
            text.parse()
                .map_err(|_| format!("{text:?} is not an integer"))?,
        ),
        _ => Value::Str(text.into()),
    };
    check(schema, &v)?;
    Ok(v)
}

fn check(schema: &Schema, v: &Value) -> Result<(), String> {
    match (schema, v) {
        (
            Schema::String {
                min_length,
                max_length,
            },
            Value::Str(s),
        ) => {
            let n = s.chars().count() as u64;
            if min_length.is_some_and(|m| n < m) || max_length.is_some_and(|m| n > m) {
                return Err(format!("length {n} out of bounds"));
            }
        }
        (Schema::Integer { minimum, maximum }, Value::Int(n)) => {
            if minimum.is_some_and(|m| *n < m) || maximum.is_some_and(|m| *n > m) {
                return Err(format!("{n} out of range"));
            }
        }
        (Schema::Boolean, Value::Bool(_)) => {}
        (Schema::Enum(vs), Value::Str(s)) => {
            if !vs.contains(s) {
                return Err(format!("{s:?} is not one of {vs:?}"));
            }
        }
        (Schema::Date, Value::Str(s)) => {
            if !is_date(s) {
                return Err(format!("{s:?} is not a YYYY-MM-DD date"));
            }
        }
        (Schema::List(inner), Value::List(items)) => {
            items.iter().try_for_each(|i| check(inner, i))?
        }
        (Schema::Object(props), Value::Object(m)) => {
            if let Some(k) = m.keys().find(|k| !props.iter().any(|p| &p.name == *k)) {
                return Err(format!("undeclared key {k:?}"));
            }
            for p in props {
                match m.get(&p.name) {
                    Some(x) => check(&p.schema, x).map_err(|e| format!("{}: {e}", p.name))?,
                    None if p.required => return Err(format!("missing key {:?}", p.name)),
                    None => {}
                }
            }
        }
        _ => return Err("wrong type".into()),
    }
    Ok(())
}

/// Input conformance (RG-ST03-4.4): exactly the declared keys (booleans and lists always
/// present), each of its type and within bounds. `Err` = first failure.
pub fn conforms_input(schema: &Schema, value: &Value) -> Result<(), String> {
    check(schema, value)
}

/// Output conformance (RG-ST03-4.5): required keys present, no undeclared key, types and bounds.
pub fn conforms_output(schema: &Schema, value: &Value) -> Result<(), String> {
    check(schema, value)
}

fn schema_json(s: &Schema) -> BTreeMap<String, Value> {
    let st = |x: &str| Value::Str(x.into());
    let mut m = BTreeMap::new();
    let ty = match s {
        Schema::String {
            min_length,
            max_length,
        } => {
            for (k, v) in [("minLength", min_length), ("maxLength", max_length)] {
                if let Some(n) = v {
                    m.insert(k.into(), Value::Int(i64::try_from(*n).unwrap_or(i64::MAX)));
                }
            }
            "string"
        }
        Schema::Integer { minimum, maximum } => {
            for (k, v) in [("minimum", minimum), ("maximum", maximum)] {
                if let Some(n) = v {
                    m.insert(k.into(), Value::Int(*n));
                }
            }
            "integer"
        }
        Schema::Boolean => "boolean",
        Schema::Enum(vs) => {
            m.insert(
                "enum".into(),
                Value::List(vs.iter().map(|v| st(v)).collect()),
            );
            "string"
        }
        Schema::Date => {
            m.insert("format".into(), st("date"));
            "string"
        }
        Schema::List(inner) => {
            m.insert("items".into(), Value::Object(schema_json(inner)));
            "array"
        }
        Schema::Object(props) => {
            let mut required: Vec<&String> = props
                .iter()
                .filter(|p| p.required)
                .map(|p| &p.name)
                .collect();
            required.sort();
            let properties = props.iter().map(|p| {
                let mut o = schema_json(&p.schema);
                o.insert("description".into(), st(&p.description));
                (p.name.clone(), Value::Object(o))
            });
            m.insert("additionalProperties".into(), Value::Bool(false));
            m.insert("properties".into(), Value::Object(properties.collect()));
            m.insert(
                "required".into(),
                Value::List(required.into_iter().map(|n| st(n)).collect()),
            );
            "object"
        }
    };
    m.insert("type".into(), st(ty));
    m
}

impl ActionDescriptor {
    /// Input schema in the JSON Schema form of RG-ST03-4.3 (handed to MCP as is, ST-07).
    pub fn input_json_schema(&self) -> Value {
        Value::Object(schema_json(&self.input))
    }

    /// Output schema in the JSON Schema form of RG-ST03-4.3.
    pub fn output_json_schema(&self) -> Value {
        Value::Object(schema_json(&self.output))
    }
}
