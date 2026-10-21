//! The published JSON schema must describe exactly the config this binary reads, and the shipped
//! examples must load cleanly. CI additionally validates the examples with a real JSON Schema
//! validator; this test is the drift alarm that runs on every `cargo test`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use resolve_merge_conflicts::cli::config_cmd::starter_config;
use resolve_merge_conflicts::config::{self, Location, Rule};
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn schema() -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join("schema/config.json")).unwrap())
        .unwrap()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

#[test]
fn the_schema_lists_exactly_the_keys_the_config_struct_has() {
    // Every field set, so every key serialises.
    let full: Value = serde_json::from_str(
        r#"{"$schema":"x","agent":"claude","prompt":"p","promptFile":"f","editor":"e","autoStage":true,
            "claude":{"model":"m","models":["m"]},"codex":{"model":"m","models":["m"]}}"#,
    )
    .unwrap();
    let loaded = config::load_from(&Location {
        path: write_temp(&full.to_string()),
        rule: Rule::Flag,
    })
    .unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let serialised = serde_json::to_value(&loaded.config).unwrap();

    let schema = schema();
    assert_eq!(keys(&schema["properties"]), keys(&serialised));
    assert_eq!(
        keys(&schema["definitions"]["agent"]["properties"]),
        keys(&serialised["claude"])
    );
}

fn write_temp(text: &str) -> PathBuf {
    let dir = tempfile::tempdir().unwrap().keep();
    let path = dir.join("config.json");
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn the_schema_forbids_unknown_keys_so_editors_catch_typos() {
    let schema = schema();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["definitions"]["agent"]["additionalProperties"],
        false
    );
}

#[test]
fn the_agent_enum_matches_the_agents_the_tool_knows() {
    let schema = schema();
    let listed: Vec<&str> = schema["properties"]["agent"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(listed, resolve_merge_conflicts::agent::names());
}

#[test]
fn every_shipped_example_and_the_starter_config_load_without_warnings() {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root().join("examples/config"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.push(write_temp(&starter_config()));
    assert!(paths.len() >= 3);
    for path in paths {
        let loaded = config::load_from(&Location {
            path: path.clone(),
            rule: Rule::Flag,
        })
        .unwrap_or_else(|error| panic!("{}: {error:#}", path.display()));
        assert!(
            loaded.warnings.is_empty(),
            "{}: {:?}",
            path.display(),
            loaded.warnings
        );
    }
}
