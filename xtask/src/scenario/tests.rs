use super::*;

#[test]
fn lines_split_as_a_shell_would() {
    assert_eq!(
        words("  client.dev_area size=64 generator=\"boreal forest\" # woods"),
        ["client.dev_area", "size=64", "generator=boreal forest"]
    );
    assert_eq!(
        words("--server server.tick count=3"),
        ["--server", "server.tick", "count=3"]
    );
}

#[test]
fn comments_and_blank_lines_are_nothing() {
    assert!(words("# a comment").is_empty());
    assert!(words("   ").is_empty());
}

#[test]
fn json_params_are_kept_whole() {
    assert_eq!(
        words(r#"client.set_cells {"cells": [{"dx": 1, "dy": 0, "cell": "Wall"}]}"#),
        [
            "client.set_cells",
            r#"{"cells": [{"dx": 1, "dy": 0, "cell": "Wall"}]}"#
        ]
    );
}

#[test]
fn every_scenario_in_the_repository_names_its_methods() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scenarios");
    for entry in fs::read_dir(folder).expect("scenarios/") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "txt") {
            let text = fs::read_to_string(&path).unwrap();
            for line in text.lines() {
                let args = words(line);
                let method = args.iter().find(|arg| !arg.starts_with("--"));
                if !args.is_empty() {
                    assert!(
                        method.is_some_and(|m| m.contains('.') && !m.contains('=')),
                        "{}: {line} names no method",
                        path.display()
                    );
                }
            }
        }
    }
}
