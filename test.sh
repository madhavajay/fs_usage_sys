#!/bin/bash

echo "Testing with quoted glob pattern..."
echo "This prevents shell expansion"

# Test with quoted pattern
sudo cargo run --example process_filter '/tmp/**/*'