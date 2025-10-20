use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi_derive::napi;
use rdev::{listen, Event, EventType, Button};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

// region: Mouse Monitoring (保留原有鼠标监听功能)

#[napi(object)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseEvent {
    pub event_type: String,
    pub x: f64,
    pub y: f64,
    pub button: i32,
    pub timestamp: f64,
    pub platform: String,
}

struct MonitorState {
    is_monitoring: bool,
    callbacks: HashMap<u32, ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled>>,
    next_callback_id: u32,
    shutdown_sender: Option<std::sync::mpsc::Sender<()>>,
    monitor_handle: Option<thread::JoinHandle<()>>,
}

impl MonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            callbacks: HashMap::new(),
            next_callback_id: 0,
            shutdown_sender: None,
            monitor_handle: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref MONITOR_STATE: Arc<Mutex<MonitorState>> = Arc::new(Mutex::new(MonitorState::new()));
    static ref LAST_POSITION: Arc<Mutex<Option<(f64, f64)>>> = Arc::new(Mutex::new(None));
}

fn convert_rdev_event(event: Event) -> Option<MouseEvent> {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let timestamp = event.time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    match event.event_type {
        EventType::ButtonPress(button) => {
            let button_num = match button {
                Button::Left => 1,
                Button::Middle => 2,
                Button::Right => 3,
                Button::Unknown(b) => b as i32,
            };

            Some(MouseEvent {
                event_type: "mousedown".to_string(),
                x: 0.0, // Will be updated with actual coordinates
                y: 0.0, // Will be updated with actual coordinates
                button: button_num,
                timestamp,
                platform: platform.to_string(),
            })
        }
        EventType::ButtonRelease(button) => {
            let button_num = match button {
                Button::Left => 1,
                Button::Middle => 2,
                Button::Right => 3,
                Button::Unknown(b) => b as i32,
            };

            Some(MouseEvent {
                event_type: "mouseup".to_string(),
                x: 0.0, // Will be updated with actual coordinates
                y: 0.0, // Will be updated with actual coordinates
                button: button_num,
                timestamp,
                platform: platform.to_string(),
            })
        }
        EventType::MouseMove { x, y } => {
            Some(MouseEvent {
                event_type: "mousemove".to_string(),
                x,
                y,
                button: 0,
                timestamp,
                platform: platform.to_string(),
            })
        }
        EventType::Wheel { delta_x: _, delta_y: _ } => {
            Some(MouseEvent {
                event_type: "wheel".to_string(),
                x: 0.0,
                y: 0.0,
                button: 0,
                timestamp,
                platform: platform.to_string(),
            })
        }
        _ => {
            None
        }
    }
}

