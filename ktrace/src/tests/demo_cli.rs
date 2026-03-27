mod common;

use common::run_binary;

#[test]
fn core_demo_prints_trace_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), &["--trace"]);
    assert!(output.contains("Available --trace-* options:"));
    assert!(output.contains("--trace <channels>"));
}

#[test]
fn omega_demo_prints_trace_namespaces() {
    let output = run_binary(env!("CARGO_BIN_EXE_omega"), &["--trace-namespaces"]);
    assert!(output.contains("Available trace namespaces:"));
    assert!(output.contains("alpha"));
    assert!(output.contains("beta"));
    assert!(output.contains("gamma"));
    assert!(output.contains("omega"));
}
