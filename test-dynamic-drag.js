#!/usr/bin/env node

const path = require('path');

// Load the plugin
const {
  startMouseMonitor,
  stopMouseMonitor,
  onMouseEvent,
  removeMouseEventListener,
  isMonitoring,
  startFileDragMonitor,
  stopFileDragMonitor,
  onFileDragEvent,
  removeFileDragEventListener,
  isFileDragMonitoring
} = require('./index');

console.log('🗂️  Dynamic File Drag Test (Mouse-Driven)');
console.log('==========================================');

async function dynamicTest() {
  try {
    // 1. Start mouse monitoring first
    console.log('1️⃣ Starting mouse monitoring...');
    await startMouseMonitor();
    console.log(`   ✅ Mouse monitoring active: ${isMonitoring()}`);

    // 2. Register mouse event callback
    console.log('\n2️⃣ Registering mouse event callback...');
    const mouseCallbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('❌ Mouse event error:', err);
        return;
      }

      if (event.eventType === 'mousedown') {
        console.log(`   🖱️ Mouse DOWN at (${event.x.toFixed(1)}, ${event.y.toFixed(1)})`);
      } else if (event.eventType === 'mouseup') {
        console.log(`   🖱️ Mouse UP at (${event.x.toFixed(1)}, ${event.y.toFixed(1)})`);
      }
    });
    console.log(`   ✅ Mouse callback registered: ${mouseCallbackId}`);

    // 3. Register file drag callback
    console.log('\n3️⃣ Registering file drag callback...');
    const dragCallbackId = await onFileDragEvent((err, event) => {
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

      if (event.eventType === 'hovered_file') {
        console.log(`   🎯 File is being hovered: ${event.filePath}`);
      } else if (event.eventType === 'dropped_file') {
        console.log(`   ✅ File was dropped: ${event.filePath}`);
      } else if (event.eventType === 'hovered_file_cancelled') {
        console.log(`   ❌ File hover cancelled`);
      }
    });
    console.log(`   ✅ Drag callback registered: ${dragCallbackId}`);

    // 4. Configure file drag monitoring (but don't start it yet)
    console.log('\n4️⃣ Configuring dynamic file drag monitoring...');
    const helperPath = path.join(__dirname, 'target', 'release', 'drag-monitor-helper');
    await startFileDragMonitor(helperPath);
    console.log(`   ✅ File drag monitoring configured`);
    console.log(`   ✅ File drag monitoring active: ${isFileDragMonitoring()}`);

    // 5. Test the dynamic behavior
    console.log('\n5️⃣ Dynamic monitoring is now configured!');
    console.log('   💡 The system will now create drag detection windows on mouse down');
    console.log('   💡 And destroy them on mouse up');
    console.log('   💡 This prevents window focus conflicts');
    console.log('\n🧪 Test Instructions:');
    console.log('   1. Press and hold the mouse button anywhere');
    console.log('   2. While holding, try to drag files to any location');
    console.log('   3. Release the mouse button');
    console.log('   4. You should see window creation/destruction logs');
    console.log('\n⏹️  Press Ctrl+C to stop');

    // Handle cleanup
    process.on('SIGINT', async () => {
      console.log('\n🛑 Cleaning up...');

      // Stop monitoring
      await removeMouseEventListener(mouseCallbackId);
      await removeFileDragEventListener(dragCallbackId);
      await stopMouseMonitor();
      await stopFileDragMonitor();

      console.log('✅ Cleanup complete');
      process.exit(0);
    });

  } catch (error) {
    console.error('❌ Error:', error.message);
    process.exit(1);
  }
}

// Run the test
dynamicTest();