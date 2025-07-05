# Examples and Use Cases

This document provides practical examples of using `fs_usage_sys` for various monitoring scenarios.

## Table of Contents

1. [Basic Monitoring](#basic-monitoring)
2. [AI Assistant Detection](#ai-assistant-detection)
3. [Development Workflow](#development-workflow)
4. [Security Monitoring](#security-monitoring)
5. [File Backup Triggers](#file-backup-triggers)
6. [Performance Monitoring](#performance-monitoring)
7. [Real-time Notifications](#real-time-notifications)

## Basic Monitoring

### Simple File Watcher

Monitor a directory for any file changes:

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use anyhow::Result;

fn main() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/Users/john/Documents/**/*")
        .build()?;
    
    monitor.start()?;
    println!("👀 Watching Documents folder...");
    
    while let Ok(event) = monitor.recv() {
        println!("📄 {} {} by {} (PID: {})",
            event.operation,
            event.path,
            event.process_name,
            event.pid
        );
    }
    
    Ok(())
}
```

### Write-Only Monitor

Only monitor file modifications (ignore reads):

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use anyhow::Result;

fn main() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/tmp/**/*")
        .watch_writes_only()  // Only writes, creates, deletes, moves
        .build()?;
    
    monitor.start()?;
    println!("✏️ Monitoring write operations only...");
    
    while let Ok(event) = monitor.recv() {
        let emoji = match event.operation.as_str() {
            op if op.contains("write") => "✏️",
            op if op.contains("open") => "📄",
            op if op.contains("unlink") => "🗑️",
            op if op.contains("rename") => "📁",
            _ => "💾",
        };
        
        println!("{} {} -> {}", emoji, event.operation, event.path);
    }
    
    Ok(())
}
```

## AI Assistant Detection

### Claude vs Manual Edit Detection

Distinguish between AI assistant changes and manual text editor changes:

```rust
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder};
use std::io::{self, Write};
use anyhow::Result;

#[derive(Debug)]
enum ChangeSource {
    AiAssistant(String),  // Process name
    TextEditor(String),   // Process name
    System(String),       // Process name
    Unknown(String),      // Process name
}

fn main() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/path/to/project/**/*.{rs,py,js,ts,md}")
        .watch_writes_only()
        .exclude_processes(["mds", "mdworker", "Spotlight"])
        .build()?;
    
    monitor.start()?;
    println!("🤖 AI vs 👤 Human change detection active");
    println!("Monitoring: Rust, Python, JavaScript, TypeScript, Markdown files");
    
    while let Ok(event) = monitor.recv() {
        let source = classify_change_source(&event);
        handle_change(&event, source)?;
    }
    
    Ok(())
}

fn classify_change_source(event: &FsEvent) -> ChangeSource {
    let process = &event.process_name;
    
    // AI Assistants and IDEs with AI capabilities
    if ["Cursor", "cursor"].iter().any(|&p| process.contains(p)) {
        return ChangeSource::AiAssistant("Cursor".to_string());
    }
    if ["Code", "code"].iter().any(|&p| process.contains(p)) {
        return ChangeSource::AiAssistant("VS Code".to_string());
    }
    if process.contains("node") && process.contains("electron") {
        return ChangeSource::AiAssistant("Electron App".to_string());
    }
    
    // Text Editors
    if ["vim", "nvim"].iter().any(|&p| process.contains(p)) {
        return ChangeSource::TextEditor("Vim".to_string());
    }
    if process.contains("emacs") {
        return ChangeSource::TextEditor("Emacs".to_string());
    }
    if ["nano", "TextEdit", "Sublime"].iter().any(|&p| process.contains(p)) {
        return ChangeSource::TextEditor(process.clone());
    }
    
    // System processes
    if process.starts_with("com.apple") || process.contains("kernel") {
        return ChangeSource::System(process.clone());
    }
    
    ChangeSource::Unknown(process.clone())
}

fn handle_change(event: &FsEvent, source: ChangeSource) -> Result<()> {
    match source {
        ChangeSource::AiAssistant(ai_name) => {
            println!("\n🤖 AI ASSISTANT CHANGE DETECTED");
            println!("   AI: {}", ai_name);
            println!("   File: {}", event.path);
            println!("   Operation: {}", event.operation);
            println!("   Time: {}", event.timestamp);
            
            // Prompt user for decision
            print!("   Accept this change? (y/n/always/never): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    println!("   ✅ Change accepted");
                }
                "n" | "no" => {
                    println!("   ❌ Change rejected");
                    // Implement rollback logic here
                    rollback_change(&event.path)?;
                }
                "always" => {
                    println!("   ✅ Auto-accepting future AI changes");
                    // Set persistent preference
                }
                "never" => {
                    println!("   ❌ Auto-rejecting future AI changes");
                    // Set persistent preference
                }
                _ => {
                    println!("   ❓ Unknown response, skipping...");
                }
            }
        }
        
        ChangeSource::TextEditor(editor) => {
            println!("✏️ Manual edit: {} in {} ({})", 
                event.operation, 
                event.path, 
                editor
            );
        }
        
        ChangeSource::System(process) => {
            println!("🔧 System change: {} by {}", event.path, process);
        }
        
        ChangeSource::Unknown(process) => {
            println!("❓ Unknown source: {} changed {} ({})", 
                process, 
                event.path, 
                event.operation
            );
        }
    }
    
    Ok(())
}

fn rollback_change(file_path: &str) -> Result<()> {
    println!("🔄 Rolling back changes to: {}", file_path);
    
    // Implementation ideas:
    // 1. Restore from git: git checkout HEAD -- file_path
    // 2. Restore from backup copy
    // 3. Show diff and let user manually revert
    
    std::process::Command::new("git")
        .args(["checkout", "HEAD", "--", file_path])
        .status()?;
    
    println!("✅ File restored from git");
    Ok(())
}
```

### Advanced AI Detection with Learning

```rust
use fs_usage_sys::{FsEvent, FsUsageMonitorBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
struct ProcessProfile {
    name: String,
    classification: ProcessClass,
    confidence: f32,
    sample_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ProcessClass {
    AiAssistant,
    TextEditor,
    BuildTool,
    System,
    Unknown,
}

struct SmartChangeDetector {
    monitor: fs_usage_sys::FsUsageMonitor,
    profiles: HashMap<String, ProcessProfile>,
    config_path: String,
}

impl SmartChangeDetector {
    fn new(watch_path: &str, config_path: &str) -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_path(watch_path)
            .watch_writes_only()
            .build()?;
        
        let profiles = Self::load_profiles(config_path)?;
        
        Ok(Self {
            monitor,
            profiles,
            config_path: config_path.to_string(),
        })
    }
    
    fn load_profiles(config_path: &str) -> Result<HashMap<String, ProcessProfile>> {
        if let Ok(data) = fs::read_to_string(config_path) {
            Ok(serde_json::from_str(&data)?)
        } else {
            // Initialize with known patterns
            let mut profiles = HashMap::new();
            
            // Known AI assistants
            profiles.insert("Cursor".to_string(), ProcessProfile {
                name: "Cursor".to_string(),
                classification: ProcessClass::AiAssistant,
                confidence: 0.95,
                sample_count: 100,
            });
            
            // Known text editors
            profiles.insert("vim".to_string(), ProcessProfile {
                name: "vim".to_string(),
                classification: ProcessClass::TextEditor,
                confidence: 0.99,
                sample_count: 1000,
            });
            
            Ok(profiles)
        }
    }
    
    fn save_profiles(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.profiles)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }
    
    fn classify_process(&mut self, process_name: &str) -> ProcessClass {
        // Check exact match first
        if let Some(profile) = self.profiles.get(process_name) {
            if profile.confidence > 0.8 {
                return profile.classification.clone();
            }
        }
        
        // Check partial matches
        for (key, profile) in &self.profiles {
            if process_name.contains(key) && profile.confidence > 0.7 {
                return profile.classification.clone();
            }
        }
        
        // Learn new process
        self.learn_new_process(process_name);
        ProcessClass::Unknown
    }
    
    fn learn_new_process(&mut self, process_name: &str) {
        // Simple heuristics for initial classification
        let classification = if process_name.contains("code") || 
                               process_name.contains("cursor") ||
                               process_name.contains("copilot") {
            ProcessClass::AiAssistant
        } else if process_name.contains("vim") || 
                  process_name.contains("emacs") || 
                  process_name.contains("nano") {
            ProcessClass::TextEditor
        } else if process_name.contains("cargo") || 
                  process_name.contains("rustc") || 
                  process_name.contains("npm") {
            ProcessClass::BuildTool
        } else {
            ProcessClass::Unknown
        };
        
        self.profiles.insert(process_name.to_string(), ProcessProfile {
            name: process_name.to_string(),
            classification,
            confidence: 0.5,
            sample_count: 1,
        });
    }
    
    fn update_confidence(&mut self, process_name: &str, correct: bool) {
        if let Some(profile) = self.profiles.get_mut(process_name) {
            profile.sample_count += 1;
            
            if correct {
                profile.confidence = (profile.confidence * 0.9 + 0.1).min(0.99);
            } else {
                profile.confidence = (profile.confidence * 0.9).max(0.1);
            }
        }
    }
    
    fn run(&mut self) -> Result<()> {
        self.monitor.start()?;
        
        while let Ok(event) = self.monitor.recv() {
            let classification = self.classify_process(&event.process_name);
            self.handle_classified_event(&event, classification)?;
        }
        
        Ok(())
    }
    
    fn handle_classified_event(&self, event: &FsEvent, class: ProcessClass) -> Result<()> {
        match class {
            ProcessClass::AiAssistant => {
                println!("🤖 AI: {} modified {}", event.process_name, event.path);
                // Implement AI-specific handling
            }
            ProcessClass::TextEditor => {
                println!("✏️ Editor: {} modified {}", event.process_name, event.path);
            }
            ProcessClass::BuildTool => {
                // Usually ignore build tools
                return Ok(());
            }
            ProcessClass::System => {
                println!("🔧 System: {} modified {}", event.process_name, event.path);
            }
            ProcessClass::Unknown => {
                println!("❓ Unknown: {} modified {} (learning...)", 
                    event.process_name, event.path);
            }
        }
        
        Ok(())
    }
}
```

## Development Workflow

### Automated Testing on File Changes

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use std::process::Command;
use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::Result;

struct DevTestRunner {
    monitor: fs_usage_sys::FsUsageMonitor,
    last_test_run: Instant,
    test_debounce: Duration,
}

impl DevTestRunner {
    fn new(project_path: &str) -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths([
                format!("{}/**/*.rs", project_path),
                format!("{}/**/*.toml", project_path),
            ])
            .watch_writes_only()
            .exclude_processes([
                "target",    // Build artifacts
                "cargo",     // Cargo itself
                "rustc",     // Rust compiler
                "rust-analyzer", // LSP
            ])
            .build()?;
        
        Ok(Self {
            monitor,
            last_test_run: Instant::now() - Duration::from_secs(60),
            test_debounce: Duration::from_secs(2),
        })
    }
    
    fn start(&mut self) -> Result<()> {
        self.monitor.start()?;
        println!("🔄 Auto-test runner started");
        
        while let Ok(event) = self.monitor.recv() {
            if self.should_run_tests(&event) {
                self.run_tests(&event.path)?;
            }
        }
        
        Ok(())
    }
    
    fn should_run_tests(&mut self, event: &fs_usage_sys::FsEvent) -> bool {
        // Debounce rapid file changes
        let now = Instant::now();
        if now.duration_since(self.last_test_run) < self.test_debounce {
            return false;
        }
        
        let path = Path::new(&event.path);
        
        // Only test for Rust files and Cargo.toml
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") | Some("toml") => {
                self.last_test_run = now;
                true
            }
            _ => false,
        }
    }
    
    fn run_tests(&self, changed_file: &str) -> Result<()> {
        println!("\n📝 File changed: {}", changed_file);
        println!("🧪 Running tests...");
        
        let start = Instant::now();
        
        // Run cargo check first (faster)
        let check_result = Command::new("cargo")
            .args(["check", "--quiet"])
            .status()?;
        
        if !check_result.success() {
            println!("❌ Cargo check failed");
            return Ok(());
        }
        
        // Run tests
        let test_result = Command::new("cargo")
            .args(["test", "--quiet"])
            .status()?;
        
        let duration = start.elapsed();
        
        if test_result.success() {
            println!("✅ All tests passed ({:.1}s)", duration.as_secs_f32());
        } else {
            println!("❌ Tests failed ({:.1}s)", duration.as_secs_f32());
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut runner = DevTestRunner::new("/path/to/project")?;
    runner.start()
}
```

### Live Documentation Generation

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use std::process::Command;
use std::path::Path;
use anyhow::Result;

struct DocGenerator {
    monitor: fs_usage_sys::FsUsageMonitor,
}

impl DocGenerator {
    fn new(project_path: &str) -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths([
                format!("{}/**/*.rs", project_path),
                format!("{}/**/*.md", project_path),
                format!("{}/Cargo.toml", project_path),
            ])
            .watch_writes_only()
            .build()?;
        
        Ok(Self { monitor })
    }
    
    fn start(&mut self) -> Result<()> {
        self.monitor.start()?;
        println!("📚 Live documentation generator started");
        
        while let Ok(event) = self.monitor.recv() {
            self.handle_doc_change(&event)?;
        }
        
        Ok(())
    }
    
    fn handle_doc_change(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        let path = Path::new(&event.path);
        
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => {
                println!("🦀 Rust file changed: {}", event.path);
                self.generate_api_docs()?;
            }
            Some("md") => {
                println!("📝 Markdown changed: {}", event.path);
                self.build_book()?;
            }
            Some("toml") if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) => {
                println!("⚙️ Cargo.toml changed");
                self.generate_api_docs()?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn generate_api_docs(&self) -> Result<()> {
        println!("📖 Generating API documentation...");
        
        let result = Command::new("cargo")
            .args(["doc", "--no-deps", "--open"])
            .status()?;
        
        if result.success() {
            println!("✅ API docs updated");
        } else {
            println!("❌ Failed to generate docs");
        }
        
        Ok(())
    }
    
    fn build_book(&self) -> Result<()> {
        println!("📚 Building mdBook...");
        
        let result = Command::new("mdbook")
            .args(["build"])
            .status()?;
        
        if result.success() {
            println!("✅ Book built successfully");
        } else {
            println!("❌ Failed to build book");
        }
        
        Ok(())
    }
}
```

## Security Monitoring

### Sensitive File Protection

```rust
use fs_usage_sys::{FsUsageMonitorBuilder, OperationType};
use serde_json;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use anyhow::Result;

struct SecurityMonitor {
    monitor: fs_usage_sys::FsUsageMonitor,
    sensitive_paths: HashSet<String>,
    authorized_processes: HashSet<String>,
}

impl SecurityMonitor {
    fn new() -> Result<Self> {
        let mut sensitive_paths = HashSet::new();
        sensitive_paths.insert("/etc/passwd".to_string());
        sensitive_paths.insert("/etc/shadow".to_string());
        sensitive_paths.insert("/etc/sudoers".to_string());
        sensitive_paths.insert("/etc/hosts".to_string());
        sensitive_paths.insert("/usr/local/bin".to_string());
        
        let mut authorized_processes = HashSet::new();
        authorized_processes.insert("sudo".to_string());
        authorized_processes.insert("dscl".to_string());
        authorized_processes.insert("installer".to_string());
        
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths([
                "/etc/**/*",
                "/usr/local/bin/**/*",
                "/System/Library/**/*",
                "/private/etc/**/*",
            ])
            .watch_operations([
                OperationType::Write,
                OperationType::Create,
                OperationType::Delete,
                OperationType::Move,
            ])
            .build()?;
        
        Ok(Self {
            monitor,
            sensitive_paths,
            authorized_processes,
        })
    }
    
    fn start_monitoring(&mut self) -> Result<()> {
        self.monitor.start()?;
        println!("🛡️ Security monitoring active");
        
        while let Ok(event) = self.monitor.recv() {
            self.analyze_security_event(&event)?;
        }
        
        Ok(())
    }
    
    fn analyze_security_event(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        let risk_level = self.assess_risk(event);
        
        match risk_level {
            RiskLevel::Critical => {
                self.handle_critical_event(event)?;
            }
            RiskLevel::High => {
                self.handle_high_risk_event(event)?;
            }
            RiskLevel::Medium => {
                self.log_medium_risk_event(event)?;
            }
            RiskLevel::Low => {
                // Log to detailed audit trail only
            }
        }
        
        Ok(())
    }
    
    fn assess_risk(&self, event: &fs_usage_sys::FsEvent) -> RiskLevel {
        // Critical: Unauthorized access to sensitive files
        if self.is_sensitive_file(&event.path) && 
           !self.is_authorized_process(&event.process_name) {
            return RiskLevel::Critical;
        }
        
        // High: System directory modifications
        if event.path.starts_with("/System/Library") ||
           event.path.starts_with("/usr/bin") ||
           event.path.starts_with("/usr/sbin") {
            return RiskLevel::High;
        }
        
        // Medium: Configuration changes
        if event.path.starts_with("/etc") {
            return RiskLevel::Medium;
        }
        
        RiskLevel::Low
    }
    
    fn is_sensitive_file(&self, path: &str) -> bool {
        self.sensitive_paths.iter().any(|sensitive| path.contains(sensitive))
    }
    
    fn is_authorized_process(&self, process: &str) -> bool {
        self.authorized_processes.iter().any(|auth| process.contains(auth))
    }
    
    fn handle_critical_event(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        println!("🚨 CRITICAL SECURITY ALERT");
        println!("   Process: {} (PID: {})", event.process_name, event.pid);
        println!("   Operation: {}", event.operation);
        println!("   File: {}", event.path);
        println!("   Time: {}", event.timestamp);
        
        // Send immediate alert
        self.send_security_alert(event, "CRITICAL")?;
        
        // Log to security audit trail
        self.log_security_event(event, "CRITICAL")?;
        
        Ok(())
    }
    
    fn handle_high_risk_event(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        println!("⚠️ High Risk Security Event");
        println!("   {} modified {} via {}", 
            event.process_name, 
            event.path, 
            event.operation
        );
        
        self.send_security_alert(event, "HIGH")?;
        self.log_security_event(event, "HIGH")?;
        
        Ok(())
    }
    
    fn log_medium_risk_event(&self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        println!("ℹ️ Medium Risk: {} -> {}", event.operation, event.path);
        self.log_security_event(event, "MEDIUM")?;
        Ok(())
    }
    
    fn send_security_alert(&self, event: &fs_usage_sys::FsEvent, level: &str) -> Result<()> {
        // Implementation would send email, Slack, webhook, etc.
        println!("📧 Security alert sent for {} event", level);
        Ok(())
    }
    
    fn log_security_event(&self, event: &fs_usage_sys::FsEvent, level: &str) -> Result<()> {
        let log_entry = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "level": level,
            "process": event.process_name,
            "pid": event.pid,
            "operation": event.operation,
            "path": event.path,
            "fs_timestamp": event.timestamp,
        });
        
        // Write to security log file
        println!("📄 Logged: {}", log_entry);
        Ok(())
    }
}

#[derive(Debug)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
```

## File Backup Triggers

### Intelligent Backup System

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::path::Path;
use std::process::Command;
use anyhow::Result;

struct SmartBackup {
    monitor: fs_usage_sys::FsUsageMonitor,
    backup_queue: HashMap<String, Instant>,
    backup_delay: Duration,
    important_extensions: Vec<String>,
}

impl SmartBackup {
    fn new(watch_paths: &[&str]) -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            .watch_paths(watch_paths.iter().copied())
            .watch_writes_only()
            .exclude_processes([
                "Time Machine",
                "backupd", 
                "Carbon Copy Cloner",
                "rsync",
            ])
            .build()?;
        
        let important_extensions = vec![
            "rs", "py", "js", "ts", "go", "cpp", "h",  // Code
            "md", "txt", "doc", "docx",                // Documents
            "json", "yaml", "toml", "xml",             // Config
            "sql", "db",                               // Database
        ].into_iter().map(String::from).collect();
        
        Ok(Self {
            monitor,
            backup_queue: HashMap::new(),
            backup_delay: Duration::from_secs(30), // Wait 30s before backing up
            important_extensions,
        })
    }
    
    fn start(&mut self) -> Result<()> {
        self.monitor.start()?;
        println!("💾 Smart backup system started");
        
        while let Ok(event) = self.monitor.recv() {
            self.handle_file_change(&event)?;
            self.process_backup_queue()?;
        }
        
        Ok(())
    }
    
    fn handle_file_change(&mut self, event: &fs_usage_sys::FsEvent) -> Result<()> {
        let path = &event.path;
        
        if self.should_backup(path) {
            println!("📝 Queuing for backup: {}", path);
            self.backup_queue.insert(path.clone(), Instant::now());
        }
        
        Ok(())
    }
    
    fn should_backup(&self, path: &str) -> bool {
        let path_obj = Path::new(path);
        
        // Check file extension
        if let Some(ext) = path_obj.extension().and_then(|s| s.to_str()) {
            if self.important_extensions.contains(&ext.to_lowercase()) {
                return true;
            }
        }
        
        // Check for specific important files
        if let Some(filename) = path_obj.file_name().and_then(|s| s.to_str()) {
            if ["Cargo.toml", "package.json", ".env", "config.yaml"].contains(&filename) {
                return true;
            }
        }
        
        false
    }
    
    fn process_backup_queue(&mut self) -> Result<()> {
        let now = Instant::now();
        let mut to_backup = Vec::new();
        
        // Find files ready for backup
        self.backup_queue.retain(|path, queued_time| {
            if now.duration_since(*queued_time) >= self.backup_delay {
                to_backup.push(path.clone());
                false // Remove from queue
            } else {
                true // Keep in queue
            }
        });
        
        // Perform backups
        for path in to_backup {
            self.backup_file(&path)?;
        }
        
        Ok(())
    }
    
    fn backup_file(&self, path: &str) -> Result<()> {
        println!("💾 Backing up: {}", path);
        
        // Create backup directory if it doesn't exist
        let backup_dir = "/Users/backup/smart_backup";
        std::fs::create_dir_all(backup_dir)?;
        
        // Generate backup filename with timestamp
        let path_obj = Path::new(path);
        let filename = path_obj.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.bak", filename, timestamp);
        let backup_path = Path::new(backup_dir).join(backup_filename);
        
        // Copy file to backup location
        match std::fs::copy(path, &backup_path) {
            Ok(_) => println!("✅ Backed up to: {}", backup_path.display()),
            Err(e) => println!("❌ Backup failed: {}", e),
        }
        
        // Optional: Compress old backups
        self.cleanup_old_backups(backup_dir)?;
        
        Ok(())
    }
    
    fn cleanup_old_backups(&self, backup_dir: &str) -> Result<()> {
        // Keep only last 10 backups per file
        // Implementation would scan backup directory and remove old files
        Ok(())
    }
}

fn main() -> Result<()> {
    let watch_paths = [
        "/Users/*/Documents/**/*",
        "/Users/*/Projects/**/*",
        "/Users/*/Code/**/*",
    ];
    
    let mut backup_system = SmartBackup::new(&watch_paths)?;
    backup_system.start()
}
```

## Performance Monitoring

### File I/O Performance Tracker

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;

struct PerformanceMonitor {
    monitor: fs_usage_sys::FsUsageMonitor,
    process_stats: HashMap<String, ProcessStats>,
    start_time: Instant,
}

#[derive(Debug, Default)]
struct ProcessStats {
    read_operations: u64,
    write_operations: u64,
    total_operations: u64,
    first_seen: Option<Instant>,
    last_seen: Option<Instant>,
}

impl PerformanceMonitor {
    fn new() -> Result<Self> {
        let monitor = FsUsageMonitorBuilder::new()
            // Monitor everything to get performance overview
            .exclude_processes([
                "kernel_task",
                "mds",
                "mdworker",
            ])
            .build()?;
        
        Ok(Self {
            monitor,
            process_stats: HashMap::new(),
            start_time: Instant::now(),
        })
    }
    
    fn start(&mut self) -> Result<()> {
        self.monitor.start()?;
        println!("📊 Performance monitoring started");
        
        // Print stats every 30 seconds
        let mut last_stats_print = Instant::now();
        let stats_interval = Duration::from_secs(30);
        
        while let Ok(event) = self.monitor.recv() {
            self.record_operation(&event);
            
            // Print periodic stats
            let now = Instant::now();
            if now.duration_since(last_stats_print) >= stats_interval {
                self.print_stats();
                last_stats_print = now;
            }
        }
        
        Ok(())
    }
    
    fn record_operation(&mut self, event: &fs_usage_sys::FsEvent) {
        let now = Instant::now();
        let stats = self.process_stats.entry(event.process_name.clone())
            .or_default();
        
        stats.total_operations += 1;
        
        // Categorize operation
        if event.operation.contains("read") || event.operation.contains("RdData") {
            stats.read_operations += 1;
        } else if event.operation.contains("write") || event.operation.contains("WrData") {
            stats.write_operations += 1;
        }
        
        // Track timing
        if stats.first_seen.is_none() {
            stats.first_seen = Some(now);
        }
        stats.last_seen = Some(now);
    }
    
    fn print_stats(&self) {
        println!("\n📊 === FILE I/O PERFORMANCE STATS ===");
        println!("Runtime: {:.1}s", self.start_time.elapsed().as_secs_f32());
        
        // Sort processes by total operations
        let mut sorted_processes: Vec<_> = self.process_stats.iter().collect();
        sorted_processes.sort_by(|a, b| b.1.total_operations.cmp(&a.1.total_operations));
        
        println!("Top 10 Most Active Processes:");
        println!("{:<20} {:>8} {:>8} {:>8} {:>8}", 
            "Process", "Total", "Reads", "Writes", "Ops/sec");
        println!("{}", "-".repeat(60));
        
        for (process_name, stats) in sorted_processes.iter().take(10) {
            let runtime = stats.first_seen
                .map(|start| self.start_time.elapsed().as_secs_f32())
                .unwrap_or(1.0);
            
            let ops_per_sec = stats.total_operations as f32 / runtime;
            
            println!("{:<20} {:>8} {:>8} {:>8} {:>8.1}", 
                Self::truncate_process_name(process_name, 20),
                stats.total_operations,
                stats.read_operations,
                stats.write_operations,
                ops_per_sec
            );
        }
        
        // Summary stats
        let total_ops: u64 = self.process_stats.values()
            .map(|s| s.total_operations)
            .sum();
        let total_reads: u64 = self.process_stats.values()
            .map(|s| s.read_operations)
            .sum();
        let total_writes: u64 = self.process_stats.values()
            .map(|s| s.write_operations)
            .sum();
        
        println!("\n📈 System Totals:");
        println!("   Total Operations: {}", total_ops);
        println!("   Read Operations:  {}", total_reads);
        println!("   Write Operations: {}", total_writes);
        println!("   Active Processes: {}", self.process_stats.len());
        
        // Identify potential performance issues
        self.analyze_performance_issues();
    }
    
    fn analyze_performance_issues(&self) {
        println!("\n🔍 Performance Analysis:");
        
        // Find processes with very high I/O
        for (process_name, stats) in &self.process_stats {
            let runtime = stats.first_seen
                .map(|start| start.elapsed().as_secs_f32())
                .unwrap_or(1.0);
            
            let ops_per_sec = stats.total_operations as f32 / runtime;
            
            if ops_per_sec > 100.0 {
                println!("⚠️ High I/O: {} ({:.1} ops/sec)", process_name, ops_per_sec);
            }
            
            // Check for read/write imbalance
            if stats.read_operations > 0 && stats.write_operations > 0 {
                let ratio = stats.read_operations as f32 / stats.write_operations as f32;
                if ratio > 10.0 {
                    println!("📖 Read-heavy: {} ({}:1 read/write ratio)", 
                        process_name, ratio as u32);
                } else if ratio < 0.1 {
                    println!("✏️ Write-heavy: {} (1:{} read/write ratio)", 
                        process_name, (1.0 / ratio) as u32);
                }
            }
        }
    }
    
    fn truncate_process_name(name: &str, max_len: usize) -> String {
        if name.len() <= max_len {
            name.to_string()
        } else {
            format!("{}...", &name[..max_len-3])
        }
    }
}

fn main() -> Result<()> {
    let mut monitor = PerformanceMonitor::new()?;
    
    println!("Starting performance monitoring...");
    println!("Press Ctrl+C to stop and see final stats");
    
    monitor.start()
}
```

## Real-time Notifications

### Webhook Integration

```rust
use fs_usage_sys::FsUsageMonitorBuilder;
use serde_json;
use reqwest;
use tokio;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut monitor = FsUsageMonitorBuilder::new()
        .watch_path("/path/to/important/files/**/*")
        .watch_writes_only()
        .build()?;
    
    monitor.start()?;
    println!("📡 Real-time notifications active");
    
    while let Ok(event) = monitor.recv() {
        if let Err(e) = send_notification(&event).await {
            eprintln!("Failed to send notification: {}", e);
        }
    }
    
    Ok(())
}

async fn send_notification(event: &fs_usage_sys::FsEvent) -> Result<()> {
    let webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL";
    
    let payload = serde_json::json!({
        "text": format!("📁 File Change Alert"),
        "attachments": [{
            "color": "warning",
            "fields": [
                {
                    "title": "File",
                    "value": event.path,
                    "short": false
                },
                {
                    "title": "Operation",
                    "value": event.operation,
                    "short": true
                },
                {
                    "title": "Process",
                    "value": format!("{} ({})", event.process_name, event.pid),
                    "short": true
                },
                {
                    "title": "Time",
                    "value": event.timestamp,
                    "short": true
                }
            ]
        }]
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Notification sent");
    } else {
        println!("❌ Notification failed: {}", response.status());
    }
    
    Ok(())
}
```

These examples demonstrate the versatility of `fs_usage_sys` for various monitoring and automation scenarios. Each can be adapted to specific requirements and integrated into larger applications.