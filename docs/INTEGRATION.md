# Integration Guide

This guide shows how to integrate `fs_usage_sys` into your applications for various use cases.

## Quick Integration

### 1. Add to Cargo.toml

```toml
[dependencies]
fs_usage_sys = "0.1.0"
anyhow = "1.0"
crossbeam-channel = "0.5"

# Optional: for async integration
tokio = { version = "1.0", features = ["full"] }
```

### 2. Basic Setup

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use anyhow::Result;

fn main() -> Result<()> {
    // Must run with sudo
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/path/to/watch/**/*")
        .watch_writes_only()
        .build()?;
    
    monitor.start()?;
    
    while let Ok(event) = monitor.recv() {
        println!("File change: {} in {}", event.operation, event.path);
    }
    
    Ok(())
}
```

## Use Case: AI Assistant File Change Detection

Perfect for applications that want to distinguish between AI-generated changes and manual edits.

### Implementation

```rust
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder, OperationType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use anyhow::Result;

pub struct AiChangeDetector {
    monitor: fs_usage_sys::FsUsageMonitor,
    project_path: PathBuf,
    pending_changes: HashMap<String, ChangeInfo>,
}

#[derive(Debug)]
pub struct ChangeInfo {
    pub source: ChangeSource,
    pub operation: String,
    pub timestamp: String,
    pub process_name: String,
    pub pid: u32,
}

#[derive(Debug, PartialEq)]
pub enum ChangeSource {
    AiAssistant,
    TextEditor,
    System,
    Unknown,
}

impl AiChangeDetector {
    pub fn new(project_path: impl Into<PathBuf>) -> Result<Self> {
        let project_path = project_path.into();
        let watch_pattern = format!("{}/**/*", project_path.display());
        
        let monitor = FsUsageMonitorBuilder::new()
            .watch_path(watch_pattern)
            .watch_writes_only()  // Only modifications
            .exclude_pid(process::id())  // Exclude self
            .exclude_processes([
                "mds", "mdworker", "Spotlight",  // System processes
                "Time Machine", "backupd",       // Backup processes
                "fsck", "diskutil"               // Disk utilities
            ])
            .build()?;
        
        Ok(Self {
            monitor,
            project_path,
            pending_changes: HashMap::new(),
        })
    }
    
    pub fn start(&mut self) -> Result<()> {
        self.monitor.start()
    }
    
    pub fn stop(&mut self) -> Result<()> {
        self.monitor.stop()
    }
    
    /// Check for new file changes, returns None if no events
    pub fn check_changes(&mut self) -> Option<ChangeInfo> {
        if let Some(event) = self.monitor.try_recv() {
            let change_info = self.classify_change(&event);
            
            // Store for potential rollback
            self.pending_changes.insert(
                event.path.clone(),
                change_info.clone()
            );
            
            Some(change_info)
        } else {
            None
        }
    }
    
    /// Block and wait for the next change
    pub fn wait_for_change(&mut self) -> Result<ChangeInfo> {
        let event = self.monitor.recv()?;
        let change_info = self.classify_change(&event);
        
        self.pending_changes.insert(
            event.path.clone(),
            change_info.clone()
        );
        
        Ok(change_info)
    }
    
    /// Get information about pending changes
    pub fn pending_changes(&self) -> &HashMap<String, ChangeInfo> {
        &self.pending_changes
    }
    
    /// Clear pending changes (e.g., after user approval)
    pub fn clear_pending(&mut self) {
        self.pending_changes.clear();
    }
    
