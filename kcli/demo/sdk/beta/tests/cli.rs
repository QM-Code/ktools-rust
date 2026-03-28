use std::process::Command;

fn run_demo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sdk_beta"))
        .args(args)
        .output()
        .expect("beta demo should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn sdk_beta_demo_processes_workers() {
    let output = run_demo(&["--beta-workers", "8"]);
    assert!(output.contains("Processing --beta-workers with value \"8\""));
    assert!(output.contains("KCLI rust beta demo SDK check passed"));
}
