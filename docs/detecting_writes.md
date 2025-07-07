# Detecting Write Operations from Applications

This guide explains how to identify write operations from various applications using fs_usage_sys, based on real-world patterns observed in fs_usage output.

## Important Note on fs_usage Behavior

Modern macOS fs_usage may not always show explicit `WrData` operations. Instead, write operations often manifest as patterns of `open`, `stat64`, and other filesystem calls. This guide helps you identify these patterns.

## Key Concepts

### 1. Don't Use `watch_writes_only()`

The `watch_writes_only()` method filters too aggressively and will miss many write operations. Instead, monitor all filesystem events and use pattern matching to identify writes:

```rust
// DON'T DO THIS:
let monitor = FsUsageMonitorBuilder::new()
    .watch_writes_only()  // This filters out important events!
    .build()?;

// DO THIS INSTEAD:
let monitor = FsUsageMonitorBuilder::new()
    .watch_path("/your/path/**/*")
    .exclude_process("mds")
    .exclude_process("mdworker")
    .exclude_process("Spotlight")
    .build()?;
```

### 2. Write Operation Patterns

Different applications have distinct patterns when writing files:

## Application-Specific Patterns

### 1. Vim

Vim uses a swap file pattern for safe editing:

```
# Vim creates a swap file when editing begins
22:22:36.481747 open [Helper:26258] /path/to/.file.txt.swp -> OK

# During editing, writes go to the swap file
22:22:39.390287 WrData[A] [vim:738463] /path/to/.file.txt.swp -> OK

# On save (:w), vim writes and removes the swap
22:22:39.390373 unlink [vim:738463] /path/to/.file.txt.swp -> OK
22:22:39.391679 open [Helper:26258] /path/to/file.txt -> OK
```

**Detection pattern for vim:**
```rust
fn is_vim_write(event: &FsEvent) -> bool {
    // Vim swap file creation
    if event.path.contains(".swp") && event.operation == "open" {
        return true;
    }
    
    // Vim writing to swap
    if event.process_name == "vim" && event.operation.contains("WrData") {
        return true;
    }
    
    // Vim saving (unlinking swap file)
    if event.process_name == "vim" && event.operation == "unlink" && event.path.contains(".swp") {
        return true;
    }
    
    false
}
```

### 2. Shell Redirects (echo, cat, etc.)

Shell commands like `echo "text" > file.txt` appear as simple open operations:

```
# Echo creating/writing to a file shows as open by Helper process
22:06:09.989848 open [Helper:26258] /path/to/write_test.txt -> OK

# The actual write might not show as WrData in fs_usage
# But the open by Helper process indicates a write operation
```

**Detection pattern for shell writes:**
```rust
fn is_shell_write(event: &FsEvent) -> bool {
    // Helper process opening files often indicates shell redirects
    if event.process_name == "Helper" && event.operation == "open" {
        // Could check if preceded by shell process activity
        return true;
    }
    
    // Direct shell operations
    if matches!(event.process_name.as_str(), "bash" | "zsh" | "sh") {
        if event.operation == "open" || event.operation.contains("write") {
            return true;
        }
    }
    
    false
}
```

### 3. Cursor/VS Code

Cursor and VS Code show a pattern of plugin and helper process interactions:

```
# Cursor exploring files (reads)
21:56:21.290627 open [(Plugin):555012] /path/to/file.txt 0.000012 Cursor -> OK

# When saving, Helper process opens the file
21:56:24.113083 open [Helper:26258] /path/to/file.txt 0.000029 -> OK

# Multiple processes may access during save
22:06:21.309605 open [(Plugin):26539] /path/to/file.txt 0.000008 Cursor -> OK
```

**Detection pattern for Cursor/VS Code:**
```rust
fn is_cursor_write(event: &FsEvent) -> bool {
    // Helper process after Plugin activity suggests a write
    if event.process_name == "Helper" && 
       (event.path.contains("Cursor") || previous_was_cursor_plugin) {
        return true;
    }
    
    // Look for Cursor in the extended info
    if event.result.contains("Cursor") && event.operation == "open" {
        // May indicate Cursor-initiated write
        return true;
    }
    
    false
}
```

### 4. General Application Writes

Many applications follow common patterns:

```
# stat64 calls to check file status
22:22:36.471201 stat64 [app:12345] /path/to/file.txt -> OK

# open call (may include flags in wide mode)
22:14:19.037972 open F=30 (R________S____) /path/to/file.txt -> OK

# Actual write (when visible)
22:22:39.390287 WrData[A] [app:12345] /path/to/file.txt -> OK
```