fn trigger_mouse_event(mouse_event: MouseEvent) {
    if let Ok(state) = MONITOR_STATE.lock() {
        for callback in state.callbacks.values() {
            callback.call(Ok(mouse_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

#[napi]
pub fn start_mouse_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire monitor state lock"))?;
    if state.is_monitoring { return Ok(()); }
    let (shutdown_sender, _shutdown_receiver) = std::sync::mpsc::channel::<()>();
    state.shutdown_sender = Some(shutdown_sender);
    let handle = thread::spawn(move || {
        let callback = move |event: Event| {
            if let Some(mut mouse_event) = convert_rdev_event(event) {
                if mouse_event.event_type != "mousemove" {
                    if let Some((x, y)) = LAST_POSITION.lock().ok().and_then(|p| *p) {
                        mouse_event.x = x;
                        mouse_event.y = y;
                    }
                } else {
                    if let Ok(mut pos) = LAST_POSITION.lock() {
                        *pos = Some((mouse_event.x, mouse_event.y));
                    }
                }
                trigger_mouse_event(mouse_event);
            }
        };
        if let Err(error) = listen(callback) {
            eprintln!("Error listening to mouse events: {:?}", error);
        }
    });
    state.monitor_handle = Some(handle);
    state.is_monitoring = true;
    Ok(())
}

#[napi]
pub fn stop_mouse_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire monitor state lock"))?;
    if !state.is_monitoring { return Ok(()); }
    if let Some(sender) = state.shutdown_sender.take() { let _ = sender.send(()); }
    if let Some(handle) = state.monitor_handle.take() { let _ = handle.join(); }
    state.is_monitoring = false;
    Ok(())
}

#[napi]
pub fn on_mouse_event(callback: JsFunction) -> Result<u32> {
    let mut state = MONITOR_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire monitor state lock"))?;
    let id = state.next_callback_id + 1;
    state.next_callback_id = id;
    let tsfn: ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    state.callbacks.insert(id, tsfn);
    Ok(id)
}

#[napi]
pub fn remove_mouse_event_listener(id: u32) -> Result<bool> {
    let mut state = MONITOR_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire monitor state lock"))?;
    Ok(state.callbacks.remove(&id).is_some())
}

#[napi]
pub fn is_monitoring() -> bool {
    MONITOR_STATE.lock().unwrap().is_monitoring
}

// endregion

// region: File Drag Monitoring (新的基于NSPasteboard的实现)

#[napi(object)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDragEvent {
    pub event_type: String,
    pub file_path: String,
    pub x: f64,
    pub y: f64,
    pub timestamp: f64,
    pub platform: String,
    pub window_id: String, // Kept for API compatibility, will be empty
}

struct FileDragMonitorState {
    is_monitoring: bool,
    callbacks: Arc<Mutex<HashMap<u32, ThreadsafeFunction<FileDragEvent, ErrorStrategy::CalleeHandled>>>>,
    next_callback_id: u32,
    drag_monitor_handle: Option<thread::JoinHandle<()>>,
    shutdown_sender: Option<std::sync::mpsc::Sender<()>>,
}

impl FileDragMonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            next_callback_id: 0,
            drag_monitor_handle: None,
            shutdown_sender: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref FILE_DRAG_STATE: Mutex<FileDragMonitorState> = Mutex::new(FileDragMonitorState::new());
}

// 检查拖拽粘贴板是否包含文件
#[cfg(target_os = "macos")]
fn check_drag_pasteboard_for_files() -> bool {
    use objc2_foundation::NSString;
    use objc2_app_kit::NSPasteboard;

    eprintln!("[macOS] DEBUG: Checking drag pasteboard for files...");

    // 获取拖拽粘贴板 - 使用 .drag 名称
    let drag_pasteboard = unsafe {
        let pasteboard_name = NSString::from_str("NSDragPboard");
        let pb = NSPasteboard::pasteboardWithName(&pasteboard_name);
        eprintln!("[macOS] DEBUG: Got drag pasteboard: {:p}", pb);
        pb
    };

    // 检查粘贴板是否包含文件URL类型 - 根据Stack Overflow答案使用 .fileURL
    let file_url_type = NSString::from_str("public.file-url");
    eprintln!("[macOS] DEBUG: Looking for file URL type: {}", file_url_type);

    unsafe {
        let types = drag_pasteboard.types();
        eprintln!("[macOS] DEBUG: Pasteboard types: {:?}", types.as_ref().map(|t| {
            // 尝试获取一些类型信息
            format!("{:p} (count: {})", t, t.len())
        }));

        if let Some(types_array) = types {
            let has_file_url = types_array.containsObject(&file_url_type);
            eprintln!("[macOS] DEBUG: Contains file URL: {}", has_file_url);
            return has_file_url;
        } else {
            eprintln!("[macOS] DEBUG: No types array found");
        }
    }

    false
}

// 触发文件拖拽事件
fn trigger_file_drag_event(event: FileDragEvent) {
    let callbacks = {
        let state = FILE_DRAG_STATE.lock().unwrap();
        Arc::clone(&state.callbacks)
    };

    if let Ok(cbs) = callbacks.lock() {
        for callback in cbs.values() {
            callback.call(Ok(event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    };
}

// macOS拖拽监控线程 - 基于 NSEvent 和 NSRunLoop 的正确实现
#[cfg(target_os = "macos")]
fn start_macos_drag_monitor_thread(shutdown_receiver: std::sync::mpsc::Receiver<()>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        use objc2_app_kit::{NSEvent, NSEventMask, NSApplication};
        use objc2_foundation::{NSAutoreleasePool, MainThreadMarker};
        use block2::StackBlock;
        use std::ptr::NonNull;
        use std::sync::atomic::{AtomicBool, Ordering};
        use dispatch::Queue;

        eprintln!("[macOS] Drag monitoring thread started (NSRunLoop-based implementation)");

        // The thread that calls `app.run()` becomes the main AppKit thread.
        // We need a separate thread to listen for the shutdown signal and dispatch `app.stop()`.
        let shutdown_handle = {
            let main_queue = Queue::main();
            thread::spawn(move || {
                // Block until the shutdown signal is received from `stop_file_drag_monitor`
                let _ = shutdown_receiver.recv();
                eprintln!("[macOS] Shutdown signal received. Dispatching stop to main AppKit queue.");

                // Queue the `stop` command on the main AppKit thread to gracefully exit the run loop.
                main_queue.exec_async(move || {
                    eprintln!("[macOS] Executing stop command on main AppKit queue.");
                    let app = unsafe { NSApplication::sharedApplication(MainThreadMarker::new_unchecked()) };
                    app.stop(None);
                });
            })
        };

        // An autorelease pool is required for Cocoa API calls.
        let _pool = unsafe { NSAutoreleasePool::new() };

        // Setup the NSApplication instance.
        let app = unsafe { NSApplication::sharedApplication(MainThreadMarker::new_unchecked()) };
        // Set activation policy to `Accessory` to prevent the app from appearing in the Dock or forcing focus.
        // unsafe { app.setActivationPolicy(NSApplicationActivationPolicy::Accessory) }; // Commented out to fix build issues

        let is_dragging = AtomicBool::new(false);

        // Create global mouse event monitors.
        let (drag_monitor, move_monitor, up_monitor) = unsafe {
            let is_dragging_ptr = &is_dragging as *const AtomicBool;

            // Monitor for LeftMouseDragged events
            let drag_block = StackBlock::new(move |event: NonNull<NSEvent>| {
                if check_drag_pasteboard_for_files() {
                    if !(*is_dragging_ptr).load(Ordering::Relaxed) {
                        (*is_dragging_ptr).store(true, Ordering::Relaxed);
                        eprintln!("[macOS] File drag detected");

                        let event_ref = event.as_ref();
                        let location = event_ref.locationInWindow();
                        let drag_event = FileDragEvent {
                            event_type: "hovered_file".to_string(),
                            file_path: "".to_string(),
                            x: location.x as f64,
                            y: location.y as f64,
                            timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
                            platform: "macos".to_string(),
                            window_id: "".to_string(),
                        };
                        trigger_file_drag_event(drag_event);
                    }
                }
            });

            // Monitor for MouseMoved events (to test for permissions)
            let move_block = StackBlock::new(|_event: NonNull<NSEvent>| {
                eprintln!("[macOS] DEBUG: Mouse moved - permissions are working!");
            });

            // Monitor for LeftMouseUp events (to detect drag end)
            let up_block = StackBlock::new(move |_event: NonNull<NSEvent>| {
                if (*is_dragging_ptr).load(Ordering::Relaxed) {
                    (*is_dragging_ptr).store(false, Ordering::Relaxed);
                    eprintln!("[macOS] Drag ended");

                    let drag_event = FileDragEvent {
                        event_type: "hovered_file_cancelled".to_string(),
                        file_path: "".to_string(),
                        x: 0.0,
                        y: 0.0,
                        timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
                        platform: "macos".to_string(),
                        window_id: "".to_string(),
                    };
                    trigger_file_drag_event(drag_event);
                }
            });

            eprintln!("[macOS] DEBUG: Setting up global event monitors...");
            let drag_monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseDragged, &drag_block);
            let move_monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::MouseMoved, &move_block);
            let up_monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseUp, &up_block);
            eprintln!("[macOS] DEBUG: Monitors created: drag={:?}, move={:?}, up={:?}", drag_monitor.is_some(), move_monitor.is_some(), up_monitor.is_some());

            (drag_monitor, move_monitor, up_monitor)
        };

        eprintln!("[macOS] Starting AppKit run loop. This will block until stop is called...");
        unsafe { app.run() }; // This blocks the thread and processes events until `app.stop()` is called.
        eprintln!("[macOS] AppKit run loop finished.");

        // Cleanup: remove the monitors.
        if let Some(monitor) = drag_monitor { unsafe { NSEvent::removeMonitor(&monitor) } }
        if let Some(monitor) = move_monitor { unsafe { NSEvent::removeMonitor(&monitor) } }
        if let Some(monitor) = up_monitor { unsafe { NSEvent::removeMonitor(&monitor) } }
        eprintln!("[macOS] Event monitors removed.");

        // Wait for the shutdown helper thread to complete.
        shutdown_handle.join().expect("Shutdown handle thread failed");
        eprintln!("[macOS] Drag monitoring thread fully stopped.");
    })
}

