# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `OperationType::Chmod` variant for detecting file permission changes
- `exact_path_matching()` builder method for efficient path containment matching
- `watch_mutations_only()` builder method to filter only write/delete/rename/chmod operations
- Support for all `WrData` variants including `WrData[A]`, `WrData[AT3]`, etc.
- Default exclusions for noisy system processes (mds, mdworker, fseventsd)
- Example `watch_mutations.rs` demonstrating real-time write detection

### Changed
- **BREAKING**: Modified `FsUsageConfig` to include `exact_path_matching` field
- Updated fs_usage flags from `-f filesys -f diskio` to `-f pathname,filesys` for better event coverage
- Enhanced `OperationType::Write` to include rename, unlink, and chmod_extended operations
- Improved path matching to support both absolute and relative path detection

### Fixed
- Missing write events that were not captured with previous fs_usage flags
- Detection of chmod and chmod_extended operations
- WrData event parsing for various format variants

### Performance
- Significant improvement in path matching performance when using exact_path_matching mode
- Reduced latency by using more efficient fs_usage flags

## [0.1.3] - 2024-01-08

### Fixed
- Fixed build issues on non-macOS platforms
- Added proper cfg attributes for macOS-specific code

## [0.1.0] - 2024-01-06

### Added
- Initial release of fs_usage_sys
- Real-time file system monitoring for macOS using fs_usage
- Path filtering with glob patterns (e.g., `/Users/*/Documents/**/*.txt`)
- Process filtering by PID or process name
- Operation type filtering (reads, writes, creates, deletes, moves, etc.)
- Builder pattern API for easy configuration
- Event streaming via channels
- AI assistant detection for distinguishing between automated and manual file changes
- Examples:
  - basic_monitor: Simple file monitoring
  - process_filter: Process-based filtering and categorization
  - writes_only: Monitor only write operations
  - debug_monitor: Debug mode with detailed logging
- Comprehensive documentation and API reference

### Features
- Noise reduction by filtering metadata operations
- Support for multiple glob patterns
- Process exclusion lists
- Automatic path normalization (/private/tmp → /tmp)
- Robust fs_usage output parsing

### Known Limitations
- macOS only (uses system fs_usage command)
- Requires sudo/root privileges
- Binary parsing may need updates for future macOS versions
