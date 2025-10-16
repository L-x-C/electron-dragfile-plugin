#!/usr/bin/env node

const path = require('path');

// Load the plugin
const {
  startFileDragMonitor,
  stopFileDragMonitor,
  onFileDragEvent,
  removeFileDragEventListener,
  isFileDragMonitoring
} = require('./index');

console.log('🗂️  Quick File Drag Test');
console.log('========================');

async function quickTest() {
  try {
    // Register callback
    const callbackId = onFileDragEvent((err, event) => {
      if (err) {
        console.error('❌ Error:', err);
        return;
      }

      console.log(`\n📁 File Event: ${event.eventType}`);
      console.log(`   Path: ${event.filePath || '(none)'}`);
      console.log(`   Position: (${event.x.toFixed(1)}, ${event.y.toFixed(1)})`);
      console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
    });

    console.log('✅ Callback registered');

    // Start monitoring
    const helperPath = path.join(__dirname, 'target', 'release', 'drag-monitor-helper');
    await startFileDragMonitor(helperPath);
    console.log('✅ File drag monitoring started');
    console.log('✅ Active:', isFileDragMonitoring());

    console.log('\n💡 Drag files around your screen to test!');
    console.log('⏹️  Press Ctrl+C to stop');

    // Handle cleanup
    process.on('SIGINT', async () => {
      console.log('\n🛑 Stopping...');
      removeFileDragEventListener(callbackId);
      await stopFileDragMonitor();
      console.log('✅ Stopped');
      process.exit(0);
    });

  } catch (error) {
    console.error('❌ Error:', error.message);
    process.exit(1);
  }
}

quickTest();