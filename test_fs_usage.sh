#!/bin/bash

echo "Testing raw fs_usage output..."
echo "This will monitor /tmp for 5 seconds"
echo "Try creating a file in /tmp in another terminal:"
echo "  touch /tmp/test_file.txt"
echo ""

sudo fs_usage -w -f filesys | grep -E "/tmp" | head -20