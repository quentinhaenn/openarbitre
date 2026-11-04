//! Catalogue build, name rule, conformance, text parsers, JSON Schema form (ST-03 T-3;
//! E-ST03-7, E-ST03-9, RG-ST03-1.2, 4.2 to 4.6).

use std::collections::{BTreeMap, BTreeSet};

use arbitre_core::value::Value;
use arbitre_core::{
    ActionDescriptor, Catalog, CatalogError, LoggedAs, Property, Schema, command_path,
    conforms_input, conforms_output, encode, mcp_name, parse_text,
};

fn prop(name: &str, required: bool, schema: Schema) -> Property {
    Property {
        name: name.into(),
        description: format!("The {name}."),
        required,
        schema,
    }
}

fn string() -> Schema {
    Schema::String {
        min_length: None,
        max_length: None,
    }
}

fn int(minimum: Option<i64>, maximum: Option<i64>) -> Schema {
    Schema::Integer { minimum, maximum }
}

fn list(s: Schema) -> Schema {
    Schema::List(Box::new(s))
}

fn enum_of(vs: &[&str]) -> Schema {
    Schema::Enum(vs.iter().map(|v| v.to_string()).collect())
}

fn empty() -> Schema {
    Schema::Object(vec![])
}

fn action(name: &str) -> ActionDescriptor {
    ActionDescriptor {
        name: name.into(),
        summary: "Does one thing.".into(),
        input: empty(),
        output: empty(),
        mutating: false,
        logged_as: LoggedAs::None,
        requires_network: BTreeSet::new(),
    }
}

fn with_input(name: &str, props: Vec<Property>) -> ActionDescriptor {
    ActionDescriptor {
        input: Schema::Object(props),
        ..action(name)
    }
}

fn with_output(name: &str, props: Vec<Property>) -> ActionDescriptor {
    ActionDescriptor {
        output: Schema::Object(props),
        ..action(name)
    }
}

fn build(modules: &[(&str, Vec<ActionDescriptor>)]) -> Result<Catalog, CatalogError> {
    Catalog::build(modules)
}

#[track_caller]
fn refused(modules: &[(&str, Vec<ActionDescriptor>)], needles: &[&str]) {
    let e = build(modules).expect_err("catalogue must be refused");
    for n in needles {
        assert!(e.message.contains(n), "{:?} does not name {n:?}", e.message);
    }
}

