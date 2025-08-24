#!/bin/bash

# Bolt Language Build Script

echo "🔨 Building Bolt compiler..."
cargo build --quiet

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo "📦 Bolt compiler available at: ./target/debug/bolt"
echo ""
echo "Usage examples:"
echo "  Debug build:   ./target/debug/bolt program.bolt -o program"
echo "  Release build: ./target/debug/bolt program.bolt -o program --release"
echo "  Run tests:     ./run_tests.sh"
echo ""
echo "Output directories:"
echo "  Debug builds:   out/debug/"
echo "  Release builds: out/release/"