# Testing Guide

## Running Tests

### Basic Tests (No sudo required)
```bash
cargo test
# or
./lint.sh
```

### Tests Requiring Sudo
Some tests require sudo permissions because `fs_usage` must be run as root on macOS:

```bash
# Run all tests including those requiring sudo
sudo cargo test -- --ignored --nocapture

# Run specific test with sudo
sudo cargo test test_captures_write_operations -- --ignored --nocapture
```

### CI Testing
For CI environments where sudo is available without password:

```bash
./scripts/test_with_sudo.sh
```

## Test Categories

1. **Unit Tests** - Test parsing and internal logic (no sudo required)
2. **Integration Tests** - Test monitor lifecycle (some require sudo)
3. **File System Tests** - Test actual file system event capture (require sudo)

## Troubleshooting

If tests fail with "No events were captured":
- Ensure you're running with sudo
- The monitored path must exist and be accessible
- fs_usage might need more time to start up
- Some paths (like temp directories) might not be properly monitored by fs_usage

## GitHub Actions CI

The ignored tests that require sudo can be run in CI by:
1. Using a macOS runner
2. Configuring passwordless sudo
3. Running `./scripts/test_with_sudo.sh`