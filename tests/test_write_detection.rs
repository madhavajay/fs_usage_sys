use anyhow::Result;
use fs_usage_sys::FsUsageMonitorBuilder;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[test]
#[cfg(target_os = "macos")]
#[ignore = "requires sudo/root permissions to run fs_usage"]
fn test_captures_write_operations() -> Result<()> {
    // Use a test directory in the project
    let test_dir = PathBuf::from("target/test_fs_events");
    fs::create_dir_all(&test_dir)?;
    let test_file = test_dir.join("test.txt");

    // Clean up any existing test file
    let _ = fs::remove_file(&test_file);

    println!("Test directory: {}", test_dir.display());
    println!("Test file: {}", test_file.display());

    // Start monitoring with absolute path
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path(test_dir.canonicalize()?.to_str().unwrap())
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("Spotlight")
        .exclude_process("fseventsd")
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
    let events = monitor.events();

    // Give the monitor more time to start up and begin capturing
    println!("Waiting for monitor to start...");
    thread::sleep(Duration::from_secs(3));

    // Trigger write operations
    println!("Writing to file...");
    fs::write(&test_file, "test content")?;
    println!("First write completed");

    // Also test append
    thread::sleep(Duration::from_millis(500));
    println!("Writing updated content...");
    fs::write(&test_file, "updated content")?;
    println!("Second write completed");

    // Give time for events to be captured and processed
    println!("Waiting for events to be captured...");
    thread::sleep(Duration::from_secs(3));

    // Check if we captured write events
    let mut found_write = false;
    let mut found_create = false;
    let mut event_count = 0;

    println!("\nCaptured events:");
    while let Ok(event) = events.try_recv() {
        event_count += 1;
        println!(
            "  {} [{}] {} -> {}",
            event.operation, event.process_name, event.path, event.result
        );

        if event.path.contains("test.txt") {
            if event.operation.contains("write")
                || event.operation.contains("WrData")
                || event.operation.contains("WrMeta")
            {
                found_write = true;
                println!("    -> Found write operation!");
            }
            if event.operation.contains("open") || event.operation.contains("creat") {
                found_create = true;
                println!("    -> Found create/open operation!");
            }
        }
    }

    monitor.stop()?;

    println!("\nTotal events captured: {}", event_count);
    println!("Found write operation: {}", found_write);
    println!("Found create operation: {}", found_create);

    // Clean up
    let _ = fs::remove_file(&test_file);

    // We should have captured at least a create/open event
    if event_count == 0 {
        eprintln!("WARNING: No events were captured. This might be due to:");
        eprintln!("  - fs_usage needs time to start up");
        eprintln!("  - The monitored path might not match exactly");
        eprintln!("  - System permissions or security policies");
        eprintln!("Consider running the test manually with: sudo cargo test test_captures_write_operations -- --ignored --nocapture");
    }

    assert!(
        event_count > 0,
        "No events were captured - fs_usage might need more startup time or path matching issues"
    );
    assert!(
        found_create || found_write,
        "Failed to capture any write or create operations for the test file"
    );

    Ok(())
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "requires sudo/root permissions to run fs_usage"]
fn test_write_only_filter() -> Result<()> {
    // Use a test directory in the project
    let test_dir = PathBuf::from("target/test_fs_events_write");
    fs::create_dir_all(&test_dir)?;
    let test_file = test_dir.join("write_test.txt");

    // Clean up any existing test file
    let _ = fs::remove_file(&test_file);

    println!("Test directory: {}", test_dir.display());
    println!("Test file: {}", test_file.display());

    // Monitor with write-only filter
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_writes_only()
        .watch_path(test_dir.canonicalize()?.to_str().unwrap())
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("fseventsd")
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
    let events = monitor.events();

    println!("Waiting for monitor to start...");
    thread::sleep(Duration::from_secs(3));

    // Perform various operations
    println!("Writing initial content...");
    fs::write(&test_file, "initial content")?;
    thread::sleep(Duration::from_millis(500));

    println!("Reading file (should not be captured)...");
    let _ = fs::read(&test_file)?; // Read operation - should not be captured
    thread::sleep(Duration::from_millis(500));

    println!("Writing modified content...");
    fs::write(&test_file, "modified content")?;

    println!("Waiting for events to be captured...");
    thread::sleep(Duration::from_secs(3));

    // Check that we only captured write-related operations
    let mut has_read_operation = false;
    let mut has_write_operation = false;

    while let Ok(event) = events.try_recv() {
        if event.path.contains("write_test.txt") {
            println!("Captured: {} [{}]", event.operation, event.process_name);

            if event.operation.contains("read") || event.operation.contains("RdData") {
                has_read_operation = true;
            }

            if event.operation.contains("write")
                || event.operation.contains("WrData")
                || event.operation.contains("open")
                || event.operation.contains("creat")
            {
                has_write_operation = true;
            }
        }
    }

    monitor.stop()?;

    // Clean up
    let _ = fs::remove_file(&test_file);

    if !has_write_operation {
        eprintln!("WARNING: No write operations were captured. This might be due to:");
        eprintln!("  - fs_usage needs time to start up");
        eprintln!("  - The monitored path might not match exactly");
        eprintln!("  - System permissions or security policies");
    }

    assert!(
        !has_read_operation,
        "Read operations should not be captured with watch_writes_only()"
    );
    assert!(
        has_write_operation,
        "Write operations should be captured - fs_usage might need more startup time"
    );

    Ok(())
}
