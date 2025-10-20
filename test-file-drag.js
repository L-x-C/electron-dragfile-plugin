/* tslint:disable */
/* eslint-disable */
/* prettier-ignore */

// Test file for NSPasteboard-based file drag monitoring

const {
  startFileDragMonitor,
  stopFileDragMonitor,
  onFileDragEvent,
  removeFileDragEventListener,
  isFileDragMonitoring
} = require('./index');

const path = require('path');

// Test configuration
const TEST_TIMEOUT = 30000; // 30 seconds

console.log('🗂️  Testing NSPasteboard-based file drag monitoring...');
console.log('=' .repeat(60));

async function testFileDragMonitoring() {
  try {
    // Check initial state
    console.log('1️⃣ Initial state check:');
    console.log(`   File drag monitoring active: ${isFileDragMonitoring()}`);

    // Register file drag event callback
    console.log('\n2️⃣ Registering file drag event callback...');
    const callbackId = onFileDragEvent((err, event) => {
      if (err) {
        console.error('❌ File drag event error:', err);
        return;
      }

      console.log(`\n📁 File Drag Event Received:`);
      console.log(`   Event Type: ${event.eventType}`);
      console.log(`   File Path: ${event.filePath || '(none)'}`);
      console.log(`   Position: (${event.x.toFixed(2)}, ${event.y.toFixed(2)})`);
      console.log(`   Timestamp: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
      console.log(`   Platform: ${event.platform}`);
      console.log(`   Window ID: ${event.windowId}`);

      if (event.eventType === 'hovered_file') {
        console.log(`   🎯 File is being hovered: ${event.filePath}`);
      } else if (event.eventType === 'dropped_file') {
        console.log(`   ✅ File was dropped: ${event.filePath}`);
      } else if (event.eventType === 'hovered_file_cancelled') {
        console.log(`   ❌ File hover cancelled`);
      }
    });

    console.log(`   ✅ Callback registered with ID: ${callbackId}`);

    // Start file drag monitoring
    console.log('\n3️⃣ Starting file drag monitoring...');
    // Helper path is no longer needed - using NSPasteboard directly
    await startFileDragMonitor("dummy-path-for-compatibility");
    console.log(`   ✅ File drag monitoring started`);
    console.log(`   ✅ File drag monitoring active: ${isFileDragMonitoring()}`);

    console.log('\n4️⃣ File drag monitoring is now active!');
    console.log('   💡 Try dragging files over any part of your screen');
    console.log('   💡 You should see events in the console');
    console.log('   💡 NSPasteboard will detect file drags system-wide');

    // Wait for user to test
    console.log(`\n⏳ Testing for ${TEST_TIMEOUT / 1000} seconds...`);
    console.log('   💡 Drag some files around your desktop to test!');

    await new Promise(resolve => setTimeout(resolve, TEST_TIMEOUT));

    // Test removing callback
    console.log('\n5️⃣ Removing event callback...');
    const removed = removeFileDragEventListener(callbackId);
    console.log(`   ✅ Callback removed: ${removed}`);

    // Stop file drag monitoring
    console.log('\n6️⃣ Stopping file drag monitoring...');
    await stopFileDragMonitor();
    console.log(`   ✅ File drag monitoring stopped`);
    console.log(`   ✅ File drag monitoring active: ${isFileDragMonitoring()}`);

    console.log('\n✅ File drag monitoring test completed successfully!');

  } catch (error) {
    console.error('❌ Test failed:', error);

    try {
      // Try to cleanup on error
      await stopFileDragMonitor();
    } catch (cleanupError) {
      console.error('❌ Cleanup failed:', cleanupError);
    }
  }
}

// Test error handling
async function testErrorHandling() {
  console.log('\n🧪 Testing error handling...');

  try {
    // Test starting twice
    console.log('   Testing double start...');
    await startFileDragMonitor("dummy-path-for-compatibility");
    await startFileDragMonitor("dummy-path-for-compatibility"); // Should not error
    console.log('   ✅ Double start handled gracefully');

    // Test stopping twice
    console.log('   Testing double stop...');
    await stopFileDragMonitor();
    await stopFileDragMonitor(); // Should not error
    console.log('   ✅ Double stop handled gracefully');

  } catch (error) {
    console.error('   ❌ Error handling test failed:', error);
  }
}

// Test event validation
async function testEventValidation() {
  console.log('\n🔍 Testing event validation...');

  try {
    await startFileDragMonitor("dummy-path-for-compatibility");

    // Register validation callback
    const validationCallbackId = onFileDragEvent((err, event) => {
      if (err) {
        console.error('   ❌ Validation callback error:', err);
        return;
      }

      // Validate event structure
      const requiredFields = ['eventType', 'file_path', 'x', 'y', 'timestamp', 'platform', 'window_id'];
      const missingFields = requiredFields.filter(field => !(field in event));

      if (missingFields.length > 0) {
        console.error(`   ❌ Missing event fields: ${missingFields.join(', ')}`);
        return;
      }

      // Validate event type
      const validEventTypes = ['hovered_file', 'dropped_file', 'hovered_file_cancelled'];
      if (!validEventTypes.includes(event.eventType)) {
        console.error(`   ❌ Invalid event type: ${event.eventType}`);
        return;
      }

      console.log('   ✅ Event validation passed');
    });

    console.log('   📋 Validation callback registered (ID: ' + validationCallbackId + ')');
    console.log('   💡 Drag files to validate event structure...');

    // Wait a bit for validation
    await new Promise(resolve => setTimeout(resolve, 5000));

    // Cleanup
    removeFileDragEventListener(validationCallbackId);
    await stopFileDragMonitor();

  } catch (error) {
    console.error('   ❌ Event validation test failed:', error);
  }
}

// Run all tests
async function runAllTests() {
  console.log('🚀 Starting comprehensive file drag monitoring tests...\n');

  await testFileDragMonitoring();
  await testErrorHandling();
  await testEventValidation();

  console.log('\n🏁 All tests completed!');
  console.log('💡 If you didn\'t see any file drag events, try:');
  console.log('   - Make sure you actually dragged files on your screen');
  console.log('   - Check if any security permissions are needed');
  console.log('   - Verify NSPasteboard access is working');
  console.log('\n👋 Test exiting...');
}

// Handle cleanup on exit
process.on('SIGINT', () => {
  console.log('\n\n🛑 Received SIGINT, cleaning up...');
  stopFileDragMonitor().then(() => {
    console.log('✅ Cleanup complete, exiting...');
    process.exit(0);
  }).catch((error) => {
    console.error('❌ Cleanup failed:', error);
    process.exit(1);
  });
});

// Run the tests
runAllTests().catch((error) => {
  console.error('❌ Test suite failed:', error);
  process.exit(1);
});