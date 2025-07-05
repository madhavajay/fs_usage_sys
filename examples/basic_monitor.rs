use anyhow::Result;
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    
    let mut builder = FsUsageMonitorBuilder::new();

    if args.len() > 1 {
        for path in &args[1..] {
            println!("Watching path: {}", path);
            builder = builder.watch_path(path);
        }
    } else {
        println!("Watching all file system events (no path filter)");
        println!("Usage: {} [path_patterns...]", args[0]);
        println!("Example: {} '/Users/*/Documents/*.txt' '/tmp/*'", args[0]);
    }

    builder = builder
        .exclude_process("fs_usage")
        .exclude_process("kernel_task");

    let mut monitor = builder.build()?;
    
    println!("Starting file system monitor...");
    monitor.start()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping monitor...");
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        match monitor.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                print_event(&event);
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
    println!("Monitor stopped");

    Ok(())
}

fn print_event(event: &FsEvent) {
    println!(
        "{} | {} [{}:{}] | {} | {}",
        event.timestamp,
        event.operation,
        event.process_name,
        event.pid,
        event.path,
        event.result
    );
}