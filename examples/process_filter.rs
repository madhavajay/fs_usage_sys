use anyhow::Result;
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder};
use std::env;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let current_pid = process::id();
    println!("Current process PID: {}", current_pid);

    let mut builder = FsUsageMonitorBuilder::new()
        .exclude_pid(current_pid)
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("Spotlight");

    if args.len() > 1 {
        for path in &args[1..] {
            println!("Adding watch path: {}", path);
            builder = builder.watch_path(path);
        }
    } else {
        println!("Using default paths: /Users/*/Documents/**/* and /tmp/**/*");
        builder = builder
            .watch_path("/Users/*/Documents/**/*")
            .watch_path("/tmp/**/*");
    }

    let mut monitor = builder.build()?;

    println!("Starting monitor with process filtering...");
    println!("Excluding: current process, mds, mdworker, Spotlight");
    
    monitor.start()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nStopping monitor...");
        r.store(false, Ordering::SeqCst);
    })?;

    let mut claude_events = Vec::new();
    let mut editor_events = Vec::new();
    let mut other_events = Vec::new();

    while running.load(Ordering::SeqCst) {
        match monitor.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                categorize_event(&event, &mut claude_events, &mut editor_events, &mut other_events);
                
                println!("\n--- Event ---");
                println!("Process: {} (PID: {})", event.process_name, event.pid);
                println!("Operation: {}", event.operation);
                println!("Path: {}", event.path);
                println!("Result: {}", event.result);
                
                if is_claude_process(&event.process_name) {
                    println!("⚠️  CLAUDE DETECTED - File change by AI assistant");
                } else if is_editor_process(&event.process_name) {
                    println!("✏️  EDITOR - File change by text editor");
                } else {
                    println!("🔧 OTHER - File change by: {}", event.process_name);
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
    
    println!("\n=== Summary ===");
    println!("Claude events: {}", claude_events.len());
    println!("Editor events: {}", editor_events.len());
    println!("Other events: {}", other_events.len());

    Ok(())
}

fn is_claude_process(process_name: &str) -> bool {
    let claude_processes = ["node", "electron", "Cursor", "Code", "code"];
    claude_processes.iter().any(|&p| process_name.contains(p))
}

fn is_editor_process(process_name: &str) -> bool {
    let editor_processes = ["vim", "nvim", "emacs", "nano", "TextEdit", "Sublime Text"];
    editor_processes.iter().any(|&p| process_name.contains(p))
}

fn categorize_event(
    event: &FsEvent,
    claude_events: &mut Vec<FsEvent>,
    editor_events: &mut Vec<FsEvent>,
    other_events: &mut Vec<FsEvent>,
) {
    if is_claude_process(&event.process_name) {
        claude_events.push(event.clone());
    } else if is_editor_process(&event.process_name) {
        editor_events.push(event.clone());
    } else {
        other_events.push(event.clone());
    }
}