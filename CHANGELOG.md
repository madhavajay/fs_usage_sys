# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/madhavajay/fs_usage_sys/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/madhavajay/fs_usage_sys/releases/tag/v0.1.0