# electron-dragfile-plugin

A high-performance native Node.js addon built with Rust and napi-rs that monitors system-wide mouse and drag events. Perfect for applications that need to track mouse movements, clicks, and drag operations across the entire system.

## ✨ Features

- 🚀 **High Performance**: Built with Rust for maximum performance and low overhead
- 🌍 **Cross-Platform**: Supports macOS, Windows, and Linux
- 📡 **System-Wide Detection**: Monitors mouse and drag events across the entire system, not just your app
- 🖱️ **Complete Mouse Tracking**: Tracks mouse movements, clicks, and wheel events
- 🔄 **Smart Drag Detection**: Intelligent drag event detection with distance threshold to avoid false triggers
- 🔧 **Easy to Use**: Simple JavaScript API with TypeScript support
- 📦 **NPM Ready**: Published to npm for easy installation
- 🎯 **Universal**: Works with any Node.js application, not just Electron

## 📦 Installation

```bash
npm install electron-dragfile-plugin
```

## 🚀 Quick Start

```javascript
const {
  startMouseMonitor,
  onMouseEvent,
  onDragEvent
} = require('electron-dragfile-plugin');

// Start monitoring mouse and drag events
await startMouseMonitor();

// Listen for mouse events
onMouseEvent((err, event) => {
  if (err) {
    console.error('Error:', err);
    return;
  }

  const buttonName = event.button === 0 ? 'None' :
    event.button === 1 ? 'Left' :
    event.button === 2 ? 'Middle' :
    event.button === 3 ? 'Right' : `Button ${event.button}`;

  console.log(`🖱️ ${event.eventType.toUpperCase()} at (${event.x.toFixed(2)}, ${event.y.toFixed(2)}) - ${buttonName}`);
  console.log(`   Platform: ${event.platform}`);
  console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
});

// Listen for drag events
onDragEvent((err, event) => {
  if (err) {
    console.error('Error:', err);
    return;
  }

  const buttonName = event.button === 0 ? 'None' :
    event.button === 1 ? 'Left' :
    event.button === 2 ? 'Middle' :
    event.button === 3 ? 'Right' : `Button ${event.button}`;

  const deltaX = event.x - event.startX;
  const deltaY = event.y - event.startY;
  const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

  console.log(`🔄 ${event.eventType.toUpperCase()} at (${event.x.toFixed(2)}, ${event.y.toFixed(2)}) - ${buttonName}`);
  console.log(`   Start position: (${event.startX.toFixed(2)}, ${event.startY.toFixed(2)})`);
  if (distance > 0.1) {
    console.log(`   Distance: (${deltaX.toFixed(2)}, ${deltaY.toFixed(2)}) total: ${distance.toFixed(2)}px`);
  }
  console.log(`   Platform: ${event.platform}`);
  console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
});
```

## 📖 API Reference

### Mouse Event Functions

#### `startMouseMonitor(): Promise<void>`
Start monitoring mouse events globally.

#### `stopMouseMonitor(): Promise<void>`
Stop monitoring mouse events.

#### `onMouseEvent(callback: Function): Promise<number>`
Register a callback for mouse events. Returns a callback ID.

#### `removeMouseEventListener(callbackId: number): Promise<boolean>`
Remove a mouse event callback using the returned ID.

### Drag Event Functions

#### `onDragEvent(callback: Function): Promise<number>`
Register a callback for drag events. Returns a callback ID.

#### `removeDragEventListener(callbackId: number): Promise<boolean>`
Remove a drag event callback using the returned ID.

### Status Functions

#### `isMonitoring(): Promise<boolean>`
Check if mouse monitoring is currently active.

### MouseEvent Interface

```typescript
interface MouseEvent {
  eventType: string;      // Event type: "mousedown", "mouseup", "mousemove", "wheel"
  x: number;             // Mouse X coordinate
  y: number;             // Mouse Y coordinate
  button: number;        // Mouse button: 0=no button, 1=left, 2=middle, 3=right
  timestamp: number;     // Unix timestamp of the event
  platform: string;     // Platform information: "macos", "windows", "linux"
}
```

### DragEvent Interface

```typescript
interface DragEvent {
  eventType: string;      // Event type: "dragstart", "dragmove", "dragend"
  x: number;             // Current mouse X coordinate
  y: number;             // Current mouse Y coordinate
  startX: number;        // Drag start X coordinate
  startY: number;        // Drag start Y coordinate
  button: number;        // Mouse button used for drag: 0=none, 1=left, 2=middle, 3=right
  timestamp: number;     // Unix timestamp of the event
  platform: string;      // Platform information: "macos", "windows", "linux"
}
```

**Smart Drag Detection**: The drag events use intelligent detection with a distance threshold (default 5px) to avoid false triggers from simple clicks or accidental mouse movements. Drag events are only triggered when the mouse is pressed and moved beyond the threshold distance.

## 🎯 Application Integration

Here's how to integrate with your Node.js application:

### Basic Usage - Mouse and Drag Events

