//! Structured value, canonical JSON, human rendering (ST-03 specs RG-ST03-2.3, 2.4).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

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
    let mut out = String::new();
    enc(v, &mut out);
    out
}

fn enc(v: &Value, out: &mut String) {
    match v {
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => write!(out, "{i}").unwrap(),
        Value::Str(s) => enc_str(s, out),
        Value::List(l) => {
            out.push('[');
            for (n, x) in l.iter().enumerate() {
                if n > 0 {
                    out.push(',');
                }
                enc(x, out);
            }
            out.push(']');
        }
        Value::Object(o) => {
            out.push('{');
            for (n, (k, x)) in o.iter().enumerate() {
                if n > 0 {
                    out.push(',');
                }
                enc_str(k, out);
                out.push(':');
                enc(x, out);
            }
            out.push('}');
        }
    }
}

fn enc_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if c < ' ' => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Human rendering of `v` (RG-ST03-2.3): deterministic, key order, one `path: leaf` line
/// per leaf (nested keys dotted, list items repeat the path), strings verbatim; an empty
/// object renders as empty text.
pub fn render(v: &Value) -> String {
    let mut out = String::new();
    walk(v, "", &mut out);
    out
}

fn walk(v: &Value, path: &str, out: &mut String) {
    match v {
        Value::List(l) => l.iter().for_each(|x| walk(x, path, out)),
        Value::Object(o) => {
            for (k, x) in o {
                if path.is_empty() {
                    walk(x, k, out);
                } else {
                    walk(x, &format!("{path}.{k}"), out);
                }
            }
        }
        leaf => {
            if !path.is_empty() {
                out.push_str(path);
                out.push_str(": ");
            }
            match leaf {
                Value::Bool(b) => write!(out, "{b}").unwrap(),
                Value::Int(i) => write!(out, "{i}").unwrap(),
                Value::Str(s) => out.push_str(s),
                _ => {}
            }
            out.push('\n');
        }
    }
}
