#!/bin/bash

# Build script for electron-dragfile-plugin
set -e

echo "🔨 Building electron-dragfile-plugin..."

# Clean any existing builds
echo "🧹 Cleaning previous builds..."
rm -rf *.node

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build the native addon
echo "🦀 Building Rust native addon..."
npm run build

echo "✅ Build completed successfully!"

# List the generated files
echo "📁 Generated files:"
ls -la *.node 2>/dev/null || echo "No .node files found"

echo "🎉 electron-dragfile-plugin is ready to use!"