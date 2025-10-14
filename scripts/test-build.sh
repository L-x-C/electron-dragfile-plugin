#!/bin/bash

# Test cross-platform build locally
echo "Testing local cross-platform build..."

# Clean existing builds
rm -f *.node
rm -rf target/

# Test current platform build
echo "Building for current platform..."
npm run build

# Check if binary was created
if ls *.node 1> /dev/null 2>&1; then
    echo "✅ Current platform build successful"
    ls -la *.node
else
    echo "❌ Current platform build failed"
    exit 1
fi

# Test prepublish
echo "Testing prepublish..."
npm run prepublishOnly

echo "✅ Local build test complete"