use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi::NapiRaw;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::ptr;

/// Simple drag event data structure
#[napi(object)]
#[derive(Debug, Clone)]
pub struct DragEvent {
    /// Array of file paths being dragged
    pub files: Vec<String>,
    /// Timestamp when the drag event occurred
    pub timestamp: f64,
    /// Mouse coordinates when drag was detected
    pub x: f64,
    pub y: f64,
    /// Platform information
    pub platform: String,
}

/// Global state for drag monitoring
struct MonitorState {
    is_monitoring: bool,
    callbacks: HashMap<u32, ThreadsafeFunction<DragEvent, ErrorStrategy::CalleeHandled>>,
    next_callback_id: u32,
}

impl MonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            callbacks: HashMap::new(),
            next_callback_id: 0,
        }
    }
}

lazy_static::lazy_static! {
    static ref MONITOR_STATE: Arc<Mutex<MonitorState>> = Arc::new(Mutex::new(MonitorState::new()));
}

/// Trigger drag event callbacks
fn trigger_drag_event(files: Vec<String>, x: f64, y: f64, platform: &str) {
    let event = DragEvent {
        files: files.clone(),
        x,
        y,
        platform: platform.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    };

    if let Ok(state) = MONITOR_STATE.lock() {
        // Clone callbacks to avoid borrowing issues
        let callbacks: Vec<_> = state.callbacks.iter().map(|(id, callback)| (*id, callback.clone())).collect();

        // Trigger callbacks with event
        for (callback_id, callback) in callbacks {
            let event_clone = event.clone();

            tokio::spawn(async move {
                let status = callback.call(Ok(event_clone), ThreadsafeFunctionCallMode::Blocking);
                if status != napi::Status::Ok {
                    eprintln!("Error calling drag event callback (ID: {}), status: {:?}", callback_id, status);
                }
            });
        }
    }
}

/// Start monitoring drag events globally
#[napi]
pub fn start_drag_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().unwrap();

    if state.is_monitoring {
        return Ok(());
    }

    // Simple monitoring without platform-specific code for now
    state.is_monitoring = true;
    println!("✅ Drag monitoring started (basic mode - no system integration)");
    Ok(())
}

/// Stop monitoring drag events
#[napi]
pub fn stop_drag_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().unwrap();

    if !state.is_monitoring {
        return Ok(());
    }

    state.is_monitoring = false;
    println!("✅ Drag monitoring stopped");
    Ok(())
}

/// Register a callback for drag events
#[napi]
pub fn on_drag_event(callback: JsFunction) -> Result<u32> {
    // Validate the callback function
    if unsafe { callback.raw() } == ptr::null_mut() {
        return Err(Error::new(
            Status::InvalidArg,
            "Callback function is null or invalid"
        ));
    }

    let mut state = MONITOR_STATE.lock().map_err(|_| {
        Error::new(
            Status::GenericFailure,
            "Failed to acquire monitor state lock"
        )
    })?;

    let callback_id = state.next_callback_id + 1; // Start from 1, not 0
    state.next_callback_id = callback_id;

    // Create a threadsafe function from the JavaScript callback
    let threadsafe_callback: ThreadsafeFunction<DragEvent, ErrorStrategy::CalleeHandled> = callback
        .create_threadsafe_function(0, |ctx| {
            Ok(vec![ctx.value])
        }).map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to create threadsafe function: {}", e)
            )
        })?;

    state.callbacks.insert(callback_id, threadsafe_callback);

    Ok(callback_id)
}

/// Remove a drag event callback
#[napi]
pub fn remove_drag_event_listener(callback_id: u32) -> Result<bool> {
    // Validate callback ID
    if callback_id == 0 {
        return Err(Error::new(
            Status::InvalidArg,
            "Callback ID cannot be 0"
        ));
    }

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

/// Check if drag monitoring is currently active
#[napi]
pub fn is_monitoring() -> bool {
    let state = MONITOR_STATE.lock().unwrap();
    state.is_monitoring
}

/// Simulate a drag event for testing purposes
#[napi]
pub fn simulate_drag_event(files: Vec<String>) -> Result<()> {
    // Validate input
    if files.is_empty() {
        return Err(Error::new(
            Status::InvalidArg,
            "Files array cannot be empty"
        ));
    }

    println!("📤 Simulating drag event with {} file(s): {:?}", files.len(), files);

    // Use the unified trigger function
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    trigger_drag_event(files, 0.0, 0.0, platform);

    Ok(())
}