use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi_derive::napi;
use rdev::{listen, Event, EventType, Button};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

// region: Mouse Event Monitoring (鼠标事件监听系统)

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


#[napi(object)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DragEvent {
    pub event_type: String,
    pub x: f64,
    pub y: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub button: i32,
    pub timestamp: f64,
    pub platform: String,
}

struct UnifiedMonitorState {
    is_monitoring: bool,
    mouse_callbacks: HashMap<u32, ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled>>,
    drag_callbacks: HashMap<u32, ThreadsafeFunction<DragEvent, ErrorStrategy::CalleeHandled>>,
    next_callback_id: u32,
    shutdown_sender: Option<std::sync::mpsc::Sender<()>>,
    monitor_handle: Option<thread::JoinHandle<()>>,
    // Drag state
    is_dragging: bool,
    drag_start_position: Option<(f64, f64)>,
    drag_button: Option<i32>,
    // Distance threshold detection
    mouse_pressed: bool,
    potential_drag_start: Option<(f64, f64)>,
    drag_threshold: f64,
}

impl UnifiedMonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            mouse_callbacks: HashMap::new(),
            drag_callbacks: HashMap::new(),
            next_callback_id: 0,
            shutdown_sender: None,
            monitor_handle: None,
            // Drag state
            is_dragging: false,
            drag_start_position: None,
            drag_button: None,
            // Distance threshold detection
            mouse_pressed: false,
            potential_drag_start: None,
            drag_threshold: 5.0, // 5 pixels threshold
        }
    }
}

lazy_static::lazy_static! {
    static ref UNIFIED_STATE: Arc<Mutex<UnifiedMonitorState>> = Arc::new(Mutex::new(UnifiedMonitorState::new()));
    static ref LAST_POSITION: Arc<Mutex<Option<(f64, f64)>>> = Arc::new(Mutex::new(None));
}

// 重置拖拽状态的辅助函数
fn reset_drag_state(state: &mut std::sync::MutexGuard<'_, UnifiedMonitorState>) {
    state.mouse_pressed = false;
    state.is_dragging = false;
    state.potential_drag_start = None;
    state.drag_start_position = None;
    state.drag_button = None;
}

fn convert_rdev_mouse_event(event: &Event) -> Option<MouseEvent> {
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
    }
}


