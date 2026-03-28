use std::process::Command;

fn run_demo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_sdk_alpha"))
        .args(args)
        .output()
        .expect("alpha demo should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn sdk_alpha_demo_processes_message() {
    let output = run_demo(&["--alpha-message", "hello"]);
    assert!(output.contains("Processing --alpha-message with value \"hello\""));
    assert!(output.contains("KCLI rust alpha demo SDK check passed"));
}