    /// Classify the source of a file change
    fn classify_change(&self, event: &FsEvent) -> ChangeInfo {
        let source = match event.process_name.as_str() {
            // AI Assistants / IDEs with AI
            name if name.contains("Cursor") => ChangeSource::AiAssistant,
            name if name.contains("Code") => ChangeSource::AiAssistant,
            name if name.contains("code") => ChangeSource::AiAssistant,
            name if name.contains("node") => ChangeSource::AiAssistant,
            name if name.contains("electron") => ChangeSource::AiAssistant,
            
            // Text Editors
            name if name.contains("vim") => ChangeSource::TextEditor,
            name if name.contains("nvim") => ChangeSource::TextEditor,
            name if name.contains("emacs") => ChangeSource::TextEditor,
            name if name.contains("nano") => ChangeSource::TextEditor,
            name if name.contains("TextEdit") => ChangeSource::TextEditor,
            name if name.contains("Sublime") => ChangeSource::TextEditor,
            
            // System processes
            name if name.starts_with("com.apple") => ChangeSource::System,
            name if name.contains("kernel") => ChangeSource::System,
            
            _ => ChangeSource::Unknown,
        };
        
        ChangeInfo {
            source,
            operation: event.operation.clone(),
            timestamp: event.timestamp.clone(),
            process_name: event.process_name.clone(),
            pid: event.pid,
        }
    }
}

