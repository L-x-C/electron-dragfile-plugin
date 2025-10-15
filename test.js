#!/usr/bin/env node

const path = require('path');

// Load the plugin
let dragPlugin;
try {
    const localPath = path.join(__dirname, 'index.js');
    if (require('fs').existsSync(localPath)) {
        dragPlugin = require(localPath);
    } else {
        dragPlugin = require('electron-dragfile-plugin');
    }
    console.log('✅ Plugin loaded');
} catch (error) {
    console.error('❌ Failed to load plugin:', error.message);
    console.error('Run: npm run build');
    process.exit(1);
}

async function main() {
    try {
        console.log('🚀 Starting drag monitoring...');

        // Start monitoring
        await dragPlugin.startDragMonitor();
        console.log('✅ Monitoring started');

        // Register callback
        const callbackId = await dragPlugin.onDragEvent((event) => {
            console.log('🎯 DRAG DETECTED!');
            console.log('   Files:', event.files);
            console.log('   Position:', `(${event.x}, ${event.y})`);
            console.log('   Platform:', event.platform);
            console.log('   Time:', new Date(event.timestamp * 1000).toLocaleTimeString());
            console.log('---');
        });

        console.log('✅ Callback registered');
        console.log('📝 Now try dragging a file on your desktop!');
        console.log('   (Press Ctrl+C to stop)');

        // Handle cleanup
        process.on('SIGINT', async () => {
            console.log('\n⏹️ Stopping...');
            await dragPlugin.removeDragEventListener(callbackId);
            await dragPlugin.stopDragMonitor();
            console.log('✅ Done');
            process.exit(0);
        });

    } catch (error) {
        console.error('❌ Error:', error.message);
        process.exit(1);
    }
}

main();