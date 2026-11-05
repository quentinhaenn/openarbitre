//! Published exit-code table (ST-03 E-ST03-6; RG-ST03-3.7, AC-3.7).

use arbitre_core::Category;

const ALL: [Category; 5] = [
    Category::Success,
    Category::Refusal,
    Category::Usage,
    Category::ToolError,
    Category::ModuleNotEnabled,
];

/// Rows `(category, json name, code)` of the table under `## Exit codes` in `README.md`.
fn readme_rows() -> Vec<(String, Option<String>, u8)> {
    let readme = include_str!("../../../README.md");
    let section = readme
        .split("\n## Exit codes\n")
        .nth(1)
        .expect("README section `## Exit codes`");
    let section = section.split("\n## ").next().unwrap();
    let clean = |c: &str| c.trim().trim_matches('`').to_string();
    section
        .lines()
        .filter(|l| l.starts_with('|'))
        .skip(2)
        .map(|l| {
            let cells: Vec<&str> = l.trim_matches('|').split('|').collect();
            let json = clean(cells[1]);
            (
                clean(cells[0]),
                (json != "—").then_some(json),
                clean(cells[2]).parse().expect("exit code"),
            )
        })
        .collect()
}

#[test]
fn readme_table_matches_the_code() {
    let want: Vec<(String, Option<String>, u8)> = ALL
        .iter()
        .map(|c| {
            (
                format!("{c:?}"),
                c.json_name().map(String::from),
                c.exit_code(),
            )
        })
        .collect();
    assert_eq!(readme_rows(), want);
}
