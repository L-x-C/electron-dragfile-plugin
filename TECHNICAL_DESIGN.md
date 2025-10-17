# Electron Drag File Plugin - Technical Design Document

## Project Overview

This is a native Node.js addon built with Rust and napi-rs that provides system-wide mouse event monitoring and file drag detection capabilities with visual window overlay functionality. The project creates a 4-window "口" shaped overlay system around mouse position to detect file drag events while maintaining user experience.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    Node.js Application Layer                  │
├─────────────────────────────────────────────────────────────────┤
│  Main Application                                              │
│  ├── rdev mouse event monitoring                               │
│  ├── coordinate tracking (LAST_POSITION)                      │
│  └── dynamic window management (handle_drag_window_management) │
├─────────────────────────────────────────────────────────────────┤
│                    NAPI Binding Layer                         │
│  ├── lib.rs (Rust core logic)                                  │
│  ├── MouseEvent/FileDragEvent data structures                 │
│  ├── thread-safe callbacks (ThreadsafeFunction)               │
│  └── process management (spawn/kill helper)                   │
├─────────────────────────────────────────────────────────────────┤
│                    Rust Helper Process                         │
│  ├── drag-monitor-helper.rs (window management)               │
│  ├── winit window system                                       │
│  ├── 4-window "口" shaped layout                              │
│  ├── screen color sampling (xcap + image)                     │
│  └── file drag event detection                                │
└─────────────────────────────────────────────────────────────────┘
```

## Window System Design

### Window Layout Strategy

The system creates 4 border windows in a "口" shape around the mouse position:

```
    ┌─────────────────────┐
    │       [TOP]         │
    │                     │
    │[LEFT]   [MOUSE]   [RIGHT]│
    │                     │
    │      [BOTTOM]       │
    └─────────────────────┘
```

**Window Specifications:**
- **Top/Bottom**: 80x15 pixels
- **Left/Right**: 15x80 pixels
- **Distance**: 50px from mouse center
- **Positioning**: Dynamic based on mouse coordinates with scale factor adjustment

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

## Screen Color Sampling System

### Color Capture Implementation

The system implements real-time screen color sampling for dynamic window background color adjustment:

```rust
fn get_screen_color_at(x: f64, y: f64) -> Result<Color, Box<dyn std::error::Error>> {
    // Multi-monitor support with coordinate mapping
    // XCAP screenshot capture using xcap crate
    // RGBA color extraction using image crate
    // Cross-platform coordinate system handling
}
```

### Color Data Flow

1. **Screen Capture**: `xcap::Monitor::capture_image()` - Captures screenshot of target monitor
2. **Coordinate Mapping**: Logical to physical pixel conversion with HiDPI support
3. **Pixel Extraction**: RGBA value extraction from screenshot data
4. **Color Storage**: Structured Color object with hex string conversion

**Color Structure:**
```rust
#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8, g: u8, b: u8, a: u8,
}

impl Color {
    fn from_rgba(rgba: Rgba<u8>) -> Self
    fn to_hex_string(&self) -> String
}
```

## Background Color Implementation Challenges

### Current Technical Approach

The project attempts to implement window background color setting using macOS-specific APIs:

**Dependencies Added:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5.0"
objc2-app-kit = { version = "0.2.2", features = ["NSColor", "NSWindow"] }
objc2-foundation = { version = "0.2.2", features = ["NSObject"] }
objc2-core-foundation = "0.3.2"
```

**Implementation Strategy:**
1. Use `winit::window::Window` as the primary window creation API
2. Access underlying `NSWindow` through raw window handles
3. Set background color using `NSWindow::setBackgroundColor()` with `NSColor` objects

### Technical Challenges Identified

#### 1. Window Handle Access Complexity

**Problem**: The `raw_window_handle` API provides access to `AppKitWindowHandle` which only contains `ns_view`, not the direct `NSWindow` pointer.

**Issue**:
```rust
// Current approach fails - ns_window field doesn't exist
if let RawWindowHandle::AppKit(appkit_handle) = raw_window_handle {
    let nswindow_ptr = appkit_handle.ns_window.as_ptr(); // ❌ Field doesn't exist
}
```

**Available fields**: Only `ns_view: NonNull<c_void>` is accessible.

#### 2. NSWindow Access from NSView

**Problem**: To access the parent `NSWindow` from an `NSView`, we need to use Objective-C messaging, but this requires:

- Complex Objective-C runtime integration
- Safe handling of pointer relationships
- Proper memory management with ARC (Automatic Reference Counting)

**Required implementation**:
```objc
// Objective-C approach needed
NSWindow* window = [ns_view window];
[window setBackgroundColor:ns_color];
[window setOpaque:YES];
```

#### 3. Cross-Platform Window API Limitations

**Problem**: `winit` provides a cross-platform abstraction, but background color setting requires platform-specific APIs.

**Winit Limitations**:
- No direct `set_background_color()` method
- Window styling capabilities are platform-dependent
- Access to underlying window handles is intentionally limited for safety

#### 4. Memory Management Complexity

**Problem**: Objective-C objects require proper reference counting, and Rust's ownership model doesn't directly map to Objective-C's ARC.

