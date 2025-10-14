#!/usr/bin/env node

// Verification script for multi-platform npm package
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔍 Verifying multi-platform npm package...\n');

// Package name to verify
const packageName = 'electron-dragfile-plugin';

async function verifyNpmPackage() {
  try {
    console.log(`📦 Checking npm package: ${packageName}`);

    // Get package info from npm
    const packageInfo = JSON.parse(execSync(`npm view ${packageName} --json`, { encoding: 'utf8' }));
    console.log(`✅ Latest version: ${packageInfo.version}`);
    console.log(`✅ Description: ${packageInfo.description}`);

    // Check optional dependencies
    if (packageInfo.optionalDependencies) {
      console.log('\n🔧 Optional dependencies (platform binaries):');
      Object.keys(packageInfo.optionalDependencies).forEach(dep => {
        console.log(`  ✅ ${dep}@${packageInfo.optionalDependencies[dep]}`);
      });
    }

    // Check files in package
    if (packageInfo.files) {
      console.log('\n📁 Files included in package:');
      packageInfo.files.forEach(file => {
        console.log(`  ✅ ${file}`);
      });
    }

    // Download and inspect package tarball
    console.log('\n📥 Downloading package to verify contents...');
    const tarballPath = execSync(`npm pack ${packageName}`, { encoding: 'utf8' }).trim();
    console.log(`✅ Downloaded: ${tarballPath}`);

    // Extract and check contents (simple tar listing)
    try {
      const tarContents = execSync(`tar -tzf ${tarballPath}`, { encoding: 'utf8' });
      const nodeFiles = tarContents.split('\n').filter(line => line.endsWith('.node'));

      console.log('\n🔧 Binary files in package:');
      if (nodeFiles.length > 0) {
        nodeFiles.forEach(file => {
          console.log(`  ✅ ${file}`);
        });
      } else {
        console.log('  ❌ No .node files found in package');
      }

      console.log(`\n📊 Total files in package: ${tarContents.split('\n').filter(f => f).length}`);
      console.log(`📊 Binary files: ${nodeFiles.length}`);

    } catch (tarError) {
      console.log('  ⚠️  Could not inspect package contents');
    }

    // Clean up
    try {
      fs.unlinkSync(tarballPath);
    } catch (cleanupError) {
      // Ignore cleanup errors
    }

    console.log('\n✅ Multi-platform package verification completed');

  } catch (error) {
    console.error('❌ Verification failed:', error.message);
    process.exit(1);
  }
}

// Check if this is being run directly
if (require.main === module) {
  verifyNpmPackage();
}

module.exports = { verifyNpmPackage };