#!/usr/bin/env node

const path = require('path');

// Load the plugin
let mousePlugin;
try {
    const localPath = path.join(__dirname, 'index.js');
    if (require('fs').existsSync(localPath)) {
        mousePlugin = require(localPath);
    } else {
        mousePlugin = require('electron-dragfile-plugin');
    }
    console.log('✅ Plugin loaded');
} catch (error) {
    console.error('❌ Failed to load plugin:', error.message);
    console.error('Run: npm run build');
    process.exit(1);
}

async function main() {
    try {
        console.log('🚀 Starting mouse and keyboard event monitoring...');

        // Start mouse monitoring
        await mousePlugin.startMouseMonitor();
        console.log('✅ Mouse monitoring started');

        // Start keyboard monitoring
        await mousePlugin.startKeyboardMonitor();
        console.log('✅ Keyboard monitoring started');

        // Register mouse callback
        const mouseCallbackId = await mousePlugin.onMouseEvent((err, event) => {
            if (err) {
                console.error('Error in mouse event callback:', err);
                return;
            }
            if (!event) {
                console.warn('Received a null mouse event.');
                return;
            }
            const buttonName = event.button === 0 ? 'None' :
                event.button === 1 ? 'Left' :
                event.button === 2 ? 'Middle' :
                event.button === 3 ? 'Right' : `Button ${event.button}`;

            console.log(`🖱️ ${event.eventType.toUpperCase()} at (${event.x.toFixed(2)}, ${event.y.toFixed(2)}) - ${buttonName}`);
            console.log(`   Platform: ${event.platform}`);
            console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
            console.log('---');
        });

        // Register keyboard callback
        const keyboardCallbackId = await mousePlugin.onKeyboardEvent((err, event) => {
            if (err) {
                console.error('Error in keyboard event callback:', err);
                return;
            }
            if (!event) {
                console.warn('Received a null keyboard event.');
                return;
            }

            const modifiers = event.modifiers.length > 0 ? event.modifiers.join('+') + '+' : '';
            console.log(`⌨️ ${event.eventType.toUpperCase()}: ${modifiers}${event.keyName} (code: ${event.keyCode})`);
            console.log(`   Platform: ${event.platform}`);
            console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
            console.log('---');
        });

        console.log('✅ Mouse and keyboard event callbacks registered');
        console.log('📝 Move your mouse, click, and press keys to see events!');
        console.log('   (Press Ctrl+C to stop)');

        // Handle cleanup
        process.on('SIGINT', async () => {
            console.log('\n⏹️ Stopping monitors...');
            await mousePlugin.removeMouseEventListener(mouseCallbackId);
            await mousePlugin.removeKeyboardEventListener(keyboardCallbackId);
            await mousePlugin.stopMouseMonitor();
            await mousePlugin.stopKeyboardMonitor();
            console.log('✅ Done');
            process.exit(0);
        });

    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error('Stack:', error.stack);
        process.exit(1);
    }
}

main();