use std::collections::BTreeMap;
use std::path::PathBuf;

use arbitre_core::value::Value;
use arbitre_core::{
    ActionDescriptor, ActionError, Catalog, Property, Schema, command_path, parse_text,
};
use clap::builder::PossibleValuesParser;
use clap::error::ErrorKind;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Outcome of a command before rendering.
pub type Outcome = Result<Value, ActionError>;

/// clap takes `&'static str` without its `string` feature; one command per process.
fn leak(s: &str) -> &'static str {
    Box::leak(s.into())
}

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
    let mut c = Command::new(leak(name)).disable_help_subcommand(true);
    if let Some(a) = node.action {
        c = c.about(leak(&a.summary));
        if let Schema::Object(props) = &a.input {
            c = c.args(props.iter().map(option));
        }
    }
    for (seg, child) in &node.children {
        c = c.subcommand(node_command(seg, child));
    }
    match (node.action, node.children.is_empty()) {
        (_, true) => c,
        (Some(_), false) => c.args_conflicts_with_subcommands(true),
        (None, false) => c.subcommand_required(true),
    }
}

/// Booleans and lists are never required on the command line (absent = `false` / `[]`).
fn option(p: &Property) -> Arg {
    let name = leak(&p.name);
    let a = Arg::new(name)
        .long(name)
        .value_name(leak(&p.name.to_uppercase()))
        .help(leak(&p.description));
    match &p.schema {
        Schema::Boolean => a.action(ArgAction::SetTrue),
        Schema::List(item) => scalar(a.action(ArgAction::Append), item),
        s => scalar(a.required(p.required), s),
    }
}

fn scalar(a: Arg, s: &Schema) -> Arg {
    match s {
        Schema::Enum(vs) => a.value_parser(PossibleValuesParser::new(vs.iter().map(|v| leak(v)))),
        _ => {
            let a = match s {
                Schema::Integer { .. } => a.value_name("INTEGER").allow_negative_numbers(true),
                Schema::Date => a.value_name("DATE"),
                _ => a,
            };
            let s = s.clone();
            a.value_parser(move |t: &str| parse_text(&s, t))
        }
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

fn values(m: &ArgMatches, name: &str, s: &Schema) -> Vec<Value> {
    if matches!(s, Schema::Enum(_)) {
        let it = m.get_many::<String>(name).into_iter().flatten();
        it.map(|x| Value::Str(x.clone())).collect()
    } else {
        m.get_many::<Value>(name)
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }
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
            Schema::List(item) => Some(Value::List(values(matches, &p.name, item))),
            s => values(matches, &p.name, s).pop(),
        };
        o.extend(v.map(|v| (p.name.clone(), v)));
    }
    Value::Object(o)
}

/// Help/version → success `{"help"|"version": text}`; other clap errors → `Usage` (RG-ST03-1.1, 1.4).
pub fn usage_error(e: clap::Error) -> Outcome {
    let text = e.to_string();
    let key = match e.kind() {
        ErrorKind::DisplayHelp => "help",
        ErrorKind::DisplayVersion => "version",
        k => {
            let id = match k {
                ErrorKind::UnknownArgument => "unknown_option",
                ErrorKind::InvalidSubcommand => "unknown_command",
                ErrorKind::MissingRequiredArgument => "missing_option",
                ErrorKind::MissingSubcommand => "missing_command",
                ErrorKind::ArgumentConflict => "repeated_option",
                ErrorKind::InvalidValue | ErrorKind::ValueValidation | ErrorKind::InvalidUtf8 => {
                    "invalid_value"
                }
                _ => "usage",
            };
            let msg = text.trim();
            let msg = msg.strip_prefix("error: ").unwrap_or(msg);
            return Err(ActionError::usage(id, msg));
        }
    };
    Ok(Value::Object([(key.to_string(), Value::Str(text))].into()))
}

#[cfg(all(test, feature = "test-actions"))]
mod tests {
    use super::*;

    fn id(args: &[&str]) -> String {
        let catalog = crate::modules::catalog(&crate::modules::all()).unwrap();
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
            "repeated_option"
        );
        assert_eq!(id(&with(&["--text", "a", "--level", "x"])), "invalid_value");
        assert_eq!(id(&with(&["--text", "a", "--count", "z"])), "invalid_value");
    }
}
