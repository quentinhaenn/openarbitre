use std::collections::BTreeMap;
use std::path::PathBuf;

use arbitre_core::Value;
use arbitre_core::{
    ActionDescriptor, ActionError, Catalog, Property, Schema, command_path, parse_text,
};
use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Outcome of a command before rendering.
pub type Outcome = Result<Value, ActionError>;

/// Command tree: one sub-command per path segment, sorted; long options from input
/// properties; global `--json`, `--store`; no short options (RG-ST03-4.7).
pub fn command(catalog: &Catalog) -> Command {
    let mut root = Node::default();
    for a in catalog.actions() {
        let node = command_path(&a.name)
            .into_iter()
            .fold(&mut root, |n, seg| n.children.entry(seg).or_default());
        node.action = Some(a);
    }
    node_command("arbitre", &root)
        .version(env!("CARGO_PKG_VERSION"))
        .propagate_version(true)
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Write one JSON result document on standard output"),
        )
        .arg(
            Arg::new("store")
                .long("store")
                .global(true)
                .value_name("PATH")
                .help("Store file (default: ARBITRE_STORE, then the platform data directory)")
                .value_parser(|s: &str| match s {
                    "" => Err("the store path must not be empty"),
                    _ => Ok(PathBuf::from(s)),
                }),
        )
}

#[derive(Default)]
struct Node<'a> {
    action: Option<&'a ActionDescriptor>,
    children: BTreeMap<&'a str, Node<'a>>,
}

fn node_command(name: &str, node: &Node) -> Command {
    let mut c = Command::new(name.to_string()).disable_help_subcommand(true);
    if let Some(a) = node.action {
        c = c.about(a.summary.clone());
        if let Schema::Object(props) = &a.input {
            c = c.args(props.iter().map(option));
        }
    }
    for (seg, child) in &node.children {
        c = c.subcommand(node_command(seg, child));
    }
    match (node.action, node.children.is_empty()) {
        (None, _) => c.subcommand_required(true),
        (_, true) => c,
        (Some(_), false) => c.args_conflicts_with_subcommands(true),
    }
}

/// Booleans and lists are never required on the command line (absent = `false` / `[]`).
fn option(p: &Property) -> Arg {
    let a = Arg::new(p.name.clone())
        .long(p.name.replace('_', "-"))
        .value_name(p.name.to_uppercase())
        .help(p.description.clone());
    match &p.schema {
        Schema::Boolean => a.action(ArgAction::SetTrue),
        Schema::List(item) => scalar(a.action(ArgAction::Append), item),
        s => scalar(a.required(p.required), s),
    }
}

fn scalar(a: Arg, s: &Schema) -> Arg {
    let t = s.clone();
    match s {
        Schema::Enum(vs) => a.value_parser(PossibleValuesParser::new(vs.clone()).map(Value::Str)),
        Schema::Integer { .. } => a
            .value_name("INTEGER")
            .allow_negative_numbers(true)
            .value_parser(move |x: &str| parse_text(&t, x)),
        Schema::Date => a
            .value_name("DATE")
            .value_parser(move |x: &str| parse_text(&t, x)),
        _ => a.value_parser(move |x: &str| parse_text(&t, x)),
    }
}

/// The selected action and the matches that carry its options.
pub fn selected<'a>(
    matches: &'a ArgMatches,
    catalog: &'a Catalog,
) -> (&'a ActionDescriptor, &'a ArgMatches) {
    let (mut path, mut leaf) = (Vec::new(), matches);
    while let Some((name, sub)) = leaf.subcommand() {
        path.push(name);
        leaf = sub;
    }
    let action = catalog
        .actions()
        .iter()
        .find(|a| command_path(&a.name) == path)
        .expect("clap only matches derived paths");
    (action, leaf)
}

fn values(m: &ArgMatches, name: &str) -> Vec<Value> {
    m.get_many::<Value>(name)
        .into_iter()
        .flatten()
        .cloned()
        .collect()
}

/// Conforming input `Value` of `action` from its parsed options (RG-ST03-4.4).
pub fn input(matches: &ArgMatches, action: &ActionDescriptor) -> Value {
    let Schema::Object(props) = &action.input else {
        unreachable!("catalogue input is an object")
    };
    let mut o = BTreeMap::new();
    for p in props {
        let v = match &p.schema {
            Schema::Boolean => Some(Value::Bool(matches.get_flag(&p.name))),
            Schema::List(_) => Some(Value::List(values(matches, &p.name))),
            _ => values(matches, &p.name).pop(),
        };
        o.extend(v.map(|v| (p.name.clone(), v)));
    }
    Value::Object(o)
}

