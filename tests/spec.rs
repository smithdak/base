//! Tethers SPEC.md §7 to the clap definition so the spec's one descriptive
//! section cannot drift silently (D-016). A failure here means the CLI and
//! the spec disagree — update whichever side is wrong, deliberately.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use clap::CommandFactory;

use base_cli::cli::Cli;

/// One `| `base …` |` table row from SPEC §7, parsed into its claims.
struct DocumentedVerb {
    name: String,
    flags: Vec<String>,
    /// Present when the row lists an exhaustive `<a\|b\|c>` alternation.
    subcommands: Option<BTreeSet<String>>,
}

fn spec_cli_section() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/SPEC.md");
    let spec = fs::read_to_string(&path).expect("docs/SPEC.md is readable");
    let start = spec
        .find("## 7. The CLI")
        .expect("SPEC.md has a `## 7. The CLI` heading (renumbered? update tests/spec.rs)");
    let section = &spec[start..];
    let end = section.find("\n## ").unwrap_or(section.len());
    section[..end].to_string()
}

fn documented_verbs(section: &str) -> Vec<DocumentedVerb> {
    let mut verbs = Vec::new();
    for line in section.lines() {
        let Some(row) = line.strip_prefix("| `base ") else {
            continue;
        };
        let Some(cell_end) = row.find('`') else {
            continue;
        };
        let mut tokens = row[..cell_end].split_whitespace();
        let name = tokens.next().expect("row names a verb").to_string();
        let mut flags = Vec::new();
        let mut subcommands = None;
        for token in tokens {
            let token = token.trim_matches(|c| matches!(c, '[' | ']'));
            if let Some(flag) = token.strip_prefix("--") {
                flags.push(flag.to_string());
            } else if token.starts_with('<') && token.contains(r"\|") {
                let inner = token.trim_matches(|c| matches!(c, '<' | '>'));
                subcommands = Some(inner.split(r"\|").map(str::to_string).collect());
            }
        }
        verbs.push(DocumentedVerb {
            name,
            flags,
            subcommands,
        });
    }
    verbs
}

fn visible_subcommands(command: &clap::Command) -> BTreeSet<String> {
    command
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set() && sub.get_name() != "help")
        .map(|sub| sub.get_name().to_string())
        .collect()
}

#[test]
fn spec_section_7_matches_the_clap_definition() {
    let section = spec_cli_section();
    let documented = documented_verbs(&section);
    assert!(
        !documented.is_empty(),
        "found no `| `base …` |` table rows in SPEC §7 (table reshaped? update tests/spec.rs)"
    );

    let command = Cli::command();
    let actual = visible_subcommands(&command);
    let listed: BTreeSet<String> = documented.iter().map(|verb| verb.name.clone()).collect();
    assert_eq!(
        listed, actual,
        "SPEC §7's verb table and the visible clap subcommands disagree — \
         update docs/SPEC.md or src/cli.rs, deliberately"
    );

    for verb in &documented {
        let subcommand = command
            .find_subcommand(&verb.name)
            .expect("set equality above guarantees the subcommand exists");
        let long_flags: BTreeSet<&str> = subcommand
            .get_arguments()
            .filter_map(|arg| arg.get_long())
            .collect();
        for flag in &verb.flags {
            assert!(
                long_flags.contains(flag.as_str()),
                "SPEC §7 documents `base {} --{flag}` but the clap definition has no such flag",
                verb.name
            );
        }
        if let Some(spec_subcommands) = &verb.subcommands {
            assert_eq!(
                spec_subcommands,
                &visible_subcommands(subcommand),
                "SPEC §7's `base {}` alternation and its clap subcommands disagree",
                verb.name
            );
        }
    }

    // "Every verb supports `--json`" holds only while the flag stays global.
    let json = command
        .get_arguments()
        .find(|arg| arg.get_long() == Some("json"))
        .expect("SPEC §7 promises `--json`; the clap definition no longer has it");
    assert!(
        json.is_global_set(),
        "SPEC §7 promises `--json` on every verb, which requires the flag to be global"
    );
    assert!(
        section.contains("`--json`"),
        "SPEC §7 dropped its `--json` promise; retire this tether deliberately"
    );

    // "…, five verbs in v1" — keep the prose count honest when verbs change.
    const COUNT_WORDS: [&str; 11] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    if let Some(word) = COUNT_WORDS.get(actual.len()) {
        assert!(
            section.contains(&format!("{word} verbs")),
            "SPEC §7 should say \"{word} verbs\" to match the {} visible subcommands",
            actual.len()
        );
    }
}
