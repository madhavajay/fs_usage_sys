#!/bin/bash

# Check if --fix flag is provided
FIX_MODE=false
if [[ "$1" == "--fix" ]]; then
    FIX_MODE=true
fi

# Run cargo fmt
if $FIX_MODE; then
    echo "Running cargo fmt (fix mode)..."
    cargo fmt
    echo "✅ Code formatted!"
else
    echo "Running cargo fmt (check mode)..."
    if cargo fmt -- --check; then
        echo "✅ Code is properly formatted!"
    else
        echo "❌ Code needs formatting. Run './scripts/lint.sh --fix' to fix."
        exit 1
    fi
fi

# Run cargo clippy
echo -e "\nRunning cargo clippy..."
if $FIX_MODE; then
    cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
else
    cargo clippy -- -D warnings
fi

# Run tests
echo -e "\nRunning tests..."
cargo test

# Note about ignored tests
echo -e "\nNote: Some tests require sudo permissions and are ignored by default."
echo "To run all tests including those requiring sudo: sudo cargo test -- --ignored"