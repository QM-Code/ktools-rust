mod common;

use common::run_binary;

#[test]
fn core_demo_processes_alpha_message() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_core"),
        "core",
        &["--alpha-message", "hello"],
    );
    assert!(output.contains("Processing --alpha-message with value \"hello\""));
    assert!(output.contains("KCLI rust demo core import/integration check passed"));
}

#[test]
fn core_demo_prints_alpha_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), "core", &["--alpha"]);
    assert!(output.contains("Available --alpha-* options:"));
    assert!(output.contains("--alpha-enable [value]"));
}

#[test]
fn omega_demo_processes_gamma_override() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_omega"),
        "omega",
        &["--newgamma-tag", "prod"],
    );
    assert!(output.contains("Processing --newgamma-tag with value \"prod\""));
    assert!(output.contains("--newgamma (gamma override)"));
}

#[test]
fn omega_demo_prints_build_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_omega"), "omega", &["--build"]);
    assert!(output.contains("Available --build-* options:"));
    assert!(output.contains("--build-profile <value>"));
    assert!(output.contains("--build-clean"));
}

#[test]
fn bootstrap_demo_runs() {
    let output = run_binary(env!("CARGO_BIN_EXE_bootstrap"), "bootstrap", &[]);
    assert!(output.contains("Bootstrap succeeded."));
}

#[test]
fn sdk_alpha_demo_processes_message() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_sdk_alpha"),
        "sdk_alpha",
        &["--alpha-message", "hello"],
    );
    assert!(output.contains("Processing --alpha-message with value \"hello\""));
    assert!(output.contains("KCLI rust alpha demo SDK check passed"));
}

#[test]
fn sdk_beta_demo_processes_workers() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_sdk_beta"),
        "sdk_beta",
        &["--beta-workers", "8"],
    );
    assert!(output.contains("Processing --beta-workers with value \"8\""));
    assert!(output.contains("KCLI rust beta demo SDK check passed"));
}

#[test]
fn sdk_gamma_demo_processes_tag() {
    let output = run_binary(
        env!("CARGO_BIN_EXE_sdk_gamma"),
        "sdk_gamma",
        &["--gamma-tag", "prod"],
    );
    assert!(output.contains("Processing --gamma-tag with value \"prod\""));
    assert!(output.contains("KCLI rust gamma demo SDK check passed"));
}
