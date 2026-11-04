//! `Value`, canonical JSON, human rendering, categories, result document (ST-03 T-2;
//! E-ST03-8, RG-ST03-2.1, 2.3, 2.4, 3.1).

use std::collections::BTreeMap;

use arbitre_core::value::Value;
use arbitre_core::{ActionError, Category, document, encode, render};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, RngSeed};

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1000,
        rng_seed: RngSeed::Fixed(0x5703_0001),
        ..ProptestConfig::default()
    }
}

fn s(x: &str) -> Value {
    Value::Str(x.into())
}

fn obj(pairs: &[(&str, Value)]) -> Value {
    Value::Object(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    )
}

/// Strings with controls, quotes, backslashes, `/`, non-Latin and any char.
fn text() -> impl Strategy<Value = String> {
    let c = prop_oneof![
        Just('"'),
        Just('\\'),
        Just('/'),
        proptest::char::range('\0', '\x1f'),
        proptest::char::range('а', 'я'),
        proptest::char::range('日', '旦'),
        any::<char>(),
    ];
    proptest::collection::vec(c, 0..12).prop_map(String::from_iter)
}

fn any_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        text().prop_map(Value::Str),
    ];
    leaf.prop_recursive(4, 64, 6, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..6).prop_map(Value::List),
            proptest::collection::btree_map(text(), inner, 0..6).prop_map(Value::Object),
        ]
    })
}

/// Objects with property-like keys, the shape rendering is checked on.
fn any_object() -> impl Strategy<Value = Value> {
    let key = "[a-z][a-z0-9_]{0,5}";
    let leaf = prop_oneof![
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        text().prop_map(Value::Str),
    ];
    let tree = leaf.prop_recursive(3, 32, 5, move |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..5).prop_map(Value::List),
            proptest::collection::btree_map(key, inner, 0..5).prop_map(Value::Object),
        ]
    });
    proptest::collection::btree_map(key, tree, 0..6).prop_map(Value::Object)
}

fn to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Bool(b) => (*b).into(),
        Value::Int(i) => (*i).into(),
        Value::Str(x) => x.as_str().into(),
        Value::List(l) => l.iter().map(to_json).collect(),
        Value::Object(o) => {
            serde_json::Value::Object(o.iter().map(|(k, v)| (k.clone(), to_json(v))).collect())
        }
    }
}

fn from_json(j: &serde_json::Value) -> Value {
    match j {
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => Value::Int(n.as_i64().expect("i64")),
        serde_json::Value::String(x) => Value::Str(x.clone()),
        serde_json::Value::Array(a) => Value::List(a.iter().map(from_json).collect()),
        serde_json::Value::Object(o) => {
            Value::Object(o.iter().map(|(k, v)| (k.clone(), from_json(v))).collect())
        }
        serde_json::Value::Null => panic!("null in canonical JSON"),
    }
}

fn leaf_text(v: &Value) -> Option<String> {
    match v {
        Value::Bool(b) => Some(b.to_string()),
        Value::Int(i) => Some(i.to_string()),
        Value::Str(x) => Some(x.clone()),
        _ => None,
    }
}

/// `(key, text)` of every leaf; key = the nearest enclosing object key.
fn leaves(v: &Value, key: Option<&str>, out: &mut Vec<(String, String)>) {
    match v {
        Value::List(l) => l.iter().for_each(|x| leaves(x, key, out)),
        Value::Object(o) => o.iter().for_each(|(k, x)| leaves(x, Some(k), out)),
        leaf => {
            if let Some(k) = key {
                out.push((k.to_string(), leaf_text(leaf).unwrap()));
            }
        }
    }
}

/// Some occurrence of `text` starts on a line that also carries `key`.
fn shown(rendered: &str, key: &str, text: &str) -> bool {
    rendered.match_indices(text).any(|(p, _)| {
        let start = rendered[..p].rfind('\n').map_or(0, |i| i + 1);
        let end = rendered[p..].find('\n').map_or(rendered.len(), |i| p + i);
        rendered[start..end].contains(key)
    })
}

// --- E-ST03-8: canonical JSON ---

proptest! {
    #![proptest_config(config())]

    #[test]
    fn encoding_parses_back_to_the_same_value(v in any_value()) {
        let bytes = encode(&v);
        let parsed: serde_json::Value = serde_json::from_str(&bytes).expect("valid JSON");
        prop_assert_eq!(&parsed, &to_json(&v));
        prop_assert_eq!(from_json(&parsed), v);
    }

    #[test]
    fn reencoding_is_byte_identical(v in any_value()) {
        let bytes = encode(&v);
        let back = from_json(&serde_json::from_str(&bytes).unwrap());
        prop_assert_eq!(encode(&back), bytes);
    }

    #[test]
    fn rendering_shows_every_leaf_with_its_key(v in any_object()) {
        let r = render(&v);
        prop_assert_eq!(&r, &render(&v.clone()));
        let mut ls = Vec::new();
        leaves(&v, None, &mut ls);
        for (k, t) in ls {
            prop_assert!(shown(&r, &k, &t), "leaf {k}: {t:?} not shown in {r:?}");
        }
    }
}

