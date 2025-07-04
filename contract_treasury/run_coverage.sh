#!/bin/bash

# Treasury Contract Test Coverage Script
# This script runs comprehensive test coverage using tarpaulin

echo "🚀 Starting Treasury Contract Test Coverage Analysis..."

# Create coverage directory if it doesn't exist
mkdir -p coverage

# Install tarpaulin if not already installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Run tests with coverage
echo "🔍 Running tests with coverage analysis..."
cargo tarpaulin \
    --out Html \
    --out Lcov \
    --output-dir ./coverage \
    --fail-under 80 \
    --line \
    --branch \
    --follow-exec \
    --all-features \
    --verbose \
    --target-dir ./target/tarpaulin \
    --timeout 300

# Check if coverage succeeded
if [ $? -eq 0 ]; then
    echo "✅ Test coverage analysis completed successfully!"
    echo "📊 Coverage report generated in ./coverage directory"
    echo "📁 HTML report: ./coverage/tarpaulin-report.html"
    echo "📁 LCOV report: ./coverage/lcov.info"
    
    # Display coverage summary if HTML report exists
    if [ -f "./coverage/tarpaulin-report.html" ]; then
        echo "📈 Opening coverage report location..."
        echo "   File path: $(realpath ./coverage/tarpaulin-report.html)"
    fi
else
    echo "❌ Test coverage analysis failed!"
    exit 1
fi

echo "🎉 Treasury Contract Test Coverage Analysis Complete!"