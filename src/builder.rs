use crate::{FsUsageConfig, FsUsageMonitor, OperationType};
use anyhow::Result;

pub struct FsUsageMonitorBuilder {
    config: FsUsageConfig,
}

impl FsUsageMonitorBuilder {
    pub fn new() -> Self {
        Self {
            config: FsUsageConfig::default(),
        }
    }

    pub fn watch_path(mut self, path: impl Into<String>) -> Self {
        self.config.watch_paths.push(path.into());
        self
    }

    pub fn watch_paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.watch_paths.extend(paths.into_iter().map(|p| p.into()));
        self
    }

    pub fn watch_pid(mut self, pid: u32) -> Self {
        self.config.watch_pids.push(pid);
        self
    }

    pub fn watch_pids(mut self, pids: impl IntoIterator<Item = u32>) -> Self {
        self.config.watch_pids.extend(pids);
        self
    }

    pub fn exclude_pid(mut self, pid: u32) -> Self {
        self.config.exclude_pids.push(pid);
        self
    }

    pub fn exclude_pids(mut self, pids: impl IntoIterator<Item = u32>) -> Self {
        self.config.exclude_pids.extend(pids);
        self
    }

    pub fn exclude_process(mut self, process: impl Into<String>) -> Self {
        self.config.exclude_processes.push(process.into());
        self
    }

    pub fn exclude_processes(mut self, processes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.exclude_processes.extend(processes.into_iter().map(|p| p.into()));
        self
    }

    pub fn watch_operations(mut self, operations: impl IntoIterator<Item = OperationType>) -> Self {
        self.config.operation_types = operations.into_iter().collect();
        self
    }

    pub fn watch_writes_only(mut self) -> Self {
        self.config.operation_types = vec![OperationType::Write, OperationType::Create, OperationType::Delete, OperationType::Move];
        self
    }

    pub fn watch_reads_only(mut self) -> Self {
        self.config.operation_types = vec![OperationType::Read];
        self
    }

    pub fn exclude_metadata(mut self) -> Self {
        self.config.operation_types = vec![OperationType::Read, OperationType::Write, OperationType::Create, OperationType::Delete, OperationType::Move];
        self
    }

    pub fn build(self) -> Result<FsUsageMonitor> {
        FsUsageMonitor::new(self.config)
    }
}

impl Default for FsUsageMonitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}