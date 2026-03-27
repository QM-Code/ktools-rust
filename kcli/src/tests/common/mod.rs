use std::path::{Path, PathBuf};
use std::process::Command;

fn fallback_binary_path(bin_name: &str) -> PathBuf {
    let current_exe = std::env::current_exe().expect("test binary path should resolve");
    current_exe
        .parent()
        .and_then(Path::parent)
        .expect("test binary should live under target/*/deps")
        .join(bin_name)
}

pub fn resolve_binary_path(reported_path: &str, bin_name: &str) -> PathBuf {
    let reported = PathBuf::from(reported_path);
    if reported.is_file() {
        return reported;
    }

    let fallback = fallback_binary_path(bin_name);
    assert!(
        fallback.is_file(),
        "binary '{bin_name}' was not found at '{reported_path}' or '{}'",
        fallback.display()
    );
    fallback
}

pub fn run_binary(reported_path: &str, bin_name: &str, args: &[&str]) -> String {
    let binary = resolve_binary_path(reported_path, bin_name);
    let output = Command::new(&binary)
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
