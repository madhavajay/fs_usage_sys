use anyhow::Result;
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder, OperationType};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    let mut builder = FsUsageMonitorBuilder::new()
        .watch_writes_only() // Only watch write operations
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("Spotlight");

    if args.len() > 1 {
        for path in &args[1..] {
            println!("Watching path: {}", path);
            builder = builder.watch_path(path);
        }
    } else {
        println!("Monitoring write operations on all paths");
        println!("Usage: {} [path_patterns...]", args[0]);
        println!("Example: {} '/tmp/**/*' '/Users/*/Documents/**/*'", args[0]);
    }

    let mut monitor = builder.build()?;

    println!("Starting file system monitor (WRITES ONLY)...");
    println!("This will only show: writes, creates, deletes, moves");
    println!("Try: echo 'test' > /tmp/test.txt");
    monitor.start()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping monitor...");
        r.store(false, Ordering::SeqCst);
    })?;

    let mut event_count = 0;
    while running.load(Ordering::SeqCst) {
        match monitor.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                event_count += 1;
                print_write_event(&event, event_count);

                // Stop after 50 events to prevent spam
                if event_count >= 50 {
                    println!("\nStopping after 50 events...");
                    break;
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(e) => {
                eprintln!("Error receiving event: {}", e);
                break;
            }
        }
    }

    monitor.stop()?;
    println!("Monitor stopped after {} write events", event_count);

    Ok(())
}

fn print_write_event(event: &FsEvent, count: usize) {
    let operation_type = match event.operation.as_str() {
        op if op.contains("write") || op.contains("WrData") => "✏️  WRITE",
        op if op.contains("open") || op.contains("creat") => "📄 CREATE",
        op if op.contains("unlink") || op.contains("rmdir") => "🗑️  DELETE",
        op if op.contains("rename") => "📁 MOVE",
        _ => "💾 MODIFY",
    };

    println!("\n=== Write Event #{} ===", count);
    println!(
        "{} | {} [{}:{}]",
        operation_type, event.operation, event.process_name, event.pid
    );
    println!("📍 Path: {}", event.path);
    println!("⏰ Time: {}", event.timestamp);

    if event.process_name.contains("Cursor") || event.process_name.contains("Code") {
        println!("⚠️  AI ASSISTANT DETECTED");
    } else if event.process_name.contains("vim") || event.process_name.contains("nano") {
        println!("✏️  TEXT EDITOR");
    }
}

