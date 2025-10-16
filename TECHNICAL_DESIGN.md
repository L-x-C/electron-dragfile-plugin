# 动态文件拖拽检测技术方案

## 项目概述

本项目实现了一个基于 Rust + Node.js 的动态文件拖拽检测系统，通过创建多个微型窗口来检测文件拖拽事件，解决了传统全屏遮挡方案的用户体验问题。

## 技术架构

### 核心组件

```
┌─────────────────────────────────────────────────────────────────┐
│                    Node.js 应用层                             │
├─────────────────────────────────────────────────────────────────┤
│  mouse-monitoring.js                                           │
│  ├── rdev 事件监听                                              │
│  ├── 坐标跟踪 (LAST_POSITION)                                  │
│  └── 动态窗口管理 (handle_drag_window_management)            │
├─────────────────────────────────────────────────────────────────┤
│                    NAPI 绑定层                                 │
│  ├── lib.rs (Rust 核心逻辑)                                      │
│  ├── MouseEvent/FileDragEvent 数据结构                        │
│  ├── 线程安全回调 (ThreadsafeFunction)                        │
│  └── 进程管理 (spawn/kill helper)                              │
├─────────────────────────────────────────────────────────────────┤
│                   Rust Helper 进程                                │
│  ├── drag-monitor-helper.rs (窗口管理)                         │
│  ├── winit 窗口系统                                            │
│  ├── 5x5 网格布局 (24个 2x2px 窗口)                           │
│  └── 文件拖拽事件检测                                        │
└─────────────────────────────────────────────────────────────────┘
```

## 窗口布局设计

### 5x5 网格布局

```
[LL,TT] [L,TT] [C,TT] [R,TT] [RR,TT]
   ⬜      ⬜      ⬜      ⬜      ⬜

[LL,T]  [L,T]  [C,T]  [R,T]  [RR,T]
   ⬜      ⬜      ⬜      ⬜      ⬜

[LL,M]  [L,M]    ✕     [R,M]  [RR,M]
   ⬜      ⬜          ⬜      ⬜

[LL,B]  [L,B]  [C,B]  [R,B]  [RR,B]
   ⬜      ⬜      ⬜      ⬜      ⬜

[LL,BB] [L,BB] [C,BB] [R,BB] [RR,BB]
   ⬜      ⬜      ⬜      ⬜      ⬜
```

### 布局参数

- **窗口大小**: 2x2 像素
- **窗口数量**: 24个 (5x5 网格去掉中心)
- **窗口间距**: 10像素
- **覆盖范围**: ~42x42像素
- **中心位置**: 无窗口，鼠标操作畅通

## 坐标系统处理

### HiDPI 显示支持

```rust
// rdev 返回逻辑坐标，winit 需要物理坐标
let scaled_mouse_x = mouse_x * scale_factor;
let scaled_mouse_y = mouse_y * scale_factor;

// 窗口位置计算
let window_x = scaled_mouse_x + col as f64 * spacing;
let window_y = scaled_mouse_y + row as f64 * spacing;
```

### 坐标传递链

1. **rdev 捕获**: 原始鼠标事件
2. **坐标跟踪**: `LAST_POSITION` 存储最后坐标
3. **事件分发**: `MouseEvent` 结构传递
4. **进程通信**: 命令行参数传递坐标
5. **窗口定位**: 应用缩放因子并创建窗口

## 动态窗口管理

### 生命周期管理

```rust
// 鼠标按下 → 创建检测窗口
"mousedown" => {
    start_file_drag_monitor_internal(helper_path, mouse_x, mouse_y);
}

// 鼠标释放 → 销毁检测窗口
"mouseup" => {
    stop_file_drag_monitor_internal();
}
```

### 线程安全设计

- **全局状态**: `Arc<Mutex<MonitorState>>`
- **回调管理**: `ThreadsafeFunction<MouseEvent>`
- **进程同步**: stdin/stdout 通信管道

## 数据结构

### MouseEvent (Node.js)

```typescript
interface MouseEvent {
    eventType: string;    // "mousedown", "mouseup", "mousemove", "wheel"
    x: number;          // 鼠标 X 坐标
    y: number;          // 鼠标 Y 坐标
    button: number;     // 按钮编号 (1=左, 2=中, 3=右)
    timestamp: number;  // Unix 时间戳
    platform: string;  // "macos", "windows", "linux"
}
```

### FileDragEvent (Node.js)

```typescript
interface FileDragEvent {
    eventType: string;    // "hovered_file", "dropped_file", "hovered_file_cancelled"
    filePath: string;     // 文件路径
    x: number;          // 事件 X 坐标
    y: number;          // 事件 Y 坐标
    timestamp: number;  // 时间戳
    platform: string;  // 平台信息
    windowId: string;   // 窗口 ID (兼容性字段)
}
```

## 边界处理策略

### 智能边界检查

```rust
// 边界限制
let max_x = (monitor_size.width - window_width) as f64;
let max_y = (monitor_size.height - window_height) as f64;

// 位置调整
let final_x = window_x.max(0.0).min(max_x);
let final_y = window_y.max(0.0).min(max_y);
```

### 边界调整日志

```
⚠️  Window [-2,-2] adjusted from (-10, -10) to (0, 0)
⚠️  Window [+2,+2] adjusted from (1910, 1080) to (1918, 1078)
✅ Grid calculation complete: 24 windows, 2 boundary adjustments
```

