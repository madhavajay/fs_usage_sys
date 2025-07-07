# Fix for icaros fs_usage_sys Integration

## Problem Summary

The fs_usage_sys library was not capturing write operations when used in icaros because:

1. **The `-w` flag means "wide format", not "writes only"** - This was a misunderstanding in the library
2. **The `watch_writes_only()` method filters too aggressively** - It only allows operations that match specific types (Write, Create, Delete, Move) but filters out "open", "stat64" and other operations that are part of write workflows
3. **WrData operations ARE being generated** but were being filtered out

## Solution

### For fs_usage_sys library:

1. The WrData parsing fix has already been applied and works correctly
2. The library now includes both `-f filesys` and `-f diskio` filters to capture all relevant events

### For icaros:

**Don't use `watch_writes_only()`**. Instead:

```rust
// Instead of this:
let builder = FsUsageMonitorBuilder::new()
    .watch_writes_only()  // <-- This filters out too much!
    .watch_path("/path/to/project/**/*")
    // ...

// Use this:
let builder = FsUsageMonitorBuilder::new()
    .watch_path("/path/to/project/**/*")
    .watch_path("/path/to/project/*")
    .exclude_process("mds")
    .exclude_process("mdworker")
    .exclude_process("Spotlight");
```

Then implement your own write detection logic:

```rust
fn is_write_operation(event: &FsEvent) -> bool {
    // Check for WrData operations
    if event.operation.contains("WrData") || 
       event.operation.contains("WrMeta") ||
       event.operation.contains("write") {
        return true;
    }
    
    // Check for file modifications
    if matches!(event.operation.as_str(), 
        "creat" | "truncate" | "ftruncate" | "unlink" | "rename") {
        return true;
    }
    
    // Check for editor swap files
    if (event.path.contains(".swp") || event.path.ends_with("~")) &&
       (event.operation == "open" || event.operation == "unlink") {
        return true;
    }
    
    false
}
```

## Test Results

With the fixes applied, the monitor now correctly captures:

```
22:22:39.390287 WrData[A] [vim:738463] /Users/madhavajay/dev/fs_usage_sys/test/.final.txt.swp -> OK
```

And other write-related operations that were previously filtered out.

## Recommendations

1. Update icaros to not use `watch_writes_only()`
2. Implement custom write detection logic based on your specific needs
3. Consider monitoring for specific patterns like:
   - WrData operations
   - Swap file creation/deletion
   - Open operations by shell processes (for redirects)
   - File truncation operations