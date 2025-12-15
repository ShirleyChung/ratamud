#!/bin/bash

echo "=========================================="
echo "Building ratamud..."
echo "=========================================="

cd /Users/shireychung/ratamud

# Run cargo build
cargo build 2>&1

# Check if build was successful
if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "✅ Build successful!"
    echo "=========================================="
else
    echo ""
    echo "=========================================="
    echo "❌ Build failed!"
    echo "=========================================="
    exit 1
fi