#[test]
fn escaping_golden_bytes() {
    let v = s("\"\\\u{8}\t\n\u{c}\r\u{0}\u{1}\u{1b}\u{1f}/é日 ~\u{7f}");
    assert_eq!(
        encode(&v),
        r#""\"\\\b\t\n\f\r\u0000\u0001\u001b\u001f/é日 ~"#.to_string() + "\u{7f}\""
    );
}

#[test]
fn scalars_golden_bytes() {
    assert_eq!(encode(&Value::Bool(true)), "true");
    assert_eq!(encode(&Value::Bool(false)), "false");
    assert_eq!(encode(&Value::Int(0)), "0");
    assert_eq!(encode(&Value::Int(-0)), "0");
    assert_eq!(encode(&Value::Int(42)), "42");
    assert_eq!(encode(&Value::Int(i64::MIN)), "-9223372036854775808");
    assert_eq!(encode(&Value::Int(i64::MAX)), "9223372036854775807");
    assert_eq!(encode(&s("")), "\"\"");
    assert_eq!(encode(&Value::List(vec![])), "[]");
    assert_eq!(encode(&Value::Object(BTreeMap::new())), "{}");
}

#[test]
fn keys_sorted_by_utf8_bytes_no_whitespace() {
    let v = obj(&[
        ("é", Value::Int(1)),
        (
            "b",
            Value::List(vec![Value::Int(1), Value::Bool(false), s("x")]),
        ),
        ("a", obj(&[("z", Value::Int(2)), ("Z", Value::Int(3))])),
        ("B", Value::Bool(true)),
    ]);
    assert_eq!(
        encode(&v),
        r#"{"B":true,"a":{"Z":3,"z":2},"b":[1,false,"x"],"é":1}"#
    );
}

// --- RG-ST03-2.3: rendering ---

#[test]
fn empty_object_renders_as_empty_text() {
    assert_eq!(render(&Value::Object(BTreeMap::new())), "");
}

#[test]
fn rendering_keeps_strings_verbatim() {
    let x = "line one\nline \"two\" \\ é日本\u{1}";
    let r = render(&obj(&[
        ("text", s(x)),
        ("n", Value::Int(-7)),
        ("f", Value::Bool(false)),
    ]));
    assert!(shown(&r, "text", x), "{r:?}");
    assert!(shown(&r, "n", "-7"), "{r:?}");
    assert!(shown(&r, "f", "false"), "{r:?}");
}

// --- RG-ST03-3.1: categories ---

const ALL: [Category; 5] = [
    Category::Success,
    Category::Refusal,
    Category::Usage,
    Category::ToolError,
    Category::ModuleNotEnabled,
];

#[test]
fn exit_code_table() {
    let codes: Vec<u8> = ALL.iter().map(|c| c.exit_code()).collect();
    assert_eq!(codes, vec![0, 1, 2, 3, 4]);
}

#[test]
fn json_names() {
    let names: Vec<Option<&str>> = ALL.iter().map(|c| c.json_name()).collect();
    assert_eq!(
        names,
        vec![
            None,
            Some("refusal"),
            Some("usage"),
            Some("tool_error"),
            Some("module_not_enabled")
        ]
    );
}

// --- RG-ST03-2.1: result document ---

#[test]
fn success_document_golden() {
    let d = document(&Ok(obj(&[])));
    assert_eq!(encode(&d), r#"{"data":{},"format":1,"ok":true}"#);
    let d = document(&Ok(obj(&[("text", s("hi"))])));
    assert_eq!(encode(&d), r#"{"data":{"text":"hi"},"format":1,"ok":true}"#);
}

#[test]
fn failure_document_golden_per_category() {
    let cases = [
        (ActionError::refusal("not_found", "no item"), "refusal", 1),
        (ActionError::usage("unknown_argument", "bad"), "usage", 2),
        (
            ActionError::tool_error("storage_busy", "busy"),
            "tool_error",
            3,
        ),
        (
            ActionError::module_not_enabled("module_not_enabled", "off"),
            "module_not_enabled",
            4,
        ),
    ];
    for (e, c, n) in cases {
        let want = format!(
            r#"{{"error":{{"category":"{c}","code":{n},"id":"{}","message":"{}"}},"format":1,"ok":false}}"#,
            e.id(),
            e.message()
        );
        assert_eq!(encode(&document(&Err(e))), want);
    }
}

#[test]
fn failure_document_carries_data_when_given() {
    let e = ActionError::refusal("invalid_transition", "say \"no\"")
        .with_data(obj(&[("item", s("x"))]));
    assert_eq!(
        encode(&document(&Err(e))),
        r#"{"data":{"item":"x"},"error":{"category":"refusal","code":1,"id":"invalid_transition","message":"say \"no\""},"format":1,"ok":false}"#
    );
}
