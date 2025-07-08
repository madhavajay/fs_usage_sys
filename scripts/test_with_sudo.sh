#!/bin/bash

# Script to run tests that require sudo
# This is meant to be used in CI environments where sudo is available without password

set -e

echo "Running tests that require sudo permissions..."

# Check if we can run sudo without password
if sudo -n true 2>/dev/null; then
    echo "✅ Sudo available without password (CI environment)"
    
    # Run the ignored tests with sudo
    sudo cargo test -- --ignored --nocapture
else
    echo "⚠️  Sudo requires password (local environment)"
    echo "To run these tests locally, use: sudo cargo test -- --ignored --nocapture"
    exit 0
fi