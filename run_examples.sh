#!/bin/bash

# Run all example programs to verify they work correctly
echo "🎯 Running Bolt Language Examples..."
echo ""

# Build compiler first
echo "Building compiler..."
cargo build > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "❌ Compiler build failed!"
    exit 1
fi

FAILED=0
PASSED=0

run_example() {
    local name=$1
    local file=$2
    local output_name=$3
    
    echo "🚀 Running $name example..."
    echo "----------------------------------------"
    
    if ./target/debug/bolt "$file" -o "$output_name" > /dev/null 2>&1; then
        if ./out/debug/$output_name; then
            echo ""
            echo "✅ $name example completed successfully"
            ((PASSED++))
        else
            echo "❌ $name example failed at runtime"
            ((FAILED++))
        fi
    else
        echo "❌ $name example failed to compile"
        ((FAILED++))
    fi
    echo "========================================"
    echo ""
}

# Run all examples
run_example "Hello World" "examples/hello.bolt" "hello"  
run_example "Calculator" "examples/calculator.bolt" "calculator"
run_example "Logic Demo" "examples/logic_demo.bolt" "logic_demo"
run_example "Showcase" "examples/showcase.bolt" "showcase"

# Summary
echo "📊 Example Results: $PASSED passed, $FAILED failed"

if [ $FAILED -eq 0 ]; then
    echo "🎉 All examples working perfectly!"
    exit 0
else
    echo "💥 Some examples failed!"
    exit 1
fi