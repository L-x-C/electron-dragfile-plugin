#!/usr/bin/env node

// Test script to verify multi-platform installation
const path = require('path');
const fs = require('fs');

console.log('🔍 Testing multi-platform installation...\n');

try {
  // Try to require the main module
  const dragPlugin = require('../index.js');

  console.log('✅ Module loaded successfully');

  // Check if monitoring function exists
  if (typeof dragPlugin.startDragMonitor === 'function') {
    console.log('✅ startDragMonitor function available');
  } else {
    console.log('❌ startDragMonitor function not found');
  }

  if (typeof dragPlugin.stopDragMonitor === 'function') {
    console.log('✅ stopDragMonitor function available');
  } else {
    console.log('❌ stopDragMonitor function not found');
  }

  if (typeof dragPlugin.onDragEvent === 'function') {
    console.log('✅ onDragEvent function available');
  } else {
    console.log('❌ onDragEvent function not found');
  }

  if (typeof dragPlugin.isMonitoring === 'function') {
    console.log('✅ isMonitoring function available');
  } else {
    console.log('❌ isMonitoring function not found');
  }

  // Check native binding
  console.log('\n🔧 Native binding information:');
  console.log(`Platform: ${process.platform}`);
  console.log(`Arch: ${process.arch}`);

  // List available .node files
  const nodeFiles = fs.readdirSync('.').filter(file => file.endsWith('.node'));
  console.log(`\n📦 Available .node files: ${nodeFiles.length}`);
  nodeFiles.forEach(file => {
    const stats = fs.statSync(file);
    console.log(`  - ${file} (${Math.round(stats.size / 1024)}KB)`);
  });

  console.log('\n✅ Multi-platform installation test completed successfully');

} catch (error) {
  console.error('❌ Module loading failed:', error.message);
  console.error('This might indicate missing platform-specific binaries');
  process.exit(1);
}