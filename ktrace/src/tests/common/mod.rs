#![allow(dead_code)]

#[cfg(unix)]
use std::fs::File;
use std::process::Command;
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};
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

pub fn run_binary(path: &str, args: &[&str]) -> String {
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