fn trigger_mouse_event(mouse_event: MouseEvent) {
    if let Ok(state) = UNIFIED_STATE.lock() {
        for callback in state.mouse_callbacks.values() {
            callback.call(Ok(mouse_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}


fn trigger_drag_event(drag_event: DragEvent) {
    if let Ok(state) = UNIFIED_STATE.lock() {
        for callback in state.drag_callbacks.values() {
            callback.call(Ok(drag_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

// 统一的事件监听函数，只处理鼠标事件
fn unified_event_listener() -> impl FnMut(Event) {
    move |event: Event| {
        // 尝试作为鼠标事件处理
        if let Some(mut mouse_event) = convert_rdev_mouse_event(&event) {
            // 处理鼠标事件的坐标
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

            // 拖拽状态检测逻辑
            if let Ok(mut state) = UNIFIED_STATE.lock() {
                match mouse_event.event_type.as_str() {
                    "mousedown" => {
                        // 记录鼠标按下状态，但不立即开始拖拽
                        state.mouse_pressed = true;
                        state.potential_drag_start = Some((mouse_event.x, mouse_event.y));
                        state.drag_button = Some(mouse_event.button);
                        // 不触发 dragstart 事件，等待移动距离超过阈值
                    }
                    "mousemove" => {
                        if state.mouse_pressed {
                            if let Some((start_x, start_y)) = state.potential_drag_start {
                                // 计算移动距离
                                let delta_x = mouse_event.x - start_x;
                                let delta_y = mouse_event.y - start_y;
                                let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

                                if distance >= state.drag_threshold {
                                    if !state.is_dragging {
                                        // 首次超过阈值，开始拖拽
                                        state.is_dragging = true;
                                        state.drag_start_position = Some((start_x, start_y));

                                        // 触发拖拽开始事件
                                        let drag_event = DragEvent {
                                            event_type: "dragstart".to_string(),
                                            x: mouse_event.x,
                                            y: mouse_event.y,
                                            start_x: start_x,
                                            start_y: start_y,
                                            button: state.drag_button.unwrap_or(0),
                                            timestamp: mouse_event.timestamp,
                                            platform: mouse_event.platform.clone(),
                                        };
                                        drop(state); // 释放锁
                                        trigger_drag_event(drag_event);
                                    } else {
                                        // 已经在拖拽中，触发拖拽移动事件
                                        let drag_event = DragEvent {
                                            event_type: "dragmove".to_string(),
                                            x: mouse_event.x,
                                            y: mouse_event.y,
                                            start_x,
                                            start_y,
                                            button: state.drag_button.unwrap_or(0),
                                            timestamp: mouse_event.timestamp,
                                            platform: mouse_event.platform.clone(),
                                        };
                                        drop(state); // 释放锁
                                        trigger_drag_event(drag_event);
                                    }
                                } else {
                                    // 距离未超过阈值，不触发事件
                                    drop(state); // 释放锁
                                }
                            } else {
                                drop(state); // 释放锁
                            }
                        } else {
                            drop(state); // 释放锁
                        }
                    }
                    "mouseup" => {
                        if state.mouse_pressed {
                            if state.is_dragging {
                                // 正在拖拽中，触发拖拽结束事件
                                // is_dragging 为 true 时，drag_start_position 应该总是有值
                                if let Some((start_x, start_y)) = state.drag_start_position {
                                    let drag_event = DragEvent {
                                        event_type: "dragend".to_string(),
                                        x: mouse_event.x,
                                        y: mouse_event.y,
                                        start_x,
                                        start_y,
                                        button: state.drag_button.unwrap_or(0),
                                        timestamp: mouse_event.timestamp,
                                        platform: mouse_event.platform.clone(),
                                    };
                                    reset_drag_state(&mut state);
                                    drop(state); // 释放锁
                                    trigger_drag_event(drag_event);
                                }
                            } else {
                                // 无论是否开始拖拽，都重置所有状态
                                reset_drag_state(&mut state);
                                drop(state); // 释放锁
                            }
                        } else {
                            drop(state); // 释放锁
                        }
                    }
                    _ => {
                        // 其他鼠标事件，不处理拖拽
                        drop(state); // 释放锁
                    }
                }
            }

            trigger_mouse_event(mouse_event);
        }
        // 忽略所有非鼠标事件
    }
}

// Mouse API functions
#[napi]
pub fn start_mouse_monitor() -> Result<()> {
    start_unified_monitor()
}

#[napi]
pub fn stop_mouse_monitor() -> Result<()> {
    stop_unified_monitor()
}

#[napi]
pub fn on_mouse_event(callback: JsFunction) -> Result<u32> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    let id = state.next_callback_id + 1;
    state.next_callback_id = id;
    let tsfn: ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    state.mouse_callbacks.insert(id, tsfn);
    Ok(id)
}

#[napi]
pub fn remove_mouse_event_listener(id: u32) -> Result<bool> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    Ok(state.mouse_callbacks.remove(&id).is_some())
}


// Drag API functions
#[napi]
pub fn on_drag_event(callback: JsFunction) -> Result<u32> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    let id = state.next_callback_id + 1;
    state.next_callback_id = id;
    let tsfn: ThreadsafeFunction<DragEvent, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    state.drag_callbacks.insert(id, tsfn);
    Ok(id)
}

#[napi]
pub fn remove_drag_event_listener(id: u32) -> Result<bool> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    Ok(state.drag_callbacks.remove(&id).is_some())
}

// Unified monitoring functions
fn start_unified_monitor() -> Result<()> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;

    if state.is_monitoring {
        return Ok(());
    }

    let (shutdown_sender, _shutdown_receiver) = std::sync::mpsc::channel::<()>();
    state.shutdown_sender = Some(shutdown_sender);

    let handle = thread::spawn(move || {
        let callback = unified_event_listener();
        if let Err(error) = listen(callback) {
            eprintln!("Error listening to input events: {:?}", error);
        }
    });

    state.monitor_handle = Some(handle);
    state.is_monitoring = true;
    Ok(())
}

fn stop_unified_monitor() -> Result<()> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;

    if !state.is_monitoring {
        return Ok(());
    }

    if let Some(sender) = state.shutdown_sender.take() {
        let _ = sender.send(());
    }

    if let Some(handle) = state.monitor_handle.take() {
        let _ = handle.join();
    }

    state.is_monitoring = false;
    Ok(())
}

#[napi]
pub fn is_monitoring() -> bool {
    UNIFIED_STATE.lock().unwrap().is_monitoring
}

// endregion