// 非macOS平台的占位符实现
#[cfg(not(target_os = "macos"))]
fn start_macos_drag_monitor_thread(_shutdown_receiver: std::sync::mpsc::Receiver<()>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        eprintln!("[warning] File drag monitoring is only supported on macOS");
    })
}

// 内部启动文件拖拽监控的函数
fn start_file_drag_monitor_internal() -> Result<()> {
    let mut state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;

    if state.is_monitoring {
        return Ok(());
    }

    let (shutdown_sender, shutdown_receiver) = std::sync::mpsc::channel::<()>();
    state.shutdown_sender = Some(shutdown_sender);

    let handle = start_macos_drag_monitor_thread(shutdown_receiver);
    state.drag_monitor_handle = Some(handle);
    state.is_monitoring = true;

    eprintln!("[main] File drag monitoring started using NSPasteboard detection");
    Ok(())
}

// 内部停止文件拖拽监控的函数
fn stop_file_drag_monitor_internal() -> Result<()> {
    let mut state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;

    if !state.is_monitoring {
        return Ok(());
    }

    if let Some(sender) = state.shutdown_sender.take() {
        let _ = sender.send(());
    }

    if let Some(handle) = state.drag_monitor_handle.take() {
        let _ = handle.join();
    }

    state.is_monitoring = false;
    eprintln!("[main] File drag monitoring stopped");
    Ok(())
}

