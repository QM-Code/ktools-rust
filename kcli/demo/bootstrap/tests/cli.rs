use std::process::Command;

#[test]
fn bootstrap_demo_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_bootstrap"))
        .output()
        .expect("bootstrap demo should run");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Bootstrap succeeded."));
}