// Usage example
fn main() -> Result<()> {
    let mut detector = AiChangeDetector::new("/path/to/project")?;
    detector.start()?;
    
    println!("Monitoring for AI vs manual changes...");
    
    loop {
        match detector.wait_for_change() {
            Ok(change) => {
                match change.source {
                    ChangeSource::AiAssistant => {
                        println!("🤖 AI Assistant changed file");
                        println!("Process: {}", change.process_name);
                        println!("Operation: {}", change.operation);
                        
                        // Implement decision logic
                        if should_approve_ai_change(&change) {
                            println!("✅ Change approved");
                            detector.clear_pending();
                        } else {
                            println!("❌ Change rejected - implement rollback");
                            // Implement rollback logic here
                        }
                    }
                    ChangeSource::TextEditor => {
                        println!("✏️ Manual edit detected");
                        detector.clear_pending();
                    }
                    ChangeSource::System => {
                        println!("🔧 System change");
                        detector.clear_pending();
                    }
                    ChangeSource::Unknown => {
                        println!("❓ Unknown source: {}", change.process_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn should_approve_ai_change(change: &ChangeInfo) -> bool {
    // Implement your decision logic here
    // Examples:
    // - Always approve certain file types
    // - Prompt user for approval
    // - Approve based on time of day
    // - Check if change is in test files vs production
    
    true // Placeholder
}
```

## Use Case: Development File Watcher

Monitor project files during development and trigger actions.

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use std::path::Path;
use std::process::Command;
use anyhow::Result;

pub struct DevWatcher {
    monitor: fs_usage_sys::FsUsageMonitor,
}

impl DevWatcher {
    pub fn new(project_path: &str) -> Result<Self> {
        let patterns = [
            format!("{}/**/*.rs", project_path),      // Rust files
            format!("{}/**/*.toml", project_path),    // Config files
            format!("{}/**/*.md", project_path),      // Documentation
        ];
        
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths(patterns)
            .watch_writes_only()
            .exclude_processes(["target", "cargo", "rustc"])  // Build artifacts
            .build()?;
        
        Ok(Self { monitor })
    }
    
    pub fn start_watching(&mut self) -> Result<()> {
        self.monitor.start()?;
        
        println!("🔍 Watching for file changes...");
        
        while let Ok(event) = self.monitor.recv() {
            self.handle_file_change(&event);
        }
        
        Ok(())
    }
    
    fn handle_file_change(&self, event: &fs_usage_sys::FsEvent) {
        let path = Path::new(&event.path);
        
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => {
                println!("🦀 Rust file changed: {}", event.path);
                self.run_cargo_check();
            }
            Some("toml") => {
                println!("⚙️  Config file changed: {}", event.path);
                self.reload_config();
            }
            Some("md") => {
                println!("📝 Documentation changed: {}", event.path);
                self.regenerate_docs();
            }
            _ => {
                println!("📄 File changed: {}", event.path);
            }
        }
    }
    
    fn run_cargo_check(&self) {
        println!("Running cargo check...");
        let _ = Command::new("cargo")
            .args(["check"])
            .status();
    }
    
    fn reload_config(&self) {
        println!("Reloading configuration...");
        // Implement config reload logic
    }
    
    fn regenerate_docs(&self) {
        println!("Regenerating documentation...");
        let _ = Command::new("cargo")
            .args(["doc"])
            .status();
    }
}
```

## Use Case: Security Monitoring

Monitor sensitive directories for unauthorized changes.

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use serde_json;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Utc};
use anyhow::Result;

pub struct SecurityMonitor {
    monitor: fs_usage_sys::FsUsageMonitor,
    log_file: std::fs::File,
}

impl SecurityMonitor {
    pub fn new() -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths([
                "/etc/**/*",                    // System config
                "/usr/local/bin/**/*",          // Local binaries
                "/System/Library/**/*",         // System libraries
                "/Applications/**/*",           // Applications
            ])
            .watch_operations([
                OperationType::Write,
                OperationType::Create,
                OperationType::Delete,
                OperationType::Move,
            ])
            .exclude_processes([
                "softwareupdate",  // System updates
                "installer",       // Package installer
                "pkgd",           // Package daemon
            ])
            .build()?;
        
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/security_monitor.log")?;
        
        Ok(Self {
            monitor,
            log_file,
        })
    }
    
    pub fn start_monitoring(&mut self) -> Result<()> {
        self.monitor.start()?;
        
        println!("🛡️ Security monitoring started");
        
        while let Ok(event) = self.monitor.recv() {
            self.handle_security_event(&event)?;
        }
        
        Ok(())
    }
    
    fn handle_security_event(&mut self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        let severity = self.assess_severity(event);
        
        // Log all events
        self.log_event(event, severity)?;
        
        // Alert on high severity
        if severity >= SecurityLevel::High {
            self.send_alert(event, severity)?;
        }
        
        Ok(())
    }
    
    fn assess_severity(&self, event: &fs_usage_sys::FsEvent) -> SecurityLevel {
        // Critical system files
        if event.path.starts_with("/System/Library") ||
           event.path.starts_with("/usr/bin") ||
           event.path.starts_with("/usr/sbin") {
            return SecurityLevel::Critical;
        }
        
        // Important config files
        if event.path.starts_with("/etc") {
            return SecurityLevel::High;
        }
        
        // Applications
        if event.path.starts_with("/Applications") {
            return SecurityLevel::Medium;
        }
        
        SecurityLevel::Low
    }
    
    fn log_event(&mut self, event: &fs_usage_sys::FsEvent, severity: SecurityLevel) -> Result<()> {
        let log_entry = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "severity": format!("{:?}", severity),
            "process": event.process_name,
            "pid": event.pid,
            "operation": event.operation,
            "path": event.path,
            "fs_timestamp": event.timestamp,
        });
        
        writeln!(self.log_file, "{}", log_entry)?;
        self.log_file.flush()?;
        
        Ok(())
    }
    
    fn send_alert(&self, event: &fs_usage_sys::FsEvent, severity: SecurityLevel) -> Result<()> {
        println!("🚨 SECURITY ALERT [{:?}]", severity);
        println!("Process: {} (PID: {})", event.process_name, event.pid);
        println!("Operation: {}", event.operation);
        println!("Path: {}", event.path);
        
        // Implement actual alerting (email, Slack, etc.)
        
        Ok(())
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum SecurityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}
```

## Async Integration

For applications using async/await:

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use anyhow::Result;

pub struct AsyncFileMonitor {
    monitor: fs_usage_sys::FsUsageMonitor,
}

impl AsyncFileMonitor {
    pub fn new(watch_path: &str) -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_path(watch_path)
            .watch_writes_only()
            .build()?;
        
        Ok(Self { monitor })
    }
    
    pub async fn start_async(&mut self) -> Result<()> {
        self.monitor.start()?;
        
        let (tx, mut rx) = mpsc::unbounded_channel();
        let receiver = self.monitor.events().clone();
        
        // Spawn blocking task for event reception
        let tx_clone = tx.clone();
        tokio::task::spawn_blocking(move || {
            while let Ok(event) = receiver.recv() {
                if tx_clone.send(event).is_err() {
                    break;
                }
            }
        });
        
        // Async event processing
        while let Some(event) = rx.recv().await {
            self.handle_event_async(event).await;
        }
        
        Ok(())
    }
    
    async fn handle_event_async(&self, event: fs_usage_sys::FsEvent) {
        // Async processing of file events
        println!("Async handling: {} -> {}", event.operation, event.path);
        
        // Example: async HTTP notification
        if let Err(e) = self.notify_webhook(&event).await {
            eprintln!("Failed to send webhook: {}", e);
        }
    }
    
    async fn notify_webhook(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "event_type": "file_change",
            "process": event.process_name,
            "operation": event.operation,
            "path": event.path,
            "timestamp": event.timestamp,
        });
        
        let _response = timeout(
            Duration::from_secs(5),
            client.post("https://your-webhook-url.com/events")
                .json(&payload)
                .send()
        ).await??;
        
        Ok(())
    }
}
```

## Testing Integration

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_detection() -> Result<()> {
        // Note: This test requires sudo to run
        let temp_dir = TempDir::new()?;
        let watch_path = format!("{}/**/*", temp_dir.path().display());
        
        let mut monitor = FsUsageMonitorBuilder::new()
            .watch_path(watch_path)
            .watch_writes_only()
            .build()?;
        
        monitor.start()?;
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;
        
        // Should receive an event
        let event = monitor.recv()?;
        assert!(event.path.contains("test.txt"));
        assert!(event.operation.contains("write") || event.operation.contains("open"));
        
        monitor.stop()?;
        Ok(())
    }
}
```

## Error Handling Best Practices

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use anyhow::{Context, Result};
use std::time::Duration;

fn robust_monitoring() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp/**/*")
        .build()
        .context("Failed to build monitor")?;
    
    // Check if we have the required permissions
    if let Err(e) = monitor.start() {
        if e.to_string().contains("Permission denied") {
            eprintln!("❌ Permission denied. Please run with sudo:");
            eprintln!("   sudo cargo run");
            return Err(e);
        }
        return Err(e).context("Failed to start monitor");
    }
    
    println!("✅ Monitor started successfully");
    
    // Graceful shutdown handling
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\n📡 Shutting down gracefully...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;
    
    // Event loop with timeout and error recovery
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match monitor.events().recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                if let Err(e) = process_event(&event) {
                    eprintln!("⚠️ Error processing event: {}", e);
                    // Continue processing other events
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue
                continue;
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                eprintln!("📡 Monitor disconnected");
                break;
            }
        }
    }
    
    monitor.stop().context("Failed to stop monitor")?;
    println!("✅ Monitor stopped successfully");
    
    Ok(())
}

fn process_event(event: &fs_usage_sys::FsEvent) -> Result<()> {
    // Your event processing logic here
    println!("Processing: {} {} {}", 
        event.process_name, 
        event.operation, 
        event.path
    );
    Ok(())
}
```

## Performance Tips

1. **Use specific glob patterns** instead of monitoring everything
2. **Filter operations** - use `watch_writes_only()` to reduce noise
3. **Exclude system processes** to reduce parsing overhead
4. **Batch process events** rather than handling individually
5. **Use timeouts** to prevent blocking indefinitely

## Common Gotchas

1. **Requires sudo** - The underlying `fs_usage` command needs root privileges
2. **Path normalization** - `/private/tmp` gets normalized to `/tmp`
3. **Process name variations** - Same app might have different process names
4. **Event flooding** - Without filtering, you'll get thousands of events per second
5. **macOS only** - This library only works on macOS systems