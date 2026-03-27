#![allow(dead_code)]

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(unix)]
use std::sync::{Mutex, OnceLock};

#[cfg(unix)]
fn stdout_guard() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(unix)]
pub fn capture_stdout<F>(action: F) -> String
where
    F: FnOnce(),
{
    let _guard = stdout_guard().lock().expect("stdout lock should work");

    let capture_path = std::env::temp_dir().join(format!(
        "ktrace-test-{}-{}.log",
        std::process::id(),
        std::thread::current().name().unwrap_or("thread")
    ));
    let capture_file = File::create(&capture_path).expect("capture file should be created");
    let capture_fd = capture_file.as_raw_fd();

    let stdout_fd: RawFd = std::io::stdout().as_raw_fd();
    let original_stdout = unsafe { libc::dup(stdout_fd) };
    assert!(original_stdout >= 0, "dup(stdout) should succeed");

    let redirect_result = unsafe { libc::dup2(capture_fd, stdout_fd) };
    assert!(redirect_result >= 0, "dup2 should redirect stdout");

    action();

    std::io::Write::flush(&mut std::io::stdout()).expect("stdout flush should succeed");
    let restore_result = unsafe { libc::dup2(original_stdout, stdout_fd) };
    assert!(restore_result >= 0, "dup2 should restore stdout");
    unsafe {
        libc::close(original_stdout);
    }
    drop(capture_file);

    std::fs::read_to_string(&capture_path).expect("captured stdout should be readable")
}

#[cfg(not(unix))]
pub fn capture_stdout<F>(action: F) -> String
where
    F: FnOnce(),
{
    action();
    String::new()
}

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
