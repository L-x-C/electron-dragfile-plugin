use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi_derive::napi;
use rdev::{listen, Event, EventType, Button, Key};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

// region: Mouse and Keyboard Monitoring (统一事件监听系统)

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
pub struct KeyboardEvent {
    pub event_type: String,
    pub key_code: i32,
    pub key_name: String,
    pub modifiers: Vec<String>,
    pub timestamp: f64,
    pub platform: String,
}

struct UnifiedMonitorState {
    is_monitoring: bool,
    mouse_callbacks: HashMap<u32, ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled>>,
    keyboard_callbacks: HashMap<u32, ThreadsafeFunction<KeyboardEvent, ErrorStrategy::CalleeHandled>>,
    next_callback_id: u32,
    shutdown_sender: Option<std::sync::mpsc::Sender<()>>,
    monitor_handle: Option<thread::JoinHandle<()>>,
}

impl UnifiedMonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            mouse_callbacks: HashMap::new(),
            keyboard_callbacks: HashMap::new(),
            next_callback_id: 0,
            shutdown_sender: None,
            monitor_handle: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref UNIFIED_STATE: Arc<Mutex<UnifiedMonitorState>> = Arc::new(Mutex::new(UnifiedMonitorState::new()));
    static ref LAST_POSITION: Arc<Mutex<Option<(f64, f64)>>> = Arc::new(Mutex::new(None));
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
        _ => {
            None
        }
    }
}

fn convert_rdev_keyboard_event(event: &Event) -> Option<KeyboardEvent> {
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
        EventType::KeyPress(key) => {
            let (key_code, key_name) = convert_key_to_code_and_name(&key);
            let modifiers = extract_modifiers(&event);

            Some(KeyboardEvent {
                event_type: "keydown".to_string(),
                key_code,
                key_name,
                modifiers,
                timestamp,
                platform: platform.to_string(),
            })
        }
        EventType::KeyRelease(key) => {
            let (key_code, key_name) = convert_key_to_code_and_name(&key);
            let modifiers = extract_modifiers(&event);

            Some(KeyboardEvent {
                event_type: "keyup".to_string(),
                key_code,
                key_name,
                modifiers,
                timestamp,
                platform: platform.to_string(),
            })
        }
        _ => {
            None
        }
    }
}

fn convert_key_to_code_and_name(key: &Key) -> (i32, String) {
    match key {
        Key::Alt => (18, "Alt".to_string()),
        Key::AltGr => (225, "AltGr".to_string()),
        Key::Backspace => (8, "Backspace".to_string()),
        Key::CapsLock => (20, "CapsLock".to_string()),
        Key::ControlLeft => (17, "ControlLeft".to_string()),
        Key::ControlRight => (17, "ControlRight".to_string()),
        Key::Delete => (46, "Delete".to_string()),
        Key::DownArrow => (40, "DownArrow".to_string()),
        Key::End => (35, "End".to_string()),
        Key::Escape => (27, "Escape".to_string()),
        Key::F1 => (112, "F1".to_string()),
        Key::F10 => (121, "F10".to_string()),
        Key::F11 => (122, "F11".to_string()),
        Key::F12 => (123, "F12".to_string()),
        Key::F2 => (113, "F2".to_string()),
        Key::F3 => (114, "F3".to_string()),
        Key::F4 => (115, "F4".to_string()),
        Key::F5 => (116, "F5".to_string()),
        Key::F6 => (117, "F6".to_string()),
        Key::F7 => (118, "F7".to_string()),
        Key::F8 => (119, "F8".to_string()),
        Key::F9 => (120, "F9".to_string()),
        Key::Home => (36, "Home".to_string()),
        Key::LeftArrow => (37, "LeftArrow".to_string()),
        Key::MetaLeft => (91, "MetaLeft".to_string()),
        Key::MetaRight => (91, "MetaRight".to_string()),
        Key::PageDown => (34, "PageDown".to_string()),
        Key::PageUp => (33, "PageUp".to_string()),
        Key::Return => (13, "Return".to_string()),
        Key::RightArrow => (39, "RightArrow".to_string()),
        Key::ShiftLeft => (16, "ShiftLeft".to_string()),
        Key::ShiftRight => (16, "ShiftRight".to_string()),
        Key::Space => (32, "Space".to_string()),
        Key::Tab => (9, "Tab".to_string()),
        Key::UpArrow => (38, "UpArrow".to_string()),
        Key::NumLock => (144, "NumLock".to_string()),
        Key::PrintScreen => (154, "PrintScreen".to_string()),
        Key::ScrollLock => (145, "ScrollLock".to_string()),
        Key::Pause => (19, "Pause".to_string()),
        Key::Insert => (45, "Insert".to_string()),
        Key::KpMultiply => (106, "Multiply".to_string()),
        Key::KpDivide => (111, "Divide".to_string()),
        Key::KeyA => (65, "A".to_string()),
        Key::KeyB => (66, "B".to_string()),
        Key::KeyC => (67, "C".to_string()),
        Key::KeyD => (68, "D".to_string()),
        Key::KeyE => (69, "E".to_string()),
        Key::KeyF => (70, "F".to_string()),
        Key::KeyG => (71, "G".to_string()),
        Key::KeyH => (72, "H".to_string()),
        Key::KeyI => (73, "I".to_string()),
        Key::KeyJ => (74, "J".to_string()),
        Key::KeyK => (75, "K".to_string()),
        Key::KeyL => (76, "L".to_string()),
        Key::KeyM => (77, "M".to_string()),
        Key::KeyN => (78, "N".to_string()),
        Key::KeyO => (79, "O".to_string()),
        Key::KeyP => (80, "P".to_string()),
        Key::KeyQ => (81, "Q".to_string()),
        Key::KeyR => (82, "R".to_string()),
        Key::KeyS => (83, "S".to_string()),
        Key::KeyT => (84, "T".to_string()),
        Key::KeyU => (85, "U".to_string()),
        Key::KeyV => (86, "V".to_string()),
        Key::KeyW => (87, "W".to_string()),
        Key::KeyX => (88, "X".to_string()),
        Key::KeyY => (89, "Y".to_string()),
        Key::KeyZ => (90, "Z".to_string()),
        Key::Unknown(code) => (*code as i32, format!("Unknown({})", code)),
        _ => (0, format!("{:?}", key)),
    }
}

