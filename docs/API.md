# fs_usage_sys API Documentation

## Overview

The `fs_usage_sys` library provides a safe Rust wrapper around macOS's `fs_usage` command for real-time file system monitoring with advanced filtering capabilities.

## Core Types

### `FsEvent`

Represents a single file system event captured from `fs_usage`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsEvent {
    pub timestamp: String,      // Timestamp from fs_usage
    pub process_name: String,   // Name of the process that triggered the event
    pub pid: u32,              // Process ID
    pub operation: String,      // Operation type (read, write, open, etc.)
    pub path: String,          // File path involved
    pub result: String,        // "OK" or error code
}
```

**Example:**
```rust
FsEvent {
    timestamp: "23:52:52.781431".to_string(),
    process_name: "vim".to_string(),
    pid: 12345,
    operation: "write".to_string(),
    path: "/tmp/test.txt".to_string(),
    result: "OK".to_string(),
}
```

### `OperationType`

Enumeration of file system operation categories for filtering.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Read,      // File reading operations
    Write,     // File writing operations  
    Create,    // File/directory creation
    Delete,    // File/directory deletion
    Move,      // File/directory renaming/moving
    Access,    // Access checks and permissions
    Metadata,  // Metadata operations (stat, xattr)
    All,       // No filtering (default)
}
```

**Operation Mapping:**
- `Read`: `read`, `pread`, `readv`, `preadv`, `RdData`, `RdMeta`
- `Write`: `write`, `pwrite`, `writev`, `pwritev`, `WrData`, `WrMeta`, `ftruncate`
- `Create`: `open`, `creat`, `mkdir`, `mkfifo`, `mknod`, `symlink`, `link`
- `Delete`: `unlink`, `rmdir`, `remove`
- `Move`: `rename`, `renameat`
- `Access`: `access`, `faccessat`, `stat`, `stat64`, `lstat`, `lstat64`, `fstat`, `fstat64`
- `Metadata`: `getxattr`, `setxattr`, `listxattr`, `removexattr`, `getattrlist`, `setattrlist`

### `FsUsageMonitor`

The main monitoring struct that wraps the `fs_usage` process and provides event streaming.

```rust
pub struct FsUsageMonitor {
    // Internal fields are private
}
```

**Methods:**

#### `start(&mut self) -> Result<()>`
Starts the `fs_usage` monitoring process.

**Requirements:** Must be run with root/sudo privileges.

**Example:**
```rust
let mut monitor = FsUsageMonitorBuilder::new().build()?;
monitor.start()?;
```

#### `stop(&mut self) -> Result<()>`
Stops the monitoring process and cleans up resources.

```rust
monitor.stop()?;
```

#### `is_running(&self) -> bool`
Returns whether the monitor is currently active.

```rust
if monitor.is_running() {
    println!("Monitor is active");
}
```

#### `recv(&self) -> Result<FsEvent>`
Blocks until the next event is received.

```rust
while let Ok(event) = monitor.recv() {
    println!("Event: {:?}", event);
}
```

#### `try_recv(&self) -> Option<FsEvent>`
Non-blocking event retrieval.

```rust
if let Some(event) = monitor.try_recv() {
    println!("Event: {:?}", event);
}
```

#### `events(&self) -> &Receiver<FsEvent>`
Returns the underlying event receiver for advanced usage.

```rust
use std::time::Duration;

let receiver = monitor.events();
match receiver.recv_timeout(Duration::from_secs(1)) {
    Ok(event) => println!("Event: {:?}", event),
    Err(_) => println!("Timeout"),
}
```

### `FsUsageMonitorBuilder`

Builder pattern implementation for configuring monitoring parameters.

```rust
pub struct FsUsageMonitorBuilder {
    // Internal configuration
}
```

## Builder API

### Constructor

#### `new() -> Self`
Creates a new builder with default settings.

```rust
let builder = FsUsageMonitorBuilder::new();
```

### Path Filtering