#[napi]
pub fn start_file_drag_monitor(_helper_path: String) -> Result<()> {
    // _helper_path 参数为了保持API兼容性，但新的实现不需要它
    eprintln!("[main] Starting file drag monitoring (NSPasteboard-based implementation)");
    start_file_drag_monitor_internal()
}

#[napi]
pub fn stop_file_drag_monitor() -> Result<()> {
    eprintln!("[main] Stopping file drag monitoring");
    stop_file_drag_monitor_internal()
}

#[napi]
pub fn on_file_drag_event(callback: JsFunction) -> Result<u32> {
    let (id, callbacks_arc) = {
        let mut state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;
        let id = state.next_callback_id + 1;
        state.next_callback_id = id;
        (id, Arc::clone(&state.callbacks))
    };

    let tsfn: ThreadsafeFunction<FileDragEvent, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

    let mut callbacks = callbacks_arc.lock().unwrap();
    callbacks.insert(id, tsfn);

    Ok(id)
}

#[napi]
pub fn remove_file_drag_event_listener(id: u32) -> Result<bool> {
    let state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;
    let mut callbacks = state.callbacks.lock().unwrap();
    Ok(callbacks.remove(&id).is_some())
}

#[napi]
pub fn is_file_drag_monitoring() -> bool {
    FILE_DRAG_STATE.lock().unwrap().is_monitoring
}

// endregion