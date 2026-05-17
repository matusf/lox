use std::process::Command;
use std::sync::Once;

static BUILD: Once = Once::new();

fn ensure_built() {
    BUILD.call_once(|| {
        let status = Command::new("cargo")
            .args([
                "build",
                "--bin",
                "codecrafters-shim",
                "--features",
                "test-harness",
            ])
            .status()
            .unwrap_or_else(|e| panic!("{}", format!("cargo build failed: {e}")));
        assert!(status.success())
    });
}

fn run_case(json_path: &str) {
    ensure_built();
    let dir = std::env::current_dir().expect("current dir");
    let json = std::fs::read_to_string(json_path).expect("read json");

    let status = Command::new("tester")
        .env("CODECRAFTERS_REPOSITORY_DIR", dir.join("tests"))
        .env("CODECRAFTERS_TEST_CASES_JSON", json)
        .status()
        .expect("run tester");

    assert!(status.success(), "failed case: {json_path}");
}

#[test]
fn test_tokenizer() {
    run_case("./tests/cases/tokenizer.json");
}
#[test]
fn test_parser() {
    run_case("./tests/cases/parser.json");
}
#[test]
fn test_eval() {
    run_case("./tests/cases/eval.json");
}
#[test]
fn test_statements() {
    run_case("./tests/cases/statements.json");
}
#[test]
fn test_control_flow() {
    run_case("./tests/cases/control_flow.json");
}
#[test]
fn test_functions() {
    run_case("./tests/cases/functions.json");
}
#[test]
fn test_resolving() {
    run_case("./tests/cases/resolving.json");
}