/// Help/version → success `{"help"|"version": text}`; other clap errors → `Usage` (RG-ST03-1.1, 1.4).
pub fn usage_error(e: clap::Error) -> Outcome {
    let text = e.to_string();
    let id = match e.kind() {
        ErrorKind::DisplayHelp => return Ok(doc("help", text)),
        ErrorKind::DisplayVersion => return Ok(doc("version", text)),
        ErrorKind::UnknownArgument => "unknown_option",
        ErrorKind::InvalidSubcommand => "unknown_command",
        ErrorKind::MissingRequiredArgument => "missing_option",
        ErrorKind::MissingSubcommand => "missing_command",
        ErrorKind::ArgumentConflict => "conflicting_arguments",
        ErrorKind::InvalidValue | ErrorKind::ValueValidation | ErrorKind::InvalidUtf8 => {
            "invalid_value"
        }
        _ => "usage",
    };
    let msg = text.trim();
    Err(ActionError::usage(
        id,
        msg.strip_prefix("error: ").unwrap_or(msg),
    ))
}

fn doc(key: &str, text: String) -> Value {
    Value::Object([(key.to_string(), Value::Str(text))].into())
}

#[cfg(all(test, feature = "test-actions"))]
mod tests {
    use super::*;

    fn id(args: &[&str]) -> String {
        let catalog = crate::modules::catalog(&crate::modules::all(), |_| true).unwrap();
        let e = command(&catalog)
            .try_get_matches_from(args)
            .expect_err("malformed");
        usage_error(e).unwrap_err().id().to_string()
    }

    #[test]
    fn usage_ids_by_error_kind() {
        let echo = ["arbitre", "test", "echo"];
        let with = |extra: &[&'static str]| [&echo[..], extra].concat();
        assert_eq!(id(&with(&["--text", "a", "--bogus"])), "unknown_option");
        assert_eq!(id(&["arbitre", "test", "nope"]), "unknown_command");
        assert_eq!(id(&echo), "missing_option");
        assert_eq!(id(&["arbitre", "test"]), "missing_command");
        assert_eq!(
            id(&with(&["--text", "a", "--text", "b"])),
            "conflicting_arguments"
        );
        assert_eq!(id(&with(&["--text", "a", "--level", "x"])), "invalid_value");
        assert_eq!(id(&with(&["--text", "a", "--count", "z"])), "invalid_value");
    }
}

#[cfg(test)]
mod bare_root {
    use super::*;

    #[test]
    fn option_names_use_dashes() {
        // RG-ST03-4.3: `due_date` ⇒ `--due-date`; the input key keeps `_`.
        let a = ActionDescriptor {
            name: "arbitre.x".into(),
            summary: "s".into(),
            input: Schema::Object(vec![Property {
                name: "due_date".into(),
                description: "d".into(),
                required: true,
                schema: Schema::String {
                    min_length: None,
                    max_length: None,
                },
            }]),
            output: Schema::Object(vec![]),
            mutating: false,
            logged_as: arbitre_core::LoggedAs::None,
            requires_network: Default::default(),
        };
        let catalog = Catalog::build(&[a], &[]).unwrap();
        let m = command(&catalog)
            .try_get_matches_from(["arbitre", "x", "--due-date", "d"])
            .unwrap();
        let (action, leaf) = selected(&m, &catalog);
        let want = Value::Object([("due_date".into(), Value::Str("d".into()))].into());
        assert_eq!(input(leaf, action), want);
        let bad = command(&catalog).try_get_matches_from(["arbitre", "x", "--due_date", "d"]);
        assert!(bad.is_err());
    }

    #[test]
    fn bare_root_is_usage_even_with_an_empty_catalogue() {
        let catalog = Catalog::build(&[], &[]).unwrap();
        for args in [&["arbitre"][..], &["arbitre", "--json"]] {
            let e = command(&catalog).try_get_matches_from(args).unwrap_err();
            assert_eq!(usage_error(e).unwrap_err().id(), "missing_command");
        }
    }
}
