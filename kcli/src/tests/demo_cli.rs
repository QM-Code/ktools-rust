use std::process::Command;

fn run_binary(path: &str, args: &[&str]) -> String {
    let output = Command::new(path)
        .args(args)
        .output()
        .expect("binary should run");
    assert!(
        output.status.success(),
        "binary failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn core_demo_processes_alpha_message() {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), &["--alpha-message", "hello"]);
    assert!(output.contains("Processing --alpha-message with value \"hello\""));
    assert!(output.contains("KCLI rust demo core import/integration check passed"));
}

#[test]
fn core_demo_prints_alpha_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_core"), &["--alpha"]);
    assert!(output.contains("Available --alpha-* options:"));
    assert!(output.contains("--alpha-enable [value]"));
}

#[test]
fn omega_demo_processes_gamma_override() {
    let output = run_binary(env!("CARGO_BIN_EXE_omega"), &["--newgamma-tag", "prod"]);
    assert!(output.contains("Processing --newgamma-tag with value \"prod\""));
    assert!(output.contains("--newgamma (gamma override)"));
}

#[test]
fn omega_demo_prints_build_help() {
    let output = run_binary(env!("CARGO_BIN_EXE_omega"), &["--build"]);
    assert!(output.contains("Available --build-* options:"));
    assert!(output.contains("--build-profile <value>"));
    assert!(output.contains("--build-clean"));
}
