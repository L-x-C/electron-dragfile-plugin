const assert = require('assert');
const path = require('path');

// Test with the native binding directly
const nativeModule = require('../index.js');

describe('electron-dragfile-plugin', function() {
  this.timeout(10000);

  before(async function() {
    // Ensure we start with a clean state
    try {
      await nativeModule.stopDragMonitor();
    } catch (e) {
      // Ignore errors, might not be running
    }
  });

  after(async function() {
    // Clean up after tests
    try {
      await nativeModule.stopDragMonitor();
    } catch (e) {
      // Ignore errors
    }
  });

  describe('Basic functionality', function() {
    it('should check monitoring status', async function() {
      const isMonitoring = await nativeModule.isMonitoring();
      assert.strictEqual(typeof isMonitoring, 'boolean');
      assert.strictEqual(isMonitoring, false);
    });

    it('should start and stop monitoring', async function() {
      await nativeModule.startDragMonitor();

      let isMonitoring = await nativeModule.isMonitoring();
      assert.strictEqual(isMonitoring, true);

      await nativeModule.stopDragMonitor();

      isMonitoring = await nativeModule.isMonitoring();
      assert.strictEqual(isMonitoring, false);
    });
  });

  describe('Event handling', function() {
    let callbackId;
    let receivedEvents = [];

    before(async function() {
      await nativeModule.startDragMonitor();
    });

    after(async function() {
      if (callbackId) {
        await nativeModule.removeDragEventListener(callbackId);
      }
      await nativeModule.stopDragMonitor();
    });

    it('should register drag event callback', async function() {
      callbackId = await nativeModule.onDragEvent((event) => {
        receivedEvents.push(event);
      });

      assert.strictEqual(typeof callbackId, 'number');
      assert.ok(callbackId > 0);
    });

    it('should receive simulated drag events', async function() {
      const testFiles = [
        '/path/to/test/file1.txt',
        '/path/to/test/file2.jpg'
      ];

      // Clear previous events
      receivedEvents = [];

      // Simulate drag event
      await nativeModule.simulateDragEvent(testFiles);

      // Give some time for async event handling
      await new Promise(resolve => setTimeout(resolve, 100));

      assert.strictEqual(receivedEvents.length, 1);

      const event = receivedEvents[0];
      assert.ok(Array.isArray(event.files));
      assert.strictEqual(event.files.length, 2);
      assert.strictEqual(event.files[0], testFiles[0]);
      assert.strictEqual(event.files[1], testFiles[1]);
      assert.strictEqual(typeof event.timestamp, 'number');
      assert.ok(event.timestamp > 0);
      assert.strictEqual(event.source, 'test');
    });

    it('should remove drag event callback', async function() {
      const removed = await nativeModule.removeDragEventListener(callbackId);
      assert.strictEqual(removed, true);

      // Try to remove again - should return false
      const removedAgain = await nativeModule.removeDragEventListener(callbackId);
      assert.strictEqual(removedAgain, false);
    });
  });

  describe('DragMonitor class', function() {
    const { DragMonitor } = nativeModule;
    let monitor;
    let receivedEvents = [];

    beforeEach(function() {
      monitor = new DragMonitor();
      receivedEvents = [];
    });

    afterEach(async function() {
      if (monitor) {
        await monitor.stop();
      }
    });

    it('should create a new DragMonitor instance', function() {
      assert.ok(monitor instanceof DragMonitor);
      assert.strictEqual(typeof monitor.start, 'function');
      assert.strictEqual(typeof monitor.stop, 'function');
      assert.strictEqual(typeof monitor.isActive, 'function');
    });

    it('should start and stop monitoring', async function() {
      await monitor.start((event) => {
        receivedEvents.push(event);
      });

      const isActive = await monitor.isActive();
      assert.strictEqual(isActive, true);

      await monitor.stop();

      const isActiveAfterStop = await monitor.isActive();
      assert.strictEqual(isActiveAfterStop, false);
    });

    it('should receive events when started', async function() {
      await monitor.start((event) => {
        receivedEvents.push(event);
      });

      await nativeModule.simulateDragEvent(['/test/file.txt']);

      // Give some time for async event handling
      await new Promise(resolve => setTimeout(resolve, 100));

      assert.strictEqual(receivedEvents.length, 1);
      assert.strictEqual(receivedEvents[0].files[0], '/test/file.txt');
    });

    it('should throw error when starting twice', async function() {
      await monitor.start(() => {});

      let threwError = false;
      try {
        await monitor.start(() => {});
      } catch (error) {
        threwError = true;
        assert.ok(error.message.includes('already started'));
      }

      assert.strictEqual(threwError, true);
    });
  });

  describe('Error handling', function() {
    it('should throw error for invalid callback', async function() {
      let threwError = false;
      try {
        await nativeModule.onDragEvent('not a function');
      } catch (error) {
        threwError = true;
        assert.ok(error instanceof TypeError);
        assert.ok(error.message.includes('function'));
      }

      assert.strictEqual(threwError, true);
    });

    it('should throw error for invalid callback ID', async function() {
      let threwError = false;
      try {
        await nativeModule.removeDragEventListener('not a number');
      } catch (error) {
        threwError = true;
        assert.ok(error instanceof TypeError);
        assert.ok(error.message.includes('number'));
      }

      assert.strictEqual(threwError, true);
    });

    it('should throw error for invalid files array', async function() {
      let threwError = false;
      try {
        await nativeModule.simulateDragEvent('not an array');
      } catch (error) {
        threwError = true;
        assert.ok(error instanceof TypeError);
        assert.ok(error.message.includes('array'));
      }

      assert.strictEqual(threwError, true);
    });

    it('should throw error for invalid file paths', async function() {
      let threwError = false;
      try {
        await nativeModule.simulateDragEvent(['valid/path', 123]);
      } catch (error) {
        threwError = true;
        assert.ok(error instanceof TypeError);
        assert.ok(error.message.includes('strings'));
      }

      assert.strictEqual(threwError, true);
    });
  });
});