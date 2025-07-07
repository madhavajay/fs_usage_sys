# fs_usage_sys

[![Crates.io](https://img.shields.io/crates/v/fs_usage_sys.svg)](https://crates.io/crates/fs_usage_sys)
[![Documentation](https://docs.rs/fs_usage_sys/badge.svg)](https://docs.rs/fs_usage_sys)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![macOS](https://img.shields.io/badge/platform-macOS-blue)](https://www.apple.com/macos/)

A Rust library that wraps macOS's `fs_usage` command to monitor file system events with support for glob patterns, process filtering, and operation type filtering.

## Features

- 🔍 **Real-time file system monitoring** on macOS using `fs_usage`
- 🎯 **Path filtering** with glob patterns (`/Users/*/Documents/**/*.txt`)
- 🚫 **Process filtering** by PID or process name
- ⚡ **Operation type filtering** (reads, writes, creates, deletes, etc.)
- 🤖 **AI assistant detection** - distinguish between Claude/IDE vs manual text editor changes
- 📡 **Event streaming** via channels for real-time processing
- 🛡️ **Noise reduction** - filter out metadata operations

## Quick Start

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};

// Monitor file system operations
let mut monitor = FsUsageMonitorBuilder::new()
    .watch_path("/Users/*/Documents/**/*")
    .exclude_process("mds")
    .exclude_process("mdworker")
    .build()?;

monitor.start()?;

while let Ok(event) = monitor.recv() {
    // Detect write operations (see docs/detecting_writes.md for patterns)
    if is_write_operation(&event) {
        if event.process_name.contains("Cursor") {
            println!("🤖 AI modified: {}", event.path);
        } else {
            println!("👤 Manual edit: {}", event.path);
        }
    }
}
```

**Important**: Don't use `watch_writes_only()` as it filters too aggressively. See [Detecting Write Operations](docs/detecting_writes.md) for comprehensive write detection patterns.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fs_usage_sys = "0.1.0"
```

Or use cargo add:

```bash
cargo add fs_usage_sys
```

## Requirements

- **macOS only** (uses the `fs_usage` command)
- **Rust 1.70+**
- **Root/sudo permissions** to run `fs_usage`

## Core Concepts

### Operation Types

The library categorizes file system operations into logical groups:

- **`Read`** - Reading file contents (`read`, `pread`, `RdData`)
- **`Write`** - Writing file contents (`write`, `pwrite`, `WrData`, `ftruncate`)
- **`Create`** - Creating files/directories (`open`, `creat`, `mkdir`)
- **`Delete`** - Removing files/directories (`unlink`, `rmdir`)
- **`Move`** - Renaming/moving (`rename`, `renameat`)
- **`Access`** - Checking file existence/permissions (`access`, `stat64`)
- **`Metadata`** - Reading file attributes (`getxattr`, `getattrlist`)
- **`All`** - No filtering (default)

### Event Structure

```rust
pub struct FsEvent {
    pub timestamp: String,      // "23:52:52.781431"
    pub process_name: String,   // "vim", "Cursor", "touch"
    pub pid: u32,              // Process ID
    pub operation: String,      // "write", "open", "unlink"
    pub path: String,          // "/tmp/test.txt"
    pub result: String,        // "OK" or error code
}
```

## Usage Examples

### 1. Basic File Monitoring

```rust
use fs_usage_sys::FsUsageMonitorBuilder;

let mut monitor = FsUsageMonitorBuilder::new()
    .watch_path("/tmp/**/*")
    .build()?;

monitor.start()?;

while let Ok(event) = monitor.recv() {
    println!("{} {} {}", event.process_name, event.operation, event.path);
}
```

### 2. AI Assistant vs Manual Edit Detection

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use std::process;

let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/path/to/project/**/*")
    .watch_writes_only()  // Only modifications, not reads
    .exclude_pid(process::id())  // Exclude self
    .exclude_process("mds")
    .build()?;

monitor.start()?;

while let Ok(event) = monitor.recv() {
    match classify_source(&event.process_name) {
        SourceType::AiAssistant => {
            println!("🤖 AI modified: {}", event.path);
            // Decision logic: approve/reject AI changes
            if should_reject_ai_change(&event) {
                println!("❌ Rejecting AI change");
                // Implement rollback logic
            }
        }
        SourceType::TextEditor => {
            println!("✏️ Manual edit: {}", event.path);
        }
        SourceType::System => {
            println!("🔧 System process: {}", event.path);
        }
    }
}

fn classify_source(process_name: &str) -> SourceType {
    if ["Cursor", "Code", "code", "node"].iter().any(|&p| process_name.contains(p)) {
        SourceType::AiAssistant
    } else if ["vim", "nvim", "emacs", "nano"].iter().any(|&p| process_name.contains(p)) {
        SourceType::TextEditor
    } else {
        SourceType::System
    }
}
```

### 3. Advanced Filtering

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};

// Monitor specific operations only
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/Users/*/Documents/**/*")
    .watch_operations([
        OperationType::Write,
        OperationType::Create,
        OperationType::Delete
    ])
    .exclude_processes(["mds", "mdworker", "Spotlight"])
    .build()?;

// Or use convenience methods
let monitor = FsUsageMonitorBuilder::new()
    .watch_writes_only()  // Writes + Creates + Deletes + Moves
    .exclude_metadata()   // Skip stat/lstat operations
    .build()?;
```

### 4. Process-Specific Monitoring

```rust
use fs_usage_sys::FsUsageMonitorBuilder;

// Monitor only specific processes
let monitor = FsUsageMonitorBuilder::new()
    .watch_pid(12345)  // Specific PID
    .watch_pids([12345, 67890])  // Multiple PIDs
    .build()?;

// Or exclude specific processes
let monitor = FsUsageMonitorBuilder::new()
    .exclude_process("Spotlight")
    .exclude_pids([process::id()])  // Exclude self
    .build()?;
```

## Builder API Reference

### Path Filtering
- `watch_path(path)` - Add a glob pattern to monitor
- `watch_paths(paths)` - Add multiple glob patterns

### Process Filtering
- `watch_pid(pid)` - Monitor only specific process ID
- `watch_pids(pids)` - Monitor multiple specific PIDs
- `exclude_pid(pid)` - Exclude a specific PID
- `exclude_pids(pids)` - Exclude multiple PIDs
- `exclude_process(name)` - Exclude processes by name
- `exclude_processes(names)` - Exclude multiple processes

### Operation Type Filtering
- `watch_operations(types)` - Custom operation filtering
- `watch_writes_only()` - Only writes, creates, deletes, moves
- `watch_reads_only()` - Only read operations
- `exclude_metadata()` - Skip stat/lstat operations

## Glob Patterns

The library supports standard glob patterns:

- `*` - Match any number of characters (except `/`)
- `**` - Match any number of directories
- `?` - Match a single character
- `[abc]` - Match any character in brackets

Examples:
- `/tmp/*` - Files directly in `/tmp`
- `/tmp/**/*` - All files in `/tmp` and subdirectories
- `/Users/*/Documents/*.txt` - Text files in any user's Documents
- `/path/to/project/**/*.{rs,toml}` - Rust and TOML files

## Running Examples

All examples require sudo to access `fs_usage`:

```bash
# Basic monitoring
sudo cargo run --example basic_monitor

# Monitor with path filtering
sudo cargo run --example basic_monitor '/tmp/**/*'

# Process filtering and categorization
sudo cargo run --example process_filter '/Users/*/Documents/**/*'

# Only write operations (no reads/stats)
sudo cargo run --example writes_only '/tmp/**/*'

# Debug mode - see all parsing
sudo RUST_LOG=fs_usage_sys=debug cargo run --example debug_monitor
```

## Common Use Cases

### 1. Development File Monitoring

Monitor your project directory for changes during development:

```rust
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/path/to/project/**/*.{rs,toml,md}")
    .watch_writes_only()
    .exclude_process("target")  // Exclude build artifacts
    .build()?;
```

### 2. Security Monitoring

Detect unauthorized file modifications:

```rust
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/etc/**/*")
    .watch_path("/usr/local/bin/**/*")
    .watch_operations([OperationType::Write, OperationType::Delete])
    .build()?;
```

### 3. Backup Detection

Monitor when files are modified for backup purposes:

```rust
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/Users/*/Documents/**/*")
    .watch_writes_only()
    .exclude_processes(["Time Machine", "Spotlight"])
    .build()?;
```

## Error Handling

The library uses `anyhow` for error handling:

```rust
use anyhow::Result;

fn monitor_files() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/invalid/glob/[")  // This will fail
        .build()?;

    monitor.start()?;  // This requires sudo
    
    // Handle events with timeout
    loop {
        match monitor.events().recv_timeout(Duration::from_secs(1)) {
            Ok(event) => println!("Event: {:?}", event),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(e) => return Err(e.into()),
        }
    }
}
```

## Performance Considerations

- **Use operation filtering** to reduce noise (especially `exclude_metadata()`)
- **Specific glob patterns** perform better than monitoring everything
- **Process filtering** reduces parsing overhead
- **Batch event processing** rather than handling one-by-one

## Troubleshooting

### Permission Denied
```bash
# fs_usage requires root permissions
sudo cargo run --example basic_monitor
```

### No Events Received
- Check glob patterns are correct
- Verify paths exist
- Try without filtering first
- Check process isn't excluded

### Too Many Events
- Use `watch_writes_only()` to reduce noise
- Add process exclusions
- Use more specific glob patterns

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- Tests pass: `cargo test`
- Examples work: `sudo cargo run --example basic_monitor`
- Documentation is updated