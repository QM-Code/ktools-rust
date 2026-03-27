mod common;

use common::run_binary;

#[test]
fn core_demo_prints_trace_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), "core", &["--trace"]);
    assert!(output.contains("Available --trace-* options:"));
    assert!(output.contains("--trace <channels>"));
}

#[test]
fn omega_demo_prints_trace_namespaces() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_omega"),
        "omega",
        &["--trace-namespaces"],
    );
    assert!(output.contains("Available trace namespaces:"));
    assert!(output.contains("alpha"));
    assert!(output.contains("beta"));
    assert!(output.contains("gamma"));
    assert!(output.contains("omega"));
}

#[test]
fn bootstrap_demo_runs() {
    let output = run_binary(env!("CARGO_BIN_EXE_bootstrap"), "bootstrap", &[]);
    assert!(output.contains("Bootstrap succeeded."));
    assert!(output.contains("[bootstrap] [bootstrap]"));
}

#[test]
fn sdk_alpha_demo_runs() {
    let output = run_binary(env!("CARGO_BIN_EXE_sdk_alpha"), "sdk_alpha", &[]);
    assert!(output.contains("KTRACE rust alpha demo SDK check passed"));
    assert!(output.contains("[alpha] [net]"));
    assert!(output.contains("[alpha] [info]"));
}

#[test]
fn omega_demo_traces_imported_namespaces() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_omega"),
        "omega",
        &["--trace", "*.*.*.*"],
    );
    assert!(output.contains("[omega] [deep.branch.leaf]"));
    assert!(output.contains("[alpha] [net]"));
    assert!(output.contains("[beta] [io]"));
    assert!(output.contains("[gamma] [physics]"));
}
