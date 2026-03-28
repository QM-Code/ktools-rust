use std::process::Command;

fn run_demo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sdk_gamma"))
        .args(args)
        .output()
        .expect("gamma demo should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn sdk_gamma_demo_processes_tag() {
    let output = run_demo(&["--gamma-tag", "prod"]);
    assert!(output.contains("Processing --gamma-tag with value \"prod\""));
    assert!(output.contains("KCLI rust gamma demo SDK check passed"));
}