#### `watch_path(self, path: impl Into<String>) -> Self`
Adds a single glob pattern to monitor.

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_path("/tmp/**/*");
```

#### `watch_paths(self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self`
Adds multiple glob patterns.

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_paths([
        "/Users/*/Documents/**/*",
        "/tmp/**/*",
        "/var/log/*.log"
    ]);
```

**Glob Pattern Examples:**
```rust
// Files directly in /tmp
.watch_path("/tmp/*")

// All files in /tmp and subdirectories  
.watch_path("/tmp/**/*")

// Text files in any user's Documents
.watch_path("/Users/*/Documents/*.txt")

// Rust files in project
.watch_path("/path/to/project/**/*.rs")

// Multiple file types
.watch_path("/path/**/*.{rs,toml,md}")
```

### Process Filtering

#### `watch_pid(self, pid: u32) -> Self`
Monitor only a specific process ID.

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_pid(12345);
```

#### `watch_pids(self, pids: impl IntoIterator<Item = u32>) -> Self`
Monitor multiple specific process IDs.

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_pids([12345, 67890]);
```

#### `exclude_pid(self, pid: u32) -> Self`
Exclude a specific process ID from monitoring.

```rust
use std::process;

let builder = FsUsageMonitorBuilder::new()
    .exclude_pid(process::id()); // Exclude self
```

#### `exclude_pids(self, pids: impl IntoIterator<Item = u32>) -> Self`
Exclude multiple process IDs.

```rust
let builder = FsUsageMonitorBuilder::new()
    .exclude_pids([12345, 67890]);
```

#### `exclude_process(self, process: impl Into<String>) -> Self`
Exclude processes by name.

```rust
let builder = FsUsageMonitorBuilder::new()
    .exclude_process("mds")
    .exclude_process("Spotlight");
```

#### `exclude_processes(self, processes: impl IntoIterator<Item = impl Into<String>>) -> Self`
Exclude multiple processes by name.

```rust
let builder = FsUsageMonitorBuilder::new()
    .exclude_processes(["mds", "mdworker", "Spotlight"]);
```

### Operation Type Filtering

#### `watch_operations(self, operations: impl IntoIterator<Item = OperationType>) -> Self`
Monitor only specific operation types.

```rust
use fs_usage_sys::OperationType;

let builder = FsUsageMonitorBuilder::new()
    .watch_operations([
        OperationType::Write,
        OperationType::Create,
        OperationType::Delete
    ]);
```

#### `watch_writes_only(self) -> Self`
Convenience method to monitor only write-related operations (writes, creates, deletes, moves).

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_writes_only();
```

Equivalent to:
```rust
.watch_operations([
    OperationType::Write,
    OperationType::Create, 
    OperationType::Delete,
    OperationType::Move
])
```

#### `watch_reads_only(self) -> Self`
Monitor only read operations.

```rust
let builder = FsUsageMonitorBuilder::new()
    .watch_reads_only();
```

#### `exclude_metadata(self) -> Self`
Exclude metadata operations (stat, lstat, xattr).

```rust
let builder = FsUsageMonitorBuilder::new()
    .exclude_metadata();
```

Equivalent to:
```rust
.watch_operations([
    OperationType::Read,
    OperationType::Write,
    OperationType::Create,
    OperationType::Delete,
    OperationType::Move
])
```

#### `build(self) -> Result<FsUsageMonitor>`
Constructs the final monitor instance.

```rust
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/tmp/**/*")
    .watch_writes_only()
    .build()?;
```

## Complete Examples

### Basic Monitoring

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use anyhow::Result;

fn basic_monitor() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp/**/*")
        .build()?;
    
    monitor.start()?;
    
    while let Ok(event) = monitor.recv() {
        println!("{}: {} {} {}", 
            event.timestamp,
            event.process_name,
            event.operation,
            event.path
        );
    }
    
    Ok(())
}
```

### Advanced Configuration

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use std::process;
use anyhow::Result;

fn advanced_monitor() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        // Path filtering
        .watch_paths([
            "/Users/*/Documents/**/*",
            "/Users/*/Desktop/**/*",
            "/tmp/**/*"
        ])
        // Operation filtering
        .watch_operations([
            OperationType::Write,
            OperationType::Create,
            OperationType::Delete
        ])
        // Process filtering
        .exclude_pid(process::id())
        .exclude_processes([
            "mds",
            "mdworker", 
            "Spotlight",
            "Time Machine"
        ])
        .build()?;
    
    monitor.start()?;
    
    while let Ok(event) = monitor.recv() {
        match classify_event(&event) {
            EventClass::AiAssistant => {
                println!("🤖 AI: {} -> {}", event.operation, event.path);
            }
            EventClass::TextEditor => {
                println!("✏️  Editor: {} -> {}", event.operation, event.path);
            }
            EventClass::System => {
                println!("🔧 System: {} -> {}", event.operation, event.path);
            }
        }
    }
    
    Ok(())
}

