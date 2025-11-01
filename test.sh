#!/bin/bash

# Quick test script to validate basic functionality
echo "Testing Linux Process Manager basic functionality..."

cd /home/t-aelaswar/process-manager

# Test compilation
echo "1. Testing compilation..."
if cargo build --quiet; then
    echo "   ✅ Compilation successful"
else
    echo "   ❌ Compilation failed"
    exit 1
fi

# Test that the binary exists
echo "2. Testing binary creation..."
if [ -f "target/debug/process-manager" ]; then
    echo "   ✅ Binary created successfully"
else
    echo "   ❌ Binary not found"
    exit 1
fi

# Test help output
echo "3. Testing help functionality..."
if ./target/debug/process-manager --help >/dev/null 2>&1; then
    echo "   ✅ Help command works"
else
    echo "   ❌ Help command failed"
fi

# Test version output
echo "4. Testing version output..."
if ./target/debug/process-manager --version >/dev/null 2>&1; then
    echo "   ✅ Version command works"
else
    echo "   ❌ Version command failed"
fi

echo ""
echo "Basic functionality tests completed!"
echo ""
echo "To run the interactive process manager:"
echo "   cargo run"
echo ""
echo "Or directly:"
echo "   ./target/debug/process-manager"
echo ""
echo "Key features implemented:"
echo "- Real-time process monitoring"
echo "- Interactive sorting and filtering"
echo "- Process tree visualization"
echo "- Safe process termination"
echo "- System resource monitoring"
echo ""
echo "Note: The application will take over your terminal when running."
echo "Use 'h' for help and 'q' to quit once inside the application."