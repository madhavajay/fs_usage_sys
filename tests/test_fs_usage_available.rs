use std::process::Command;

#[test]
#[cfg(target_os = "macos")]
fn test_fs_usage_requires_sudo() {
    // Test that fs_usage exists and requires sudo
    let output = Command::new("fs_usage")
        .arg("-t")
        .arg("1")
        .arg("echo")
        .output()
        .expect("Failed to execute fs_usage");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("must be run as root"),
        "fs_usage should require root permissions"
    );
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "requires sudo/root permissions"]
fn test_fs_usage_with_sudo() {
    // This test verifies fs_usage works when run with sudo
    // It should be run in CI with appropriate permissions
    let output = Command::new("sudo")
        .arg("-n") // non-interactive
        .arg("fs_usage")
        .arg("-t")
        .arg("1")
        .arg("-f")
        .arg("pathname")
        .arg("echo")
        .output();

    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if stderr.contains("password is required") {
                    eprintln!("Test skipped: sudo requires password");
                    return;
                }
                if stderr.contains("Resource busy") || stderr.contains("ktrace_start") {
                    eprintln!(
                        "Test skipped: Another fs_usage or ktrace process is already running"
                    );
                    return;
                }
                panic!("fs_usage failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Test skipped: Could not run sudo command: {}", e);
        }
    }
}