## 平台兼容性

### macOS
- **权限要求**: Accessibility 权限用于全局鼠标监听
- **框架链接**: CoreFoundation, CoreGraphics, Cocoa, AppKit, Foundation
- **最低版本**: macOS 10.13
- **窗口特性**: 透明度控制、顶层窗口、无边框

### Windows
- **权限要求**: 无特殊权限
- **最低版本**: Windows 10+
- **窗口特性**: 标准窗口系统

### Linux
- **显示系统**: X11
- **架构支持**: glibc 和 musl 二进制兼容
- **窗口特性**: 标准 X11 窗口

## 性能优化

### 内存管理
- **预分配容量**: `Vec::with_capacity(24)`
- **RAII 模式**: 自动资源清理
- **线程池**: 避免频繁线程创建

### 窗口优化
- **微型窗口**: 2x2px 最小化视觉影响
- **延迟激活**: 只在鼠标按下时创建
- **批量创建**: 一次性创建所有窗口

### 事件处理
- **异步回调**: 非阻塞事件处理
- **事件过滤**: 只处理相关事件类型
- **智能分发**: 避免不必要的事件传递

## 调试功能

### 详细日志输出

```
=== COMMAND LINE DEBUG ===
Total arguments received: 3
Arg 0: './target/release/drag-monitor-helper'
Arg 1: '444.37109375'
Arg 2: '755.640625'
✓ Successfully parsed mouse coordinates: (444.37109375, 755.640625)

=== WINDOW CREATION DEBUG ===
Primary monitor size: 2880x1800
Monitor scale factor: 2
🎯 5x5 GRID WINDOW MODE - Using mouse coordinates: (444.37109375, 755.640625)
🎯 SCALE FACTOR FIX DETECTED!
Scaled physical coordinates: (888.7421875, 1511.28125)

📐 Calculating 5x5 grid positions with 10px spacing
Grid layout (no center window):
  [LLTT] (868.7421875, 1491.28125) ⬜
  [LTT] (878.7421875, 1491.28125) ⬜
  [CTT] (888.7421875, 1491.28125) ⬜
  [RTT] (898.7421875, 1491.28125) ⬜
  [RRTT] (908.7421875, 1491.28125) ⬜
  ...
```

## 使用方法

### 启动监听

```javascript
const {
    startMouseMonitor,
    onMouseEvent,
    onFileDragEvent,
    startFileDragMonitor
} = require('./index');

// 启动鼠标监听
await startMouseMonitor();

// 注册回调
const mouseCallbackId = await onMouseEvent((err, event) => {
    if (event.eventType === 'mousedown') {
        console.log(`Mouse down at (${event.x}, ${event.y})`);
    }
});

const dragCallbackId = await onFileDragEvent((err, event) => {
    if (event.eventType === 'dropped_file') {
        console.log(`File dropped: ${event.filePath}`);
    }
});

// 配置拖拽检测
await startFileDragMonitor('./target/release/drag-monitor-helper');
```

### 事件处理

```javascript
// 清理资源
process.on('SIGINT', async () => {
    await removeMouseEventListener(mouseCallbackId);
    await removeFileDragEventListener(dragCallbackId);
    await stopMouseMonitor();
    await stopFileDragMonitor();
    process.exit(0);
});
```

## 构建和部署

### 构建命令

```bash
# 构建所有平台
npm run build

# 构建当前平台
npm run build:simple

# 生成构建产物
npm run artifacts
```

### 项目结构

```
electron-dragfile-plugin/
├── src/
│   ├── lib.rs                    # 核心 Rust 逻辑
│   └── bin/
│       └── drag-monitor-helper.rs # 窗口管理程序
├── target/
│   └── release/
│       └── drag-monitor-helper   # 编译后的可执行文件
├── index.js                     # NAPI 生成的 Node.js 绑定
├── index.d.ts                   # TypeScript 类型定义
├── test-dynamic-drag.js          # 测试文件
└── package.json                 # 项目配置
```

## 技术优势

1. **无遮挡检测**: 不影响用户正常操作
2. **高精度定位**: 10px 间距，减少遗漏
3. **跨平台兼容**: 支持 Windows、macOS、Linux
4. **HiDPI 支持**: 自动处理高分辨率显示
5. **动态管理**: 按需创建/销毁检测窗口
6. **线程安全**: 多线程环境下的安全操作
7. **资源优化**: 最小化内存和 CPU 使用

## 应用场景

- **文件管理器**: 增强拖拽功能
- **开发工具**: IDE 插件拖拽支持
- **设计软件**: 素材拖拽检测
- **办公软件**: 文档拖拽增强
- **游戏引擎**: 资源导入工具

## 故障排除

### 常见问题

1. **macOS 权限问题**
   ```bash
   # 确保应用有 Accessibility 权限
   # 系统偏好设置 → 安全性与隐私 → 隐私 → 辅助功能
   ```

2. **窗口创建失败**
   - 检查 helper 程序是否存在
   - 确认构建成功
   - 查看错误日志

3. **坐标不准确**
   - 检查缩放因子
   - 验证显示器设置
   - 查看调试日志

### 调试技巧

1. **启用详细日志**: 查看坐标传递过程
2. **单窗口测试**: 直接运行 helper 程序
3. **边界测试**: 在屏幕边缘测试
4. **性能监控**: 监控 CPU 和内存使用

---

*技术方案版本: v1.0*
*最后更新: 2025-10-16*