```javascript
const {
  startMouseMonitor,
  onMouseEvent,
  onDragEvent,
  stopMouseMonitor
} = require('electron-dragfile-plugin');

async function setupInputTracking() {
  try {
    // Start monitoring
    await startMouseMonitor();
    console.log('✅ Mouse and drag monitoring started');

    // Register mouse callback
    const mouseCallbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('Mouse event error:', err);
        return;
      }

      console.log(`🖱️ ${event.eventType} at (${event.x}, ${event.y})`);
      // Handle mouse events here
    });

    // Register drag callback
    const dragCallbackId = await onDragEvent((err, event) => {
      if (err) {
        console.error('Drag event error:', err);
        return;
      }

      const deltaX = event.x - event.startX;
      const deltaY = event.y - event.startY;
      const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

      console.log(`🔄 ${event.eventType} - moved ${distance.toFixed(1)}px from (${event.startX}, ${event.startY}) to (${event.x}, ${event.y})`);
      // Handle drag events here
    });

    console.log('✅ Callbacks registered - Mouse ID:', mouseCallbackId, 'Drag ID:', dragCallbackId);

    // Later, you can stop monitoring
    // await stopMouseMonitor();

  } catch (error) {
    console.error('❌ Failed to setup input monitoring:', error);
  }
}

setupInputTracking();
```

### Error Handling

```javascript
const { startMouseMonitor, onMouseEvent } = require('electron-dragfile-plugin');

async function robustMouseTracking() {
  try {
    await startMouseMonitor();

    const callbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('Callback error:', err);
        return;
      }

      if (!event) {
        console.warn('Received null event');
        return;
      }

      // Process event safely
      processMouseEvent(event);
    });

  } catch (setupError) {
    console.error('Setup failed:', setupError);
  }
}

function processMouseEvent(event) {
  try {
    // Your event processing logic here
    console.log(`Mouse event: ${event.eventType} at (${event.x}, ${event.y})`);
  } catch (processingError) {
    console.error('Event processing error:', processingError);
  }
}
```

## 🧪 Testing

```bash
# Install dependencies
npm install

# Build the native addon
npm run build

# Run the test script
node test.js
```

The test script will start mouse monitoring and log all events to the console.

**Test Instructions:**
1. Run `node test.js`
2. Move your mouse around - you'll see mouse movement events
3. Click different mouse buttons - you'll see click events with coordinates
4. **Test drag events**: Press and hold mouse button, then move (more than 5px) - you'll see dragstart, dragmove, and dragend events
5. Try simple clicks without moving - notice no drag events are triggered (smart detection)
6. Press Ctrl+C to stop the test

**Example Output:**
```
🖱️ MOUSEMOVE at (245.67, 189.23) - None
   Platform: macos
   Time: 14:30:25
---
🔄 DRAGSTART at (150.00, 200.00) - Left
   Start position: (150.00, 200.00)
   Platform: macos
   Time: 14:30:30
---
🔄 DRAGMOVE at (160.50, 215.25) - Left
   Start position: (150.00, 200.00)
   Distance: (10.50, 15.25) total: 18.47px
   Platform: macos
   Time: 14:30:31
---
🔄 DRAGEND at (175.00, 230.00) - Left
   Start position: (150.00, 200.00)
   Distance: (25.00, 30.00) total: 39.05px
   Platform: macos
   Time: 14:30:32
---
```

## 🔧 Platform Requirements

### macOS
- Requires macOS 10.14 or later
- May need to grant Accessibility permissions for global mouse monitoring
  - Go to System Preferences → Security & Privacy → Privacy → Accessibility
  - Add your terminal or Node.js application to the list

### Windows
- Requires Windows 10 or later
- No additional permissions required

### Linux
- Requires X11 display server
- No additional permissions required

## 🏗️ Development

### Prerequisites

- Node.js 14+
- Rust 1.70+

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/electron-dragfile-plugin.git
cd electron-dragfile-plugin

# Install dependencies
npm install

# Build the native addon
npm run build

# Run tests
npm test
```

### Project Structure

```
electron-dragfile-plugin/
├── src/
│   └── lib.rs              # Rust native code
├── index.js                # Node.js entry point
├── index.d.ts              # TypeScript definitions
├── Cargo.toml              # Rust project config
├── package.json            # NPM package config
├── test.js                 # Test script
└── README.md               # This file
```

## 🔧 Platform Support

| Platform | Status | Binary |
|----------|--------|--------|
| Windows 10+ (x64) | ✅ Supported | electron-dragfile-plugin.win32-x64-msvc.node |
| macOS 10.14+ (Intel) | ✅ Supported | electron-dragfile-plugin.darwin-x64.node |
| macOS 11+ (Apple Silicon) | ✅ Supported | electron-dragfile-plugin.darwin-arm64.node |
| Linux (x64) | ✅ Supported | electron-dragfile-plugin.linux-x64-gnu.node |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [napi-rs](https://napi.rs/) for high-performance native addons
- Uses [rdev](https://github.com/narsil/rdev) for cross-platform mouse event monitoring
- Platform-specific APIs for system integration

## 📞 Support

If you encounter any issues or have questions:

- [GitHub Issues](https://github.com/yourusername/electron-dragfile-plugin/issues)
- [Discussions](https://github.com/yourusername/electron-dragfile-plugin/discussions)

---

Made with ❤️ for the Node.js community