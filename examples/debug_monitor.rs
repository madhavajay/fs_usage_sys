use anyhow::Result;
use fs_usage_sys::FsUsageMonitorBuilder;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("fs_usage_sys=debug")
        .init();

    let args: Vec<String> = env::args().collect();

    let mut builder = FsUsageMonitorBuilder::new();

    if args.len() > 1 {
        for path in &args[1..] {
            println!("Watching path: {}", path);
            builder = builder.watch_path(path);
        }
    } else {
        println!("Monitoring ALL file system events (no path filter)");
        println!("This will show a lot of activity!");
    }

    let mut monitor = builder.build()?;

    println!("Starting file system monitor (debug mode)...");
    println!("This will show raw fs_usage output parsing");
    monitor.start()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping monitor...");
        r.store(false, Ordering::SeqCst);
    })?;

    let mut event_count = 0;
    while running.load(Ordering::SeqCst) && event_count < 100 {
        match monitor.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                event_count += 1;
                println!("\n=== Event #{} ===", event_count);
                println!("Timestamp: {}", event.timestamp);
                println!("Operation: {}", event.operation);
                println!("Process: {} (PID: {})", event.process_name, event.pid);
                println!("Path: {}", event.path);
                println!("Result: {}", event.result);
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
    println!("\nMonitor stopped after {} events", event_count);

    Ok(())
}