enum EventClass {
    AiAssistant,
    TextEditor,
    System,
}

fn classify_event(event: &fs_usage_sys::FsEvent) -> EventClass {
    let ai_processes = ["Cursor", "Code", "code", "node", "electron"];
    let editor_processes = ["vim", "nvim", "emacs", "nano", "TextEdit"];
    
    if ai_processes.iter().any(|&p| event.process_name.contains(p)) {
        EventClass::AiAssistant
    } else if editor_processes.iter().any(|&p| event.process_name.contains(p)) {
        EventClass::TextEditor
    } else {
        EventClass::System
    }
}
```

### Error Handling

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use std::time::Duration;
use anyhow::Result;

fn robust_monitor() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp/**/*")
        .watch_writes_only()
        .build()?;
    
    // Start with error handling
    if let Err(e) = monitor.start() {
        eprintln!("Failed to start monitor: {}", e);
        eprintln!("Ensure you're running with sudo privileges");
        return Err(e);
    }
    
    println!("Monitor started successfully");
    
    // Event loop with timeout
    loop {
        match monitor.events().recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                println!("Event: {} {} {}", 
                    event.process_name, 
                    event.operation, 
                    event.path
                );
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // No events in the last second, continue
                continue;
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                eprintln!("Monitor disconnected");
                break;
            }
        }
    }
    
    monitor.stop()?;
    Ok(())
}
```

## Thread Safety

- `FsEvent` is `Send + Sync`
- `FsUsageMonitor` is `Send` but not `Sync` (contains internal mutability)
- Event receiver (`Receiver<FsEvent>`) is `Send + Sync` and can be shared across threads

## Performance Notes

1. **Operation Filtering**: Use `watch_writes_only()` or `exclude_metadata()` to significantly reduce event volume
2. **Path Specificity**: More specific glob patterns reduce parsing overhead
3. **Process Filtering**: Excluding system processes reduces noise
4. **Batch Processing**: Process events in batches rather than one-by-one for better performance

## Platform Requirements

- **macOS only** - Uses the system `fs_usage` command
- **Root privileges** - `fs_usage` requires sudo/root access
- **macOS 10.5+** - `fs_usage` availability

## Common Patterns

### AI Assistant Detection

```rust
fn is_ai_assistant(process_name: &str) -> bool {
    ["Cursor", "Code", "code", "node", "electron"]
        .iter()
        .any(|&p| process_name.contains(p))
}
```

### File Change Decision Logic

```rust
fn should_accept_change(event: &FsEvent) -> bool {
    // Accept manual editor changes
    if ["vim", "nvim", "emacs", "nano"].iter().any(|&p| event.process_name.contains(p)) {
        return true;
    }
    
    // Prompt for AI changes
    if is_ai_assistant(&event.process_name) {
        return prompt_user(&format!("Accept AI change to {}?", event.path));
    }
    
    // Default: accept system changes
    true
}
```