use anyhow::Result;
use fs_usage_sys::FsUsageMonitorBuilder;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Starting mutation-only file system monitor...");

    // Create monitor that only watches real write/mutation events
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/Users/madhavajay/dev/icaros/workspace2/lol")
        .watch_mutations_only()  // Only watch write, delete, rename, chmod
        .exact_path_matching(true)  // Use efficient exact path matching
        .build()?;

    monitor.start()?;
    println!("Monitor started. Watching for mutations only...");
    println!("Try editing, saving, renaming, or changing permissions on files in the watched directory.");
    println!("Press Ctrl+C to stop");

    // Process events
    loop {
        match monitor.recv() {
            Ok(event) => {
                // Only show mutation events (writes, deletes, renames, chmod)
                println!(
                    "{} {} {} {}",
                    event.timestamp,
                    event.operation.pad_to(20),
                    event.path,
                    event.process_name
                );
            }
            Err(e) => {
                eprintln!("Error receiving event: {}", e);
                break;
            }
        }
    }

    Ok(())
}

trait PadTo {
    fn pad_to(&self, width: usize) -> String;
}

impl PadTo for String {
    fn pad_to(&self, width: usize) -> String {
        format!("{:<width$}", self, width = width)
    }
}