/// Input schema of `test.echo`; mirror of `crates/cli/src/wiring/test.rs`.
fn echo_input() -> Schema {
    Schema::Object(vec![
        Property {
            name: "text".into(),
            description: "Text to echo.".into(),
            required: true,
            schema: Schema::String {
                min_length: None,
                max_length: Some(200),
            },
        },
        Property {
            name: "count".into(),
            description: "A count.".into(),
            required: false,
            schema: int(Some(0), Some(100)),
        },
        Property {
            name: "flag".into(),
            description: "A flag.".into(),
            required: true,
            schema: Schema::Boolean,
        },
        Property {
            name: "level".into(),
            description: "A level.".into(),
            required: false,
            schema: enum_of(&["low", "medium", "high"]),
        },
        Property {
            name: "due".into(),
            description: "A due date.".into(),
            required: false,
            schema: Schema::Date,
        },
        Property {
            name: "tag".into(),
            description: "A tag, repeatable.".into(),
            required: true,
            schema: list(string()),
        },
        Property {
            name: "note".into(),
            description: "An optional note.".into(),
            required: false,
            schema: string(),
        },
    ])
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

// --- E-ST03-7: catalogue build ---

#[test]
fn valid_catalogue_builds_sorted() {
    let c = build(&[
        ("test", vec![action("test.write"), action("test.echo")]),
        ("arbitre", vec![action("arbitre.test-foundation")]),
    ])
    .expect("valid");
    let names: Vec<&str> = c.actions().iter().map(|a| a.name.as_str()).collect();
    assert_eq!(
        names,
        ["arbitre.test-foundation", "test.echo", "test.write"]
    );
}

#[test]
fn empty_catalogue_builds() {
    assert!(build(&[]).expect("valid").actions().is_empty());
}

#[test]
fn action_and_group_on_one_path_accepted() {
    // Q-B (a).
    build(&[(
        "test",
        vec![action("test.group"), action("test.group.child")],
    )])
    .expect("valid");
    build(&[("plan", vec![action("plan"), action("plan.add")])]).expect("valid");
}

#[test]
fn duplicate_name_refused() {
    refused(
        &[
            ("test", vec![action("test.echo")]),
            ("test", vec![action("test.echo")]),
        ],
        &["test.echo", "test"],
    );
    refused(
        &[("test", vec![action("test.echo"), action("test.echo")])],
        &["test.echo"],
    );
}

#[test]
fn duplicate_command_path_refused() {
    refused(
        &[
            ("export", vec![action("export")]),
            ("arbitre", vec![action("arbitre.export")]),
        ],
        &["export", "arbitre.export"],
    );
}

#[test]
fn ill_formed_names_refused() {
    let long_ok = format!("test.{}", "a".repeat(59));
    assert_eq!(long_ok.len(), 64);
    build(&[("test", vec![action(&long_ok)])]).expect("64 characters accepted");
    let long = format!("test.{}", "a".repeat(60));
    for name in [
        "Test.echo",
        "test..echo",
        "test.echo-",
        "test.-echo",
        "test.e--cho",
        "test.1echo",
        "test_echo",
        "test.echo_x",
        "test.écho",
        "test.",
        ".test",
        long.as_str(),
    ] {
        refused(&[("test", vec![action(name)])], &[name]);
    }
    refused(&[("test", vec![action("")])], &["test"]);
    refused(&[("arbitre", vec![action("arbitre")])], &["arbitre"]);
}

#[test]
fn first_segment_must_be_the_module_id() {
    refused(&[("test", vec![action("plan.add")])], &["plan.add", "test"]);
    refused(
        &[("test", vec![action("arbitre.export")])],
        &["arbitre.export", "test"],
    );
}

#[test]
fn input_schemas_outside_the_subset_refused() {
    let bad: Vec<(&str, Vec<Property>)> = vec![
        (
            "optional boolean",
            vec![prop("flag", false, Schema::Boolean)],
        ),
        ("optional list", vec![prop("tag", false, list(string()))]),
        (
            "list of list",
            vec![prop("tag", true, list(list(string())))],
        ),
        (
            "list of boolean",
            vec![prop("tag", true, list(Schema::Boolean))],
        ),
        ("list of object", vec![prop("tag", true, list(empty()))]),
        ("nested object", vec![prop("sub", true, empty())]),
        ("empty enum", vec![prop("level", true, enum_of(&[]))]),
        (
            "duplicate enum",
            vec![prop("level", true, enum_of(&["a", "a"]))],
        ),
        (
            "ill-formed enum value",
            vec![prop("level", true, enum_of(&["Big"]))],
        ),
        (
            "minimum > maximum",
            vec![prop("count", true, int(Some(5), Some(4)))],
        ),
        (
            "minLength > maxLength",
            vec![prop(
                "text",
                true,
                Schema::String {
                    min_length: Some(3),
                    max_length: Some(2),
                },
            )],
        ),
        ("reserved json", vec![prop("json", true, string())]),
        ("reserved store", vec![prop("store", true, string())]),
        ("reserved help", vec![prop("help", true, string())]),
        ("reserved version", vec![prop("version", true, string())]),
        ("ill-formed name Text", vec![prop("Text", true, string())]),
        ("ill-formed name a-b", vec![prop("a-b", true, string())]),
        ("ill-formed name a__b", vec![prop("a__b", true, string())]),
        ("ill-formed name _a", vec![prop("_a", true, string())]),
        ("ill-formed name a_", vec![prop("a_", true, string())]),
        (
            "missing description",
            vec![Property {
                description: String::new(),
                ..prop("text", true, string())
            }],
        ),
        (
            "duplicate property",
            vec![prop("text", true, string()), prop("text", false, string())],
        ),
    ];
    for (case, props) in bad {
        let e = build(&[("test", vec![with_input("test.echo", props)])]).expect_err(case);
        assert!(e.message.contains("test.echo"), "{case}: {:?}", e.message);
    }
}

#[test]
fn input_and_output_must_be_objects() {
    for s in [string(), Schema::Boolean, list(string())] {
        let a = ActionDescriptor {
            input: s.clone(),
            ..action("test.echo")
        };
        refused(&[("test", vec![a])], &["test.echo"]);
        let a = ActionDescriptor {
            output: s,
            ..action("test.echo")
        };
        refused(&[("test", vec![a])], &["test.echo"]);
    }
}

#[test]
fn output_schemas_nest_freely() {
    let nested = vec![
        prop(
            "sub",
            true,
            Schema::Object(vec![prop("x", true, Schema::Boolean)]),
        ),
        prop(
            "items",
            false,
            list(Schema::Object(vec![prop(
                "tags",
                true,
                list(list(string())),
            )])),
        ),
    ];
    build(&[("test", vec![with_output("test.read", nested)])]).expect("valid");
    build(&[(
        "test",
        vec![with_input(
            "test.echo",
            match echo_input() {
                Schema::Object(p) => p,
                _ => unreachable!(),
            },
        )],
    )])
    .expect("echo input is in the subset");
}

#[test]
fn bounds_equal_accepted() {
    build(&[(
        "test",
        vec![with_input(
            "test.echo",
            vec![prop("count", true, int(Some(3), Some(3)))],
        )],
    )])
    .expect("minimum = maximum");
}

#[test]
fn logged_as_without_mutating_refused() {
    for l in [LoggedAs::Delete, LoggedAs::Move] {
        let a = ActionDescriptor {
            logged_as: l,
            ..action("test.write")
        };
        refused(&[("test", vec![a.clone()])], &["test.write"]);
        build(&[(
            "test",
            vec![ActionDescriptor {
                mutating: true,
                ..a
            }],
        )])
        .expect("mutating + logged_as");
    }
}

#[test]
fn bad_summary_refused() {
    for summary in ["", "two\nlines", "carriage\rreturn"] {
        let a = ActionDescriptor {
            summary: summary.into(),
            ..action("test.echo")
        };
        refused(&[("test", vec![a])], &["test.echo"]);
    }
}

// --- RG-ST03-4.2: command path, MCP name ---

#[test]
fn command_paths() {
    assert_eq!(command_path("test.echo"), ["test", "echo"]);
    assert_eq!(command_path("test.group.child"), ["test", "group", "child"]);
    assert_eq!(command_path("arbitre.test-foundation"), ["test-foundation"]);
    assert_eq!(command_path("arbitre.module.enable"), ["module", "enable"]);
    assert_eq!(command_path("plan"), ["plan"]);
}

#[test]
fn mcp_names() {
    assert_eq!(mcp_name("plan.move-item"), "plan_move-item");
    assert_eq!(mcp_name("arbitre.module.enable"), "arbitre_module_enable");
    assert_eq!(mcp_name("plan"), "plan");
}

// --- RG-ST03-1.2, 4.4: text parsers (matrix US-1 rows 3-7) ---

#[test]
fn integer_text() {
    let s = int(Some(-5), Some(100));
    assert_eq!(parse_text(&s, "42"), Ok(Value::Int(42)));
    assert_eq!(parse_text(&s, "-5"), Ok(Value::Int(-5)));
    assert_eq!(parse_text(&s, "100"), Ok(Value::Int(100)));
    for bad in ["101", "-6", "1000", "abc", "", "4.2", "1e2", " 4", "0x10"] {
        assert!(parse_text(&s, bad).is_err(), "{bad:?}");
    }
    let free = int(None, None);
    assert_eq!(
        parse_text(&free, "-9223372036854775808"),
        Ok(Value::Int(i64::MIN))
    );
    assert!(parse_text(&free, "9223372036854775808").is_err());
}

#[test]
fn date_text() {
    for ok in [
        "2024-02-29",
        "2000-02-29",
        "0001-01-01",
        "9999-12-31",
        "2026-04-30",
    ] {
        assert_eq!(parse_text(&Schema::Date, ok), Ok(s(ok)), "{ok}");
    }
    for bad in [
        "2023-02-29",
        "1900-02-29",
        "2026-02-31",
        "2026-04-31",
        "0000-01-01",
        "2026-1-01",
        "2026-01-1",
        "31/02",
        "2026-13-01",
        "2026-00-10",
        "2026-01-00",
        "10000-01-01",
        "2026-01-01 ",
        "",
    ] {
        assert!(parse_text(&Schema::Date, bad).is_err(), "{bad:?}");
    }
}

#[test]
fn enum_text() {
    let e = enum_of(&["low", "medium", "high"]);
    assert_eq!(parse_text(&e, "high"), Ok(s("high")));
    for bad in ["huge", "Low", "", "low "] {
        assert!(parse_text(&e, bad).is_err(), "{bad:?}");
    }
}

#[test]
fn string_text_length_in_characters() {
    let st = Schema::String {
        min_length: Some(1),
        max_length: Some(3),
    };
    assert_eq!(parse_text(&st, "a"), Ok(s("a")));
    assert_eq!(parse_text(&st, "abc"), Ok(s("abc")));
    assert_eq!(parse_text(&st, "été"), Ok(s("été")));
    assert!(parse_text(&st, "").is_err());
    assert!(parse_text(&st, "abcd").is_err());
    assert_eq!(parse_text(&string(), "free \n text"), Ok(s("free \n text")));
}

// --- RG-ST03-4.4, 4.5: conformance ---

fn echo_value(extra: &[(&str, Value)]) -> Value {
    let mut m: BTreeMap<String, Value> = [
        ("text", s("hi")),
        ("flag", Value::Bool(false)),
        ("tag", Value::List(vec![])),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    for (k, v) in extra {
        m.insert(k.to_string(), v.clone());
    }
    Value::Object(m)
}

#[test]
fn conforming_inputs_accepted() {
    let schema = echo_input();
    assert_eq!(conforms_input(&schema, &echo_value(&[])), Ok(()));
    let full = echo_value(&[
        ("count", Value::Int(100)),
        ("level", s("high")),
        ("due", s("2024-02-29")),
        ("tag", Value::List(vec![s("a"), s("b"), s("a")])),
        ("note", s("n")),
        ("flag", Value::Bool(true)),
    ]);
    assert_eq!(conforms_input(&schema, &full), Ok(()));
}

#[test]
fn non_conforming_inputs_refused() {
    let schema = echo_input();
    let mut missing_text = echo_value(&[]);
    let mut missing_flag = echo_value(&[]);
    let mut missing_tag = echo_value(&[]);
    if let (Value::Object(a), Value::Object(b), Value::Object(c)) =
        (&mut missing_text, &mut missing_flag, &mut missing_tag)
    {
        a.remove("text");
        b.remove("flag");
        c.remove("tag");
    }
    let bad = [
        ("missing required", missing_text),
        ("missing boolean", missing_flag),
        ("missing list", missing_tag),
        ("extra key", echo_value(&[("bogus", s("x"))])),
        ("wrong type", echo_value(&[("count", s("3"))])),
        ("above maximum", echo_value(&[("count", Value::Int(101))])),
        ("below minimum", echo_value(&[("count", Value::Int(-1))])),
        ("non-existent date", echo_value(&[("due", s("2026-02-31"))])),
        ("enum outside", echo_value(&[("level", s("huge"))])),
        ("too long", echo_value(&[("text", s(&"x".repeat(201)))])),
        (
            "list item type",
            echo_value(&[("tag", Value::List(vec![Value::Int(1)]))]),
        ),
        ("not an object", s("hi")),
    ];
    for (case, v) in bad {
        assert!(conforms_input(&schema, &v).is_err(), "{case}");
    }
}

#[test]
fn output_conformance() {
    let schema = Schema::Object(vec![
        prop("key", true, string()),
        prop(
            "items",
            false,
            list(Schema::Object(vec![prop("n", true, int(Some(0), None))])),
        ),
    ]);
    let item = |n| obj(&[("n", Value::Int(n))]);
    assert_eq!(conforms_output(&schema, &obj(&[("key", s("k"))])), Ok(()));
    assert_eq!(
        conforms_output(
            &schema,
            &obj(&[
                ("key", s("k")),
                ("items", Value::List(vec![item(0), item(3)]))
            ])
        ),
        Ok(())
    );
    for (case, v) in [
        ("missing required", obj(&[])),
        (
            "extra key",
            obj(&[("key", s("k")), ("x", Value::Bool(true))]),
        ),
        ("wrong type", obj(&[("key", Value::Int(1))])),
        (
            "nested bound",
            obj(&[("key", s("k")), ("items", Value::List(vec![item(-1)]))]),
        ),
    ] {
        assert!(conforms_output(&schema, &v).is_err(), "{case}");
    }
    assert_eq!(conforms_output(&empty(), &obj(&[])), Ok(()));
    assert!(conforms_output(&empty(), &obj(&[("x", s("y"))])).is_err());
}

// --- E-ST03-9: JSON Schema form ---

#[test]
fn echo_input_json_schema_golden() {
    let d = ActionDescriptor {
        input: echo_input(),
        ..action("test.echo")
    };
    assert_eq!(
        encode(&d.input_json_schema()),
        concat!(
            r#"{"additionalProperties":false,"properties":{"#,
            r#""count":{"description":"A count.","maximum":100,"minimum":0,"type":"integer"},"#,
            r#""due":{"description":"A due date.","format":"date","type":"string"},"#,
            r#""flag":{"description":"A flag.","type":"boolean"},"#,
            r#""level":{"description":"A level.","enum":["low","medium","high"],"type":"string"},"#,
            r#""note":{"description":"An optional note.","type":"string"},"#,
            r#""tag":{"description":"A tag, repeatable.","items":{"type":"string"},"type":"array"},"#,
            r#""text":{"description":"Text to echo.","maxLength":200,"type":"string"}"#,
            r#"},"required":["flag","tag","text"],"type":"object"}"#
        )
    );
}

#[test]
fn each_subset_type_json_schema() {
    let cases: Vec<(Schema, &str)> = vec![
        (
            Schema::String {
                min_length: Some(1),
                max_length: Some(5),
            },
            r#"{"description":"The p.","maxLength":5,"minLength":1,"type":"string"}"#,
        ),
        (string(), r#"{"description":"The p.","type":"string"}"#),
        (
            int(Some(-3), Some(7)),
            r#"{"description":"The p.","maximum":7,"minimum":-3,"type":"integer"}"#,
        ),
        (
            int(None, None),
            r#"{"description":"The p.","type":"integer"}"#,
        ),
        (
            Schema::Boolean,
            r#"{"description":"The p.","type":"boolean"}"#,
        ),
        (
            enum_of(&["b", "a"]),
            r#"{"description":"The p.","enum":["b","a"],"type":"string"}"#,
        ),
        (
            Schema::Date,
            r#"{"description":"The p.","format":"date","type":"string"}"#,
        ),
        (
            list(Schema::Date),
            r#"{"description":"The p.","items":{"format":"date","type":"string"},"type":"array"}"#,
        ),
        (
            Schema::Object(vec![prop("x", true, Schema::Boolean)]),
            r#"{"additionalProperties":false,"description":"The p.","properties":{"x":{"description":"The x.","type":"boolean"}},"required":["x"],"type":"object"}"#,
        ),
    ];
    for (schema, want) in cases {
        let d = with_output("test.read", vec![prop("p", true, schema)]);
        assert_eq!(
            encode(&d.output_json_schema()),
            format!(
                r#"{{"additionalProperties":false,"properties":{{"p":{want}}},"required":["p"],"type":"object"}}"#
            )
        );
    }
}

#[test]
fn empty_object_json_schema() {
    assert_eq!(
        encode(&action("test.noop").output_json_schema()),
        r#"{"additionalProperties":false,"properties":{},"required":[],"type":"object"}"#
    );
}

#[test]
fn string_bounds_equal_accepted() {
    let eq = Schema::String {
        min_length: Some(2),
        max_length: Some(2),
    };
    build(&[(
        "test",
        vec![with_input("test.echo", vec![prop("text", true, eq)])],
    )])
    .expect("minLength = maxLength");
}

#[test]
fn nested_output_properties_checked() {
    let bad = Schema::Object(vec![prop("Bad", true, Schema::Boolean)]);
    refused(
        &[(
            "test",
            vec![with_output("test.read", vec![prop("sub", true, bad)])],
        )],
        &["test.read"],
    );
}

#[test]
fn command_path_drops_arbitre_only_when_first() {
    assert_eq!(command_path("test.arbitre"), ["test", "arbitre"]);
}

#[test]
fn date_shape_edges() {
    assert_eq!(parse_text(&Schema::Date, "2023-02-28"), Ok(s("2023-02-28")));
    for bad in ["2026/01/01", "2026-01/01", "2026/01-01", "2026-01-011"] {
        assert!(parse_text(&Schema::Date, bad).is_err(), "{bad:?}");
    }
}

#[test]
fn required_sorted_by_name() {
    let d = with_input(
        "test.echo",
        vec![prop("zeta", true, string()), prop("alpha", true, string())],
    );
    let e = encode(&d.input_json_schema());
    assert!(e.contains(r#""required":["alpha","zeta"]"#), "{e}");
}
