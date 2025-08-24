#!/bin/bash

# Build the compiler
echo "Building Bolt compiler..."
cargo build --quiet

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo

# Run tests
PASSED=0
FAILED=0

# Create test output directory
mkdir -p out/test

for test_file in tests/*.bolt; do
    if [ -f "$test_file" ]; then
        basename=$(basename "$test_file" .bolt)
        expected_file="tests/expected/${basename}.txt"
        output_exe="test_${basename}"
        
        echo "🧪 Testing $basename..."
        
        # Compile the test (debug mode for better error messages)
        ./target/debug/bolt "$test_file" -o "$output_exe" > /dev/null 2>&1
        
        if [ $? -ne 0 ]; then
            echo "❌ $basename: Compilation failed"
            FAILED=$((FAILED + 1))
            continue
        fi
        
        # Run the test and capture output
        actual_output=$(timeout 5s ./out/debug/"$output_exe" 2>&1)
        exit_code=$?
        
        if [ $exit_code -eq 124 ]; then
            echo "❌ $basename: Test timed out"
            FAILED=$((FAILED + 1))
            continue
        elif [ $exit_code -ne 0 ]; then
            echo "❌ $basename: Runtime error (exit code $exit_code)"
            FAILED=$((FAILED + 1))
            continue
        fi
        
        # Compare with expected output
        if [ -f "$expected_file" ]; then
            expected_output=$(cat "$expected_file")
            if [ "$actual_output" = "$expected_output" ]; then
                echo "✅ $basename: PASSED"
                PASSED=$((PASSED + 1))
            else
                echo "❌ $basename: FAILED"
                echo "  Expected:"
                echo "$expected_output" | sed 's/^/    /'
                echo "  Actual:"
                echo "$actual_output" | sed 's/^/    /'
                FAILED=$((FAILED + 1))
            fi
        else
            echo "⚠️  $basename: No expected output file found"
            echo "  Actual output:"
            echo "$actual_output" | sed 's/^/    /'
        fi
    fi
done

echo
echo "📊 Test Results: $PASSED passed, $FAILED failed"

# Optional: Clean up test artifacts (uncomment if you want automatic cleanup)
# echo "🧹 Cleaning up test artifacts..."
# rm -rf out/debug/test_*

if [ $FAILED -eq 0 ]; then
    echo "🎉 All tests passed!"
    echo "📁 Test executables are in out/debug/ for debugging"
    exit 0
else
    echo "💥 Some tests failed!"
    echo "📁 Test executables are in out/debug/ for debugging"
    exit 1
fi