## Comprehensive Write Detection Function

Here's a complete function that combines all patterns:

```rust
use fs_usage_sys::FsEvent;

pub fn is_write_operation(event: &FsEvent, recent_events: &[FsEvent]) -> bool {
    // Direct write operations (when visible)
    if event.operation.contains("WrData") || 
       event.operation.contains("WrMeta") ||
       event.operation.contains("write") {
        return true;
    }
    
    // File creation/modification operations
    if matches!(event.operation.as_str(), 
        "creat" | "truncate" | "ftruncate" | "rename") {
        return true;
    }
    
    // Vim patterns
    if event.process_name == "vim" {
        if event.operation == "unlink" && event.path.contains(".swp") {
            return true; // Vim saving
        }
    }
    if event.path.contains(".swp") && event.operation == "open" {
        return true; // Vim swap file creation
    }
    
    // Shell redirect patterns
    if event.process_name == "Helper" && event.operation == "open" {
        // Check if this might be a shell redirect
        // You might want to look at recent events to confirm
        return true;
    }
    
    // Shell commands
    if matches!(event.process_name.as_str(), "bash" | "zsh" | "sh") {
        if event.operation == "open" {
            return true;
        }
    }
    
    // Editor backup files
    if event.path.ends_with("~") && 
       (event.operation == "open" || event.operation == "unlink") {
        return true;
    }
    
    // File deletion is a form of write
    if event.operation == "unlink" {
        return true;
    }
    
    false
}

// Enhanced detection using context
pub fn is_likely_write_with_context(
    event: &FsEvent, 
    index: usize, 
    all_events: &[FsEvent]
) -> bool {
    // Check the event itself
    if is_write_operation(event, &all_events[0..index]) {
        return true;
    }
    
    // Look for patterns in surrounding events
    if event.operation == "open" {
        // Check if followed by file size change or modification
        for i in 1..=5 {
            if index + i < all_events.len() {
                let next = &all_events[index + i];
                if next.path == event.path {
                    if next.operation.contains("write") || 
                       next.operation.contains("WrData") {
                        return true;
                    }
                }
            }
        }
        
        // Check if preceded by stat64 from same process
        if index > 0 {
            let prev = &all_events[index - 1];
            if prev.path == event.path && 
               prev.process_name == event.process_name &&
               prev.operation == "stat64" {
                // stat64 followed by open often indicates write intent
                return true;
            }
        }
    }
    
    false
}
```

## Usage Example

```rust
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder};
use std::collections::VecDeque;

fn monitor_writes() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/path/to/monitor/**/*")
        .exclude_process("mds")
        .exclude_process("mdworker")
        .exclude_process("Spotlight")
        .build()?;
    
    monitor.start()?;
    
    // Keep recent events for context
    let mut recent_events: VecDeque<FsEvent> = VecDeque::with_capacity(100);
    
    loop {
        match monitor.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                recent_events.push_back(event.clone());
                if recent_events.len() > 100 {
                    recent_events.pop_front();
                }
                
                let events_vec: Vec<_> = recent_events.iter().cloned().collect();
                let current_index = events_vec.len() - 1;
                
                if is_likely_write_with_context(&event, current_index, &events_vec) {
                    println!("WRITE DETECTED: {} wrote to {} via {}", 
                        event.process_name, 
                        event.path, 
                        event.operation
                    );
                    
                    // Handle the write event
                    handle_write_event(&event);
                }
            }
            Err(_) => continue,
        }
    }
}
```

## Tips for Accurate Write Detection

1. **Monitor patterns, not just single events** - Write operations often involve multiple system calls
2. **Track Helper process activity** - Many applications use the Helper process for file operations
3. **Watch for swap/backup files** - Editors create temporary files with patterns like `.swp`, `.tmp`, `~`
4. **Consider the process name** - Different applications have characteristic behaviors
5. **Use context** - Look at events before and after to understand the full operation
6. **Don't rely solely on WrData** - Modern macOS may not always emit these events

## Common Pitfalls

1. **Over-filtering with `watch_writes_only()`** - This will miss many legitimate write operations
2. **Ignoring open operations** - Many writes appear as simple open calls
3. **Not considering Helper process** - This process handles many file operations for GUI applications
4. **Missing editor patterns** - Editors use complex sequences of operations for safe saving

## Platform Notes

- This guide is specific to macOS fs_usage output
- Different macOS versions may show slightly different event patterns
- The `-w` flag in fs_usage means "wide format", not "writes only"
- Adding `-f diskio` to fs_usage flags can help capture more write events