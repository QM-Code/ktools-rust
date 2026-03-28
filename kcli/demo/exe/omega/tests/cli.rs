use std::process::Command;

fn run_demo(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_omega"))
        .args(args)
        .output()
        .expect("omega demo should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("stdout should be valid UTF-8")
}

#[test]
fn omega_demo_processes_gamma_override() {
    let output = run_demo(&["--newgamma-tag", "prod"]);
    assert!(output.contains("Processing --newgamma-tag with value \"prod\""));
    assert!(output.contains("--newgamma (gamma override)"));
}

#[test]
fn omega_demo_prints_build_help() {
    let output = run_demo(&["--build"]);
    assert!(output.contains("Available --build-* options:"));
    assert!(output.contains("--build-profile <value>"));
    assert!(output.contains("--build-clean"));
}
