use super::*;

#[test]
fn options_off_by_default_turn_on_with_their_words_in_any_case() {
    for value in ["1", "on", "true", "yes", "ON", "True", "yEs"] {
        assert!(option_value(Some(value), false), "{value}");
    }
}

#[test]
fn options_on_by_default_turn_off_with_their_words_in_any_case() {
    for value in ["0", "off", "no", "false", "OFF", "False", "nO"] {
        assert!(!option_value(Some(value), true), "{value}");
    }
}

#[test]
fn anything_else_leaves_an_option_at_its_default() {
    for default in [false, true] {
        assert_eq!(option_value(None, default), default);
        for value in ["", "maybe", "2", " true", "y", "n"] {
            assert_eq!(option_value(Some(value), default), default, "{value}");
        }
    }
    // The other side's words don't count either.
    assert!(!option_value(Some("off"), false));
    assert!(option_value(Some("on"), true));
}

#[test]
fn unset_or_unknown_levels_log_as_usual() {
    for value in [None, Some("loud"), Some("")] {
        let plugin = log_plugin_for(value);
        assert_eq!(plugin.level, Level::INFO);
        assert_eq!(plugin.filter, format!("{DEFAULT_FILTER},{SILENCED}"));
    }
}

#[test]
fn quieter_levels_apply_to_everything() {
    let plugin = log_plugin_for(Some("WARN"));
    assert_eq!(plugin.level, Level::WARN);
    assert_eq!(plugin.filter, format!("{DEFAULT_FILTER},{SILENCED}"));
}

#[test]
fn louder_levels_apply_only_to_our_crates() {
    let plugin = log_plugin_for(Some("debug"));
    assert_eq!(plugin.level, Level::INFO);
    assert!(plugin.filter.starts_with(DEFAULT_FILTER));
    assert!(plugin.filter.contains("server=DEBUG"));
}

/// Every source file in `folder` and the folders inside it.
fn source_files(folder: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(source_files(&path));
        } else {
            files.push(path);
        }
    }
    files
}

/// Every `HOW_` name in this crate's source files.
fn variables_in_the_source() -> std::collections::BTreeSet<String> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut names = std::collections::BTreeSet::new();
    for file in source_files(&folder) {
        let source = std::fs::read_to_string(file).unwrap();
        let is_part = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_';
        for (start, _) in source.match_indices("HOW_") {
            let rest = &source[start..];
            let end = rest.find(|c| !is_part(c)).unwrap_or(rest.len());
            names.insert(rest[..end].to_owned());
        }
    }
    // The prefix alone, where the source talks about the names in general.
    names.remove("HOW_");
    names
}

#[test]
fn the_user_documentation_lists_every_server_variable() {
    let page = include_str!("../../../../docs/environment-variables.md");
    let names = variables_in_the_source();
    assert!(names.contains("HOW_DATA"), "{names:?}");
    for name in names {
        assert!(
            page.contains(&format!("`{name}`")),
            "{name} is missing from docs/environment-variables.md"
        );
    }
}