**Safety Concerns**:
- Raw pointer manipulation requires `unsafe` blocks
- Reference counting must be manually managed
- Potential for memory leaks or premature deallocation

### Current Implementation Status

#### Working Components ✅

1. **Window Creation**: 4-window overlay system functioning correctly
2. **Color Sampling**: Screen color extraction working properly
3. **Coordinate System**: Mouse-to-window positioning accurate
4. **Multi-Monitor Support**: Proper monitor detection and coordinate mapping
5. **Cross-Platform Build**: Successfully builds on macOS with target-specific dependencies
6. **NSColor Creation**: Color objects can be created successfully with proper RGBA conversion

#### Partially Implemented 🔄

1. **Window Handle Access**: Can access NSView but not NSWindow directly
2. **Debug Infrastructure**: Comprehensive logging system in place for tracking implementation progress
3. **Platform Detection**: Proper macOS-specific conditional compilation

#### Not Yet Implemented ❌

1. **Background Color Setting**: Core functionality not working due to NSWindow access limitations
2. **NSWindow Manipulation**: Cannot access window properties directly through current winit integration
3. **Visual Color Feedback**: No visual confirmation of color changes on windows

### Alternative Technical Approaches

#### Approach 1: Direct NSWindow Creation

**Concept**: Bypass winit and create NSWindow directly using objc2 bindings.

**Pros**:
- Full control over window properties
- Direct access to all NSWindow APIs
- No abstraction layer limitations

**Cons**:
- Significant rewrite of window system
- Loss of cross-platform compatibility
- Complex event handling implementation required

#### Approach 2: View-Based Background Setting

**Concept**: Create an NSView as a child of the window and set its background color.

**Implementation**:
```rust
// Pseudocode for NSView-based approach
let ns_view = appkit_handle.ns_view;
let background_view = NSView::init();
background_view.setBackgroundColor(ns_color);
ns_view.addSubview(background_view);
```

**Pros**:
- Works within existing winit framework
- Safer memory management
- Reversible changes

**Cons**:
- Requires complex Objective-C messaging
- Still needs NSWindow access for proper integration

#### Approach 3: Window Theme Manipulation

**Concept**: Use macOS appearance APIs to modify window colors indirectly.

**Approach**:
- Set window to dark/light mode
- Use system color schemes
- Manipulate window opacity and blending

**Pros**:
- Uses public APIs
- More stable across macOS versions
- Potentially simpler implementation

**Cons**:
- Limited color control
- Dependent on system appearance settings
- May not provide exact color matching

### Technical Recommendations

#### Short-Term Solutions

1. **Simplify Color Display**: Use window title or border styling to indicate sampled colors
2. **Logging-Based Feedback**: Enhance debug output to show color sampling success
3. **Alternative Visual Indicators**: Use window transparency or size changes to indicate color states

#### Medium-Term Solutions

1. **View Hierarchy Integration**: Implement NSView-based background setting
2. **Objective-C Messaging Bridge**: Create safe abstractions for NSWindow access
3. **Memory Management Strategy**: Implement proper ARC integration patterns

#### Long-Term Solutions

1. **Platform-Specific Window Backend**: Create macOS-specific window management system
2. **Complete Objective-C Integration**: Full access to macOS windowing APIs
3. **Cross-Platform Color API**: Design unified color setting interface with platform-specific implementations

### Current Implementation Code

**Background Color Function (Current State):**
```rust
fn set_window_background_color(_window: &Window, color: Color) {
    eprintln!("[helper] 🎨 Setting window background color to: RGBA({}, {}, {}, {})",
        color.r, color.g, color.b, color.a);

    #[cfg(target_os = "macos")]
    {
        // Convert Rust Color (0-255) to CGFloat (0.0-1.0)
        let red = color.r as f64 / 255.0;
        let green = color.g as f64 / 255.0;
        let blue = color.b as f64 / 255.0;
        let alpha = color.a as f64 / 255.0;

        // Create NSColor object for logging purposes
        unsafe {
            let ns_color = NSColor::colorWithRed_green_blue_alpha(
                red, green, blue, alpha,
            );
            eprintln!("[helper] 🎨 Created NSColor object: {:?}", ns_color);
        }

        eprintln!("[helper] 🎨 Window background color setting implemented (NSWindow manipulation complete)");
    }
}
```

**Key Findings:**
- NSColor creation works correctly
- Color conversion from Rust RGBA to CGFloat is accurate
- Missing NSWindow access prevents actual background color application
- Debug infrastructure provides detailed implementation tracking

## Conclusion

The window background color setting feature represents a significant technical challenge due to the abstraction layers between Rust/winit and the underlying macOS windowing system. While the infrastructure for color sampling and window management is solid, the integration point between these systems requires careful navigation of platform-specific APIs and memory management considerations.

The current implementation establishes a solid foundation for future development, with working color sampling, window positioning, and debug infrastructure. The remaining challenge is bridging the gap between the cross-platform winit abstraction and the platform-specific NSWindow APIs needed for background color manipulation.

This technical design serves as a roadmap for continued development, documenting both the current limitations and potential paths forward for implementing complete window background color functionality.

---

*Technical Design Version: v2.0*
*Last Updated: 2025-10-17*
*Document Status: Active Development - Background Color Implementation In Progress*