fn extract_modifiers(event: &Event) -> Vec<String> {
    let mut modifiers = Vec::new();

    // Since rdev doesn't provide direct modifier information,
    // we'll check if certain modifier keys are being pressed
    // This is a simplified approach - in a real implementation,
    // you might want to track modifier state separately
    match event.event_type {
        EventType::KeyPress(key) | EventType::KeyRelease(key) => {
            match key {
                Key::ShiftLeft | Key::ShiftRight => modifiers.push("shift".to_string()),
                Key::ControlLeft | Key::ControlRight => modifiers.push("control".to_string()),
                Key::Alt | Key::AltGr => modifiers.push("alt".to_string()),
                Key::MetaLeft | Key::MetaRight => modifiers.push("meta".to_string()),
                _ => {}
            }
        }
        _ => {}
    }

    modifiers
}

fn trigger_mouse_event(mouse_event: MouseEvent) {
    if let Ok(state) = UNIFIED_STATE.lock() {
        for callback in state.mouse_callbacks.values() {
            callback.call(Ok(mouse_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

fn trigger_keyboard_event(keyboard_event: KeyboardEvent) {
    if let Ok(state) = UNIFIED_STATE.lock() {
        for callback in state.keyboard_callbacks.values() {
            callback.call(Ok(keyboard_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

// 统一的事件监听函数，同时处理鼠标和键盘事件
fn unified_event_listener() -> impl FnMut(Event) {
    move |event: Event| {
        // 首先尝试作为鼠标事件处理
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
            trigger_mouse_event(mouse_event);
        }
        // 如果不是鼠标事件，尝试作为键盘事件处理
        else if let Some(keyboard_event) = convert_rdev_keyboard_event(&event) {
            trigger_keyboard_event(keyboard_event);
        }
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

// Keyboard API functions
#[napi]
pub fn start_keyboard_monitor() -> Result<()> {
    start_unified_monitor()
}

#[napi]
pub fn stop_keyboard_monitor() -> Result<()> {
    stop_unified_monitor()
}

#[napi]
pub fn on_keyboard_event(callback: JsFunction) -> Result<u32> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    let id = state.next_callback_id + 1;
    state.next_callback_id = id;
    let tsfn: ThreadsafeFunction<KeyboardEvent, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    state.keyboard_callbacks.insert(id, tsfn);
    Ok(id)
}

#[napi]
pub fn remove_keyboard_event_listener(id: u32) -> Result<bool> {
    let mut state = UNIFIED_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire unified monitor state lock"))?;
    Ok(state.keyboard_callbacks.remove(&id).is_some())
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

