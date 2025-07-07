use anyhow::Result;
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
#[cfg(target_os = "macos")]
fn test_captures_write_operations() -> Result<()> {
    // Create a temp directory
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    
    println!("Test directory: {}", temp_dir.path().display());
    println!("Test file: {}", test_file.display());
    
    // Start monitoring
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path(&format!("{}/**/*", temp_dir.path().to_str().unwrap()))
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("Spotlight")
        .build()?;
    
    monitor.start()?;
    let events = monitor.events();
    
    // Give the monitor time to start
    thread::sleep(Duration::from_millis(500));
    
    // Trigger write operations
    println!("Writing to file...");
    fs::write(&test_file, "test content")?;
    
    // Also test append
    thread::sleep(Duration::from_millis(100));
    fs::write(&test_file, "updated content")?;
    
    // Give time for events to be captured
    thread::sleep(Duration::from_millis(1000));
    
    // Check if we captured write events
    let mut found_write = false;
    let mut found_create = false;
    let mut event_count = 0;
    
    println!("\nCaptured events:");
    while let Ok(event) = events.try_recv() {
        event_count += 1;
        println!("  {} [{}] {} -> {}", 
            event.operation, 
            event.process_name, 
            event.path,
            event.result
        );
        
        if event.path.contains("test.txt") {
            if event.operation.contains("write") || 
               event.operation.contains("WrData") || 
               event.operation.contains("WrMeta") {
                found_write = true;
                println!("    -> Found write operation!");
            }
            if event.operation.contains("open") || 
               event.operation.contains("creat") {
                found_create = true;
                println!("    -> Found create/open operation!");
            }
        }
    }
    
    monitor.stop()?;
    
    println!("\nTotal events captured: {}", event_count);
    println!("Found write operation: {}", found_write);
    println!("Found create operation: {}", found_create);
    
    // We should have captured at least a create/open event
    assert!(event_count > 0, "No events were captured");
    assert!(found_create || found_write, 
        "Failed to capture any write or create operations for the test file");
    
    Ok(())
}

#[test]
#[cfg(target_os = "macos")]
fn test_write_only_filter() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("write_test.txt");
    
    // Monitor with write-only filter
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_writes_only()
        .watch_path(&format!("{}/**/*", temp_dir.path().to_str().unwrap()))
        .exclude_process("mds")
        .exclude_process("mdworker")
        .build()?;
    
    monitor.start()?;
    let events = monitor.events();
    
    thread::sleep(Duration::from_millis(500));
    
    // Perform various operations
    fs::write(&test_file, "initial content")?;
    thread::sleep(Duration::from_millis(100));
    let _ = fs::read(&test_file)?; // Read operation - should not be captured
    thread::sleep(Duration::from_millis(100));
    fs::write(&test_file, "modified content")?;
    
    thread::sleep(Duration::from_millis(1000));
    
    // Check that we only captured write-related operations
    let mut has_read_operation = false;
    let mut has_write_operation = false;
    
    while let Ok(event) = events.try_recv() {
        if event.path.contains("write_test.txt") {
            println!("Captured: {} [{}]", event.operation, event.process_name);
            
            if event.operation.contains("read") || 
               event.operation.contains("RdData") {
                has_read_operation = true;
            }
            
            if event.operation.contains("write") || 
               event.operation.contains("WrData") ||
               event.operation.contains("open") ||
               event.operation.contains("creat") {
                has_write_operation = true;
            }
        }
    }
    
    monitor.stop()?;
    
    assert!(!has_read_operation, "Read operations should not be captured with watch_writes_only()");
    assert!(has_write_operation, "Write operations should be captured");
    
    Ok(())
}