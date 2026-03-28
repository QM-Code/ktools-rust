use std::process::Command;

fn run_demo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_core"))
        .args(args)
        .output()
        .expect("core demo should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn core_demo_processes_alpha_message() {
    let output = run_demo(&["--alpha-message", "hello"]);
    assert!(output.contains("Processing --alpha-message with value \"hello\""));
    assert!(output.contains("KCLI rust demo core import/integration check passed"));
}

#[test]
fn core_demo_prints_alpha_help() {
    let output = run_demo(&["--alpha"]);
    assert!(output.contains("Available --alpha-* options:"));
    assert!(output.contains("--alpha-enable [value]"));
}
