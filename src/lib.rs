use napi::bindgen_prelude::*;
use napi_derive::napi;
// use serde::{Deserialize, Serialize}; // Reserved for future use
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Drag event data structure
#[napi(object)]
#[derive(Debug)]
pub struct DragEvent {
    /// Array of file paths being dragged
    pub files: Vec<String>,
    /// Timestamp when the drag event occurred (as f64 for compatibility)
    pub timestamp: f64,
    /// Optional source window information
    pub source: Option<String>,
}

/// Global state for drag monitoring
struct MonitorState {
    is_monitoring: bool,
    callbacks: Vec<u32>,
    next_callback_id: u32,
}

impl MonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            callbacks: Vec::new(),
            next_callback_id: 0,
        }
    }
}

lazy_static::lazy_static! {
    static ref MONITOR_STATE: Arc<Mutex<MonitorState>> = Arc::new(Mutex::new(MonitorState::new()));
}

/// Start monitoring drag events globally
#[napi]
pub fn start_drag_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().unwrap();

    if state.is_monitoring {
        return Ok(());
    }

    // Start platform-specific monitoring
    #[cfg(target_os = "windows")]
    {
        platform::windows::start_monitoring()?;
    }

    #[cfg(target_os = "macos")]
    {
        platform::macos::start_monitoring()?;
    }

    state.is_monitoring = true;
    Ok(())
}

/// Stop monitoring drag events
#[napi]
pub fn stop_drag_monitor() -> Result<()> {
    let mut state = MONITOR_STATE.lock().unwrap();

    if !state.is_monitoring {
        return Ok(());
    }

    // Stop platform-specific monitoring
    #[cfg(target_os = "windows")]
    {
        platform::windows::stop_monitoring()?;
    }

    #[cfg(target_os = "macos")]
    {
        platform::macos::stop_monitoring()?;
    }

    state.is_monitoring = false;
    state.callbacks.clear();
    Ok(())
}

/// Register a callback for drag events (simplified version)
#[napi]
pub fn on_drag_event(_callback: JsFunction) -> Result<u32> {
    let mut state = MONITOR_STATE.lock().unwrap();
    let callback_id = state.next_callback_id;
    state.next_callback_id += 1;
    state.callbacks.push(callback_id);

    // For now, just store the callback ID
    // In a real implementation, you'd need to handle JavaScript callbacks more carefully

    Ok(callback_id)
}

/// Remove a drag event callback
#[napi]
pub fn remove_drag_event_listener(callback_id: u32) -> Result<bool> {
    let mut state = MONITOR_STATE.lock().unwrap();
    if let Some(pos) = state.callbacks.iter().position(|&id| id == callback_id) {
        state.callbacks.remove(pos);
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
    let event = DragEvent {
        files,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
        source: Some("test".to_string()),
    };

    // In a real implementation, you'd trigger the callbacks here
    println!("Simulated drag event: {:?}", event);

    Ok(())
}

// Platform-specific implementations
#[cfg(target_os = "windows")]
mod platform {
    pub mod windows {
        use super::super::*;
        use std::sync::atomic::{AtomicBool, Ordering};

        static IS_MONITORING: AtomicBool = AtomicBool::new(false);

        pub fn start_monitoring() -> Result<()> {
            if IS_MONITORING.swap(true, Ordering::SeqCst) {
                return Ok(());
            }

            // TODO: Implement Windows-specific drag monitoring
            println!("Windows drag monitoring started (placeholder implementation)");
            Ok(())
        }

        pub fn stop_monitoring() -> Result<()> {
            IS_MONITORING.store(false, Ordering::SeqCst);
            println!("Windows drag monitoring stopped (placeholder implementation)");
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    pub mod macos {
        use super::super::*;
        use std::sync::atomic::{AtomicBool, Ordering};

        static IS_MONITORING: AtomicBool = AtomicBool::new(false);

        pub fn start_monitoring() -> Result<()> {
            if IS_MONITORING.swap(true, Ordering::SeqCst) {
                return Ok(());
            }

            // TODO: Implement macOS-specific drag monitoring
            println!("macOS drag monitoring started (placeholder implementation)");
            Ok(())
        }

        pub fn stop_monitoring() -> Result<()> {
            IS_MONITORING.store(false, Ordering::SeqCst);
            println!("macOS drag monitoring stopped (placeholder implementation)");
            Ok(())
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    pub fn start_monitoring() -> Result<()> {
        Err(Error::new(
            Status::Unsupported,
            "Platform not supported for drag monitoring",
        ))
    }

    pub fn stop_monitoring() -> Result<()> {
        Ok(())
    }
}