use anyhow::Result;
use fs_usage_sys::FsUsageMonitorBuilder;
use std::thread;
use std::time::Duration;

#[test]
#[cfg(target_os = "macos")]
fn test_monitor_can_start_and_stop() -> Result<()> {
    // This test just verifies the monitor can start and stop without errors
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp")
        .exclude_process("mds")
        .exclude_process("mdworker")
        .build()?;

    // Should fail to start without sudo
    match monitor.start() {
        Ok(_) => {
            // If it somehow started (maybe in CI with sudo), stop it
            thread::sleep(Duration::from_millis(100));
            monitor.stop()?;
        }
        Err(e) => {
            // Expected: fs_usage requires root
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("must be run as root")
                    || error_msg.contains("exited with code 1")
                    || error_msg.contains("Failed to start fs_usage"),
                "Expected permission error, got: {}",
                error_msg
            );
        }
    }

    Ok(())
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "requires sudo/root permissions"]
fn test_monitor_lifecycle_with_sudo() -> Result<()> {
    // This test should work when run with sudo
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp")
        .exclude_process("mds")
        .exclude_process("mdworker")
        .build()?;

    match monitor.start() {
        Ok(_) => {}
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Resource busy") || error_msg.contains("ktrace_start") {
                eprintln!("Test skipped: Another fs_usage or ktrace process is already running");
                return Ok(());
            }
            return Err(e);
        }
    }

    // Let it run for a bit
    thread::sleep(Duration::from_secs(1));

    // Should be able to access events channel
    let events = monitor.events();

    // Drain any events (we don't care about specific events here)
    while events.try_recv().is_ok() {
        // Just drain
    }

    // Should stop cleanly
    monitor.stop()?;

    Ok(())
}
