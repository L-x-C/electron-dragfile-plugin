/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

// Simple winit test to debug the issue

console.log('🔍 Testing winit basic functionality...');

// First, let's see if the module loads properly
try {
  const {
    startFileDragMonitor,
    stopFileDragMonitor,
    isFileDragMonitoring
  } = require('./index');

  console.log('✅ Module loaded successfully');
  console.log('📋 Available functions:', Object.keys({
    startFileDragMonitor,
    stopFileDragMonitor,
    isFileDragMonitoring
  }));

  console.log('🔍 Testing initial state...');
  console.log('   File drag monitoring active:', isFileDragMonitoring());

  console.log('\n🚀 Starting file drag monitor...');

  // Try to start with a timeout
  const startTime = Date.now();

  startFileDragMonitor();

  // Give it some time to start
  setTimeout(() => {
    console.log(`⏱️  After ${Date.now() - startTime}ms:`);
    console.log('   File drag monitoring active:', isFileDragMonitoring());

    // Try to stop after a short time
    setTimeout(() => {
      console.log('\n🛑 Stopping file drag monitor...');
      stopFileDragMonitor();
      console.log('   File drag monitoring active:', isFileDragMonitoring());
      console.log('\n✅ Test completed');
    }, 3000);

  }, 2000);

} catch (error) {
  console.error('❌ Error loading or testing module:', error);
  console.error('Stack trace:', error.stack);
}

// Also check if native binding exists
console.log('\n🔍 Checking native binding...');
try {
  const nativeBinding = require('./index.node');
  console.log('✅ Native binding loaded');
  console.log('Available functions:', Object.keys(nativeBinding));
} catch (error) {
  console.error('❌ Error loading native binding:', error);
}