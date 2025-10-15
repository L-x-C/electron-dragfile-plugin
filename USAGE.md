# Electron Drag File Plugin - 使用指南

## 概述

这个插件提供系统级的文件拖拽监听功能，可以检测用户在系统任何地方拖拽文件的操作，而不仅限于你的应用窗口内。

## 主要特性

- 🎯 **全局监听**: 监听整个系统的文件拖拽事件
- 🔄 **实时检测**: 鼠标按下文件并移动时即可检测
- 🍎 **macOS 支持**: 使用 Core Graphics API 和 NSPasteboard
- 🪟 **Windows 支持**: 使用 Windows API 和 OLE/COM
- 🔧 **Node.js 兼容**: 可在纯 Node.js 环境中测试
- ⚡ **异步回调**: 非阻塞的事件处理

## 安装和构建

### 1. 安装依赖
```bash
npm install
```

### 2. 构建原生插件
```bash
npm run build
```

### 3. 开发模式构建（调试版本）
```bash
npm run build:debug
```

## 使用方法

### 基本用法

```javascript
const dragPlugin = require('electron-dragfile-plugin');

// 1. 开始监听
await dragPlugin.startDragMonitor();

// 2. 注册事件回调
const callbackId = await dragPlugin.onDragEvent((event) => {
    console.log('检测到文件拖拽:', event);
    console.log('文件列表:', event.files);
    console.log('时间戳:', event.timestamp);
    console.log('来源:', event.source);
});

// 3. 停止监听
await dragPlugin.stopDragMonitor();

// 4. 移除回调
await dragPlugin.removeDragEventListener(callbackId);
```

### 在 Electron 中使用

主进程 (main.js):
```javascript
const { app, BrowserWindow, ipcMain } = require('electron');
const dragPlugin = require('electron-dragfile-plugin');

async function initializeDragMonitoring() {
    try {
        await dragPlugin.startDragMonitor();

        const callbackId = await dragPlugin.onDragEvent((event) => {
            // 发送事件到渲染进程
            if (mainWindow && !mainWindow.isDestroyed()) {
                mainWindow.webContents.send('drag-event', event);
            }
        });

        console.log('拖拽监听已启动，回调ID:', callbackId);
    } catch (error) {
        console.error('启动拖拽监听失败:', error);
    }
}

app.whenReady().then(() => {
    createWindow();
    initializeDragMonitoring();
});
```

预加载脚本 (preload.js):
```javascript
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronDragAPI', {
    startDragMonitor: () => ipcRenderer.invoke('drag-monitor:start'),
    stopDragMonitor: () => ipcRenderer.invoke('drag-monitor:stop'),
    getMonitoringStatus: () => ipcRenderer.invoke('drag-monitor:status'),
    simulateDragEvent: (files) => ipcRenderer.invoke('drag-monitor:simulate', files),
    onDragEvent: (callback) => {
        ipcRenderer.on('drag-event', (event, data) => callback(data));
    },
    removeDragEventListener: () => {
        ipcRenderer.removeAllListeners('drag-event');
    }
});
```

渲染进程:
```javascript
// 开始监听
await window.electronDragAPI.startDragMonitor();

// 监听拖拽事件
window.electronDragAPI.onDragEvent((event) => {
    console.log('文件拖拽事件:', event);
    // 在这里处理拖拽的文件
    event.files.forEach(file => {
        console.log('拖拽的文件:', file);
    });
});
```

## API 参考

### 函数

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `startDragMonitor()` | 无 | `Promise<void>` | 开始监听全局拖拽事件 |
| `stopDragMonitor()` | 无 | `Promise<void>` | 停止监听 |
| `onDragEvent(callback)` | `function` | `Promise<number>` | 注册事件回调，返回回调ID |
| `removeDragEventListener(id)` | `number` | `Promise<boolean>` | 移除事件回调 |
| `isMonitoring()` | 无 | `Promise<boolean>` | 检查监听状态 |
| `simulateDragEvent(files)` | `string[]` | `Promise<void>` | 模拟拖拽事件（测试用） |

### DragEvent 对象

```typescript
interface DragEvent {
    files: string[];        // 被拖拽的文件路径列表
    timestamp: number;      // 事件时间戳（Unix 时间戳，秒）
    source?: string;        // 事件来源（"system" 或 "test"）
}
```

## 权限要求

### macOS

在 macOS 上，插件需要**辅助功能权限**才能监听全局鼠标事件：

1. 打开 **系统偏好设置** > **安全性与隐私** > **隐私**
2. 选择 **辅助功能**
3. 点击 🔒 锁并解锁
4. 添加你的应用程序或终端应用到列表中
5. 重启你的应用程序

### Windows

在 Windows 上，插件可能需要管理员权限来设置全局钩子：

- 以管理员身份运行应用程序
- 确保安全软件没有阻止钩子操作

## 测试

### 使用测试脚本

项目包含一个简单的 Node.js 测试脚本：

```bash
node test-drag.js
```

这个脚本会：
1. 测试基本功能
2. 测试错误处理
3. 启动实时监听（30秒）
4. 在测试期间尝试拖拽文件来验证功能

### 在 Electron 示例中测试

```bash
cd example
npm install
npm start
```

## 故障排除

### 常见问题

1. **"Failed to create event tap" (macOS)**
   - 确保已授予辅助功能权限
   - 重启应用程序

2. **"Failed to set mouse hook" (Windows)**
   - 以管理员身份运行
   - 检查安全软件设置

3. **没有收到拖拽事件**
   - 确保监听已启动
   - 检查回调是否正确注册
   - 验证系统权限

4. **构建失败**
   - 确保 Rust 已安装 (`rustup --version`)
   - 清理并重新构建 (`rm -rf target && npm run build`)

### 调试模式

使用调试版本进行开发：

```bash
npm run build:debug
```

启用详细日志：
```javascript
// 在启动监听前设置环境变量
process.env.DEBUG = 'electron-dragfile-plugin:*';
```

## 技术细节

### macOS 实现
- 使用 `CGEvent.tapCreate` 监听全局鼠标事件
- 通过 `NSPasteboard` 检测拖拽的文件内容
- 在后台线程中运行 `CFRunLoop`

### Windows 实现
- 使用 `SetWindowsHookEx` 设置全局鼠标钩子
- 通过剪贴板 API 检测拖拽文件
- 在消息循环中处理事件

### 回调机制
- 使用 `ThreadsafeFunction` 确保跨线程安全
- 异步事件处理，不阻塞主线程
- 错误恢复和资源清理

## 许可证

MIT License