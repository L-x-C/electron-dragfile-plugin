use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi_derive::napi;
use rdev::{listen, Event, EventType, Button};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Mouse event data structure for Node.js
#[napi(object)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseEvent {
    /// Type of mouse event: "mousedown", "mouseup", "mousemove"
    pub event_type: String,
    /// Mouse X coordinate
    pub x: f64,
    /// Mouse Y coordinate
    pub y: f64,
    /// Mouse button: 0=no button, 1=left, 2=middle, 3=right
    pub button: i32,
    /// Timestamp when the event occurred
    pub timestamp: f64,
    /// Platform information
    pub platform: String,
}

/// Global state for mouse monitoring
struct MonitorState {
    is_monitoring: bool,
    callbacks: HashMap<u32, ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled>>,
    next_callback_id: u32,
    shutdown_sender: Option<mpsc::Sender<()>>,
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

/// Convert rdev EventType to our mouse event format
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
            // Include wheel events as well
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
            // Ignore keyboard events
            None
        }
    }
}

/// Trigger mouse event callbacks
fn trigger_mouse_event(mouse_event: MouseEvent) {
    if let Ok(state) = MONITOR_STATE.lock() {
        // Clone callbacks to avoid borrowing issues
        let callbacks: Vec<_> = state.callbacks.iter().map(|(id, callback)| (*id, callback.clone())).collect();

        // Trigger callbacks with event
        for (_callback_id, callback) in callbacks {
            let event_clone = mouse_event.clone();
            callback.call(Ok(event_clone), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

/// Get current mouse position
fn get_mouse_position() -> Option<(f64, f64)> {
    LAST_POSITION.lock().ok()?.clone()
}

/// Set current mouse position
fn set_mouse_position(x: f64, y: f64) {
    if let Ok(mut pos) = LAST_POSITION.lock() {
        *pos = Some((x, y));
    }
}

/// Start monitoring mouse events globally
#[napi]
pub fn start_mouse_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Failed to acquire monitor state lock"
        )
    })?;

    if state.is_monitoring {
        return Ok(());
    }

    // Create a channel for shutdown communication
    let (shutdown_sender, _shutdown_receiver) = mpsc::channel::<()>();
    state.shutdown_sender = Some(shutdown_sender);

    // Start monitoring in a separate thread
    let handle = thread::spawn(move || {
        println!("🖱️ Starting mouse event monitoring...");

        let callback = move |event: Event| {
            if let Some(mut mouse_event) = convert_rdev_event(event) {
                // Update coordinates for button events using last known position
                if mouse_event.event_type != "mousemove" {
                    if let Some((x, y)) = get_mouse_position() {
                        mouse_event.x = x;
                        mouse_event.y = y;
                    }
                } else {
                    // Store the position for button events
                    set_mouse_position(mouse_event.x, mouse_event.y);
                }

                trigger_mouse_event(mouse_event);
            }
        };

        // Start the actual event listening
        if let Err(error) = listen(callback) {
            println!("Error listening to mouse events: {:?}", error);
        }
    });

    state.monitor_handle = Some(handle);
    state.is_monitoring = true;
    println!("✅ Mouse monitoring started");
    Ok(())
}

/// Stop monitoring mouse events
#[napi]
pub fn stop_mouse_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Failed to acquire monitor state lock"
        )
    })?;

    if !state.is_monitoring {
        return Ok(());
    }

    // Send shutdown signal if we have a sender
    if let Some(sender) = state.shutdown_sender.take() {
        let _ = sender.send(());
    }

    // Wait for the monitor thread to finish
    if let Some(handle) = state.monitor_handle.take() {
        let _ = handle.join();
    }

    state.is_monitoring = false;
    println!("✅ Mouse monitoring stopped");
    Ok(())
}

/// Register a callback for mouse events
#[napi]
pub fn on_mouse_event(callback: JsFunction) -> Result<u32> {
    let mut state = MONITOR_STATE.lock().map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Failed to acquire monitor state lock"
        )
    })?;

    let callback_id = state.next_callback_id + 1; // Start from 1, not 0
    state.next_callback_id = callback_id;

    // Create a threadsafe function from the JavaScript callback
    let threadsafe_callback: ThreadsafeFunction<MouseEvent, ErrorStrategy::CalleeHandled> = callback
        .create_threadsafe_function(0, |ctx| {
            Ok(vec![ctx.value])
        })
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to create threadsafe function: {}", e)
            )
        })?;

    state.callbacks.insert(callback_id, threadsafe_callback);

    Ok(callback_id)
}

/// Remove a mouse event callback
#[napi]
pub fn remove_mouse_event_listener(callback_id: u32) -> Result<bool> {
    let mut state = MONITOR_STATE.lock().map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Failed to acquire monitor state lock"
        )
    })?;

    if state.callbacks.remove(&callback_id).is_some() {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check if mouse monitoring is currently active
#[napi]
pub fn is_monitoring() -> bool {
    let state = MONITOR_STATE.lock().unwrap();
    state.is_monitoring
}

