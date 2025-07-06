#![cfg(target_os = "macos")]

mod builder;

pub use builder::FsUsageMonitorBuilder;

use anyhow::{Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsEvent {
    pub timestamp: String,
    pub process_name: String,
    pub pid: u32,
    pub operation: String,
    pub path: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Read,
    Write,
    Create,
    Delete,
    Move,
    Access,
    Metadata,
    All,
}

impl OperationType {
    pub fn matches_operation(&self, operation: &str) -> bool {
        match self {
            OperationType::All => true,
            OperationType::Read => matches!(
                operation,
                "read" | "pread" | "readv" | "preadv" | "RdData" | "RdMeta"
            ),
            OperationType::Write => matches!(
                operation,
                "write" | "pwrite" | "writev" | "pwritev" | "WrData" | "WrMeta" | "ftruncate"
            ),
            OperationType::Create => matches!(
                operation,
                "open" | "creat" | "mkdir" | "mkfifo" | "mknod" | "symlink" | "link"
            ),
            OperationType::Delete => matches!(operation, "unlink" | "rmdir" | "remove"),
            OperationType::Move => matches!(operation, "rename" | "renameat"),
            OperationType::Access => matches!(
                operation,
                "access"
                    | "faccessat"
                    | "stat"
                    | "stat64"
                    | "lstat"
                    | "lstat64"
                    | "fstat"
                    | "fstat64"
            ),
            OperationType::Metadata => matches!(
                operation,
                "stat"
                    | "stat64"
                    | "lstat"
                    | "lstat64"
                    | "fstat"
                    | "fstat64"
                    | "getxattr"
                    | "setxattr"
                    | "listxattr"
                    | "removexattr"
                    | "getattrlist"
                    | "setattrlist"
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsUsageConfig {
    pub watch_paths: Vec<String>,
    pub watch_pids: Vec<u32>,
    pub exclude_pids: Vec<u32>,
    pub exclude_processes: Vec<String>,
    pub operation_types: Vec<OperationType>,
}

impl Default for FsUsageConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            watch_pids: vec![],
            exclude_pids: vec![],
            exclude_processes: vec![],
            operation_types: vec![OperationType::All],
        }
    }
}

pub struct FsUsageMonitor {
    config: FsUsageConfig,
    patterns: Vec<Pattern>,
    process: Option<Child>,
    event_sender: Sender<FsEvent>,
    event_receiver: Receiver<FsEvent>,
    is_running: Arc<Mutex<bool>>,
}

impl FsUsageMonitor {
    pub fn new(config: FsUsageConfig) -> Result<Self> {
        let patterns = config
            .watch_paths
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile glob patterns")?;

        let (event_sender, event_receiver) = unbounded();

        Ok(Self {
            config,
            patterns,
            process: None,
            event_sender,
            event_receiver,
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if *self.is_running.lock().unwrap() {
            return Err(anyhow::anyhow!("Monitor is already running"));
        }

        let mut cmd = Command::new("fs_usage");
        cmd.arg("-w")
            .arg("-f")
            .arg("filesys")
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        // Only add -p flags if we have specific PIDs to watch
        if !self.config.watch_pids.is_empty() {
            for pid in &self.config.watch_pids {
                cmd.arg("-p").arg(pid.to_string());
            }
        }

        for process in &self.config.exclude_processes {
            cmd.arg("-e").arg(process);
        }

        info!("Starting fs_usage monitor with args: {:?}", cmd);
        let mut child = cmd.spawn().context("Failed to spawn fs_usage process")?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        *self.is_running.lock().unwrap() = true;
        self.process = Some(child);

        let sender = self.event_sender.clone();
        let patterns = self.patterns.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if !*is_running.lock().unwrap() {
                    break;
                }

                match line {
                    Ok(line) => {
                        debug!("Raw fs_usage line: {}", line);
                        if let Some(event) = parse_fs_usage_line(&line) {
                            debug!("Parsed event: {:?}", event);
                            if should_send_event(&event, &patterns, &config) {
                                debug!("Sending event for path: {}", event.path);
                                if let Err(e) = sender.send(event) {
                                    error!("Failed to send event: {}", e);
                                    break;
                                }
                            } else {
                                debug!("Event filtered out: {:?}", event);
                            }
                        } else {
                            debug!("Failed to parse line: {}", line);
                        }
                    }
                    Err(e) => {
                        error!("Error reading line: {}", e);
                        break;
                    }
                }
            }
            *is_running.lock().unwrap() = false;
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        *self.is_running.lock().unwrap() = false;

        if let Some(mut process) = self.process.take() {
            info!("Stopping fs_usage monitor");
            process.kill().context("Failed to kill fs_usage process")?;
            process.wait().context("Failed to wait for process")?;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    pub fn events(&self) -> &Receiver<FsEvent> {
        &self.event_receiver
    }

    pub fn try_recv(&self) -> Option<FsEvent> {
        self.event_receiver.try_recv().ok()
    }

    pub fn recv(&self) -> Result<FsEvent> {
        self.event_receiver
            .recv()
            .context("Failed to receive event")
    }
}

impl Drop for FsUsageMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn parse_fs_usage_line(line: &str) -> Option<FsEvent> {
    // fs_usage format examples:
    // 23:52:52.781431  fstatat64              [  2]           [-2]/private/tmp/test123.txt                                                                                                                                          0.001226   touch.3523509
    // 23:52:51.346567  lstat64                [  2]           private/tmp/LittleSnitchDebugLogs                                                                                                                                     0.000025   at.obdev.littlesnitch.networkex.3515250
    // 23:57:54.210609  read              F=86   B=0xea                                                                                                                                                                              0.000001   ghostty.3386479

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let timestamp = parts[0].to_string();
    let operation = parts[1].to_string();

    // Find the process.pid at the end (last part)
    let process_info = parts.last()?;

    // Parse process name and PID (format: processname.pid)
    let dot_pos = process_info.rfind('.')?;
    let process_name = process_info[..dot_pos].to_string();
    let pid = process_info[dot_pos + 1..].parse::<u32>().ok()?;

    // Find the path - it's between the operation and the duration/process
    // Skip timestamp, operation, and look for the path
    let mut path_parts = Vec::new();
    let mut found_path_start = false;

    for (i, part) in parts.iter().enumerate() {
        if i < 2 {
            continue;
        } // Skip timestamp and operation
        if i == parts.len() - 1 {
            break;
        } // Skip process.pid at end
        if i == parts.len() - 2 {
            break;
        } // Skip duration before process.pid

        // Skip optional info like [  2], F=86, B=0xea
        if part.starts_with('[') && part.ends_with(']') {
            continue;
        }
        if part.starts_with("F=") || part.starts_with("B=") {
            continue;
        }

        // Look for path indicators
        if part.contains('/') || found_path_start {
            found_path_start = true;
            path_parts.push(*part);
        }
    }

    // Skip events without file paths (just file descriptors)
    if path_parts.is_empty() {
        return None;
    }

    let path = path_parts
        .join(" ")
        .split("Err#")
        .next()?
        .trim()
        .to_string();

    // Clean up path - remove [-2] prefixes and normalize
    let path = if path.starts_with("[-") {
        path.split("]").nth(1)?.to_string()
    } else {
        path
    };

    // Convert private/tmp to /tmp etc
    let path = if path.starts_with("private/tmp") {
        path.replace("private/tmp", "/tmp")
    } else if path.starts_with("/private/tmp") {
        path.replace("/private/tmp", "/tmp")
    } else {
        path
    };

    // Skip if path is empty after cleanup
    if path.is_empty() {
        return None;
    }

    let result = if line.contains("Err#") {
        line.split("Err#")
            .nth(1)?
            .split_whitespace()
            .next()?
            .to_string()
    } else {
        "OK".to_string()
    };

    Some(FsEvent {
        timestamp,
        process_name,
        pid,
        operation,
        path,
        result,
    })
}

fn should_send_event(event: &FsEvent, patterns: &[Pattern], config: &FsUsageConfig) -> bool {
    debug!(
        "Checking event: pid={}, operation={}, path={}",
        event.pid, event.operation, event.path
    );

    if config.exclude_pids.contains(&event.pid) {
        debug!("Event excluded by PID: {}", event.pid);
        return false;
    }

    if !config.watch_pids.is_empty() && !config.watch_pids.contains(&event.pid) {
        debug!("Event not in watch PIDs: {}", event.pid);
        return false;
    }

    // Check operation type filtering
    if !config.operation_types.contains(&OperationType::All) {
        let matches_operation = config
            .operation_types
            .iter()
            .any(|op_type| op_type.matches_operation(&event.operation));
        if !matches_operation {
            debug!("Event operation '{}' not in allowed types", event.operation);
            return false;
        }
    }

    if patterns.is_empty() {
        debug!("No patterns, allowing event");
        return true;
    }

    for pattern in patterns {
        if pattern.matches(&event.path) {
            debug!(
                "Pattern '{}' matches path '{}'",
                pattern.as_str(),
                event.path
            );
            return true;
        } else {
            debug!(
                "Pattern '{}' does NOT match path '{}'",
                pattern.as_str(),
                event.path
            );
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fs_usage_line() {
        // Test actual fs_usage format
        let line = "23:52:52.781431  fstatat64              [  2]           [-2]/private/tmp/test123.txt                                                                                                                                          0.001226   touch.3523509";
        let event = parse_fs_usage_line(line).unwrap();
        assert_eq!(event.timestamp, "23:52:52.781431");
        assert_eq!(event.operation, "fstatat64");
        assert_eq!(event.process_name, "touch");
        assert_eq!(event.pid, 3523509);
        assert_eq!(event.path, "/tmp/test123.txt");

        // Test another format
        let line2 = "23:52:51.346567  lstat64                [  2]           private/tmp/LittleSnitchDebugLogs                                                                                                                                     0.000025   at.obdev.littlesnitch.networkex.3515250";
        let event2 = parse_fs_usage_line(line2).unwrap();
        assert_eq!(event2.operation, "lstat64");
        assert_eq!(event2.process_name, "at.obdev.littlesnitch.networkex");
        assert_eq!(event2.pid, 3515250);
        assert_eq!(event2.path, "/tmp/LittleSnitchDebugLogs");
    }

    #[test]
    fn test_glob_patterns() {
        let pattern = Pattern::new("/Users/*/Documents/*.txt").unwrap();
        assert!(pattern.matches("/Users/john/Documents/file.txt"));
        assert!(!pattern.matches("/Users/john/Downloads/file.txt"));

        // Test recursive glob
        let pattern2 = Pattern::new("/tmp/**/*").unwrap();
        assert!(pattern2.matches("/tmp/test.txt"));
        assert!(pattern2.matches("/tmp/a/b/c/test.txt"));
        assert!(pattern2.matches("/tmp/subfolder/file.log"));
        assert!(!pattern2.matches("/var/tmp/test.txt"));
    }

    #[test]
    fn test_operation_filtering() {
        assert!(OperationType::Write.matches_operation("write"));
        assert!(OperationType::Write.matches_operation("WrData"));
        assert!(!OperationType::Write.matches_operation("read"));

        assert!(OperationType::Read.matches_operation("read"));
        assert!(OperationType::Read.matches_operation("RdData"));
        assert!(!OperationType::Read.matches_operation("write"));

        assert!(OperationType::Create.matches_operation("open"));
        assert!(OperationType::Delete.matches_operation("unlink"));
        assert!(OperationType::Move.matches_operation("rename"));

        assert!(OperationType::All.matches_operation("anything"));
    }
}
