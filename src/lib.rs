use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode, ErrorStrategy};
use napi_derive::napi;
use rdev::{listen, Event, EventType, Button};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio, Child};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

// region: Mouse Monitoring

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
    static ref DRAG_STATE: Arc<Mutex<DragState>> = Arc::new(Mutex::new(DragState::new()));
}

#[derive(Debug)]
struct DragState {
    is_dragging: bool,
    helper_path: Option<String>,
}

impl DragState {
    fn new() -> Self {
        Self {
            is_dragging: false,
            helper_path: None,
        }
    }
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
    // Handle dynamic window creation/destruction based on mouse events
    handle_drag_window_management(&mouse_event);

    if let Ok(state) = MONITOR_STATE.lock() {
        for callback in state.callbacks.values() {
            callback.call(Ok(mouse_event.clone()), ThreadsafeFunctionCallMode::Blocking);
        }
    }
}

fn handle_drag_window_management(mouse_event: &MouseEvent) {
    match mouse_event.event_type.as_str() {
        "mousedown" => {
            // On mouse down, create drag monitoring window at mouse position
            if let Ok(mut drag_state) = DRAG_STATE.lock() {
                if let Some(ref helper_path) = drag_state.helper_path {
                    if !drag_state.is_dragging {
                        eprintln!("[main] === MOUSE COORDINATE DEBUG ===");
                        eprintln!("[main] Raw mouse down detected at ({}, {})", mouse_event.x, mouse_event.y);
                        eprintln!("[main] Platform: {}", mouse_event.platform);
                        eprintln!("[main] Timestamp: {}", mouse_event.timestamp);
                        eprintln!("[main] Button: {}", mouse_event.button);
                        eprintln!("[main] Passing coordinates to helper process...");

                        if let Err(e) = start_file_drag_monitor_internal(helper_path, mouse_event.x, mouse_event.y) {
                            eprintln!("[main] Failed to start file drag monitor: {}", e);
                        } else {
                            eprintln!("[main] Successfully sent coordinates ({}, {}) to helper process", mouse_event.x, mouse_event.y);
                            drag_state.is_dragging = true;
                        }
                        eprintln!("[main] === END MOUSE DEBUG ===");
                    }
                }
            }
        }
        "mouseup" => {
            // On mouse up, stop drag monitoring
            if let Ok(mut drag_state) = DRAG_STATE.lock() {
                if drag_state.is_dragging {
                    eprintln!("[main] Mouse up detected, stopping file drag monitor");
                    if let Err(e) = stop_file_drag_monitor_internal() {
                        eprintln!("[main] Failed to stop file drag monitor: {}", e);
                    } else {
                        drag_state.is_dragging = false;
                    }
                }
            }
        }
        _ => {
            // Other events don't affect drag monitoring
        }
    }
}

#[napi] pub fn start_mouse_monitor() -> Result<()> {
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

// region: File Drag Monitoring

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

#[derive(Deserialize, Debug)]
struct HelperDragEvent {
    event_type: String,
    path: Option<String>,
    x: f64,
    y: f64,
}

struct FileDragMonitorState {
    is_monitoring: bool,
    callbacks: Arc<Mutex<HashMap<u32, ThreadsafeFunction<FileDragEvent, ErrorStrategy::CalleeHandled>>>>,
    next_callback_id: u32,
    helper_process: Option<Child>,
    reader_thread: Option<thread::JoinHandle<()>>,
}

impl FileDragMonitorState {
    fn new() -> Self {
        Self {
            is_monitoring: false,
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            next_callback_id: 0,
            helper_process: None,
            reader_thread: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref FILE_DRAG_STATE: Mutex<FileDragMonitorState> = Mutex::new(FileDragMonitorState::new());
}

// Internal function to start file drag monitoring
fn start_file_drag_monitor_internal(helper_path_str: &str, mouse_x: f64, mouse_y: f64) -> Result<()> {
    eprintln!("[main] === HELPER PROCESS DEBUG ===");
    eprintln!("[main] start_file_drag_monitor_internal called with coordinates: ({}, {})", mouse_x, mouse_y);
    eprintln!("[main] Helper path: {}", helper_path_str);

    let mut state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;

    if state.is_monitoring {
        eprintln!("[main] Already monitoring, returning early");
        return Ok(());
    }

    let helper_path = std::path::PathBuf::from(helper_path_str);

    if !helper_path.exists() {
        eprintln!("[main] Helper executable not found at: {:?}", helper_path);
        return Err(Error::new(Status::GenericFailure, format!("Helper executable not found at {:?}. Please ensure it has been built.", helper_path)));
    }

    let x_str = mouse_x.to_string();
    let y_str = mouse_y.to_string();
    eprintln!("[main] About to spawn helper with args: [\"{}\", \"{}\"]", x_str, y_str);

    let mut child = Command::new(helper_path)
        .arg(&x_str)
        .arg(&y_str)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to spawn helper process: {}", e)))?;

    eprintln!("[main] Helper process spawned successfully with PID: {:?}", child.id());
    eprintln!("[main] === END HELPER DEBUG ===");

    let stdout = child.stdout.take().ok_or_else(|| Error::new(Status::GenericFailure, "Failed to capture helper stdout"))?;
    let callbacks = Arc::clone(&state.callbacks);

    let reader_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(json) => {
                    if let Ok(helper_event) = serde_json::from_str::<HelperDragEvent>(&json) {
                        let event_type = match helper_event.event_type.as_str() {
                            "hovered" => "hovered_file",
                            "dropped" => "dropped_file",
                            "cancelled" => "hovered_file_cancelled",
                            _ => "unknown",
                        }.to_string();

                        let event = FileDragEvent {
                            event_type,
                            file_path: helper_event.path.unwrap_or_default(),
                            x: helper_event.x,
                            y: helper_event.y,
                            timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
                            platform: std::env::consts::OS.to_string(),
                            window_id: "".to_string(),
                        };

                        let cbs = callbacks.lock().unwrap();
                        for callback in cbs.values() {
                            callback.call(Ok(event.clone()), ThreadsafeFunctionCallMode::Blocking);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from helper process: {}", e);
                    break;
                }
            }
        }
    });

    state.helper_process = Some(child);
    state.reader_thread = Some(reader_handle);
    state.is_monitoring = true;

    Ok(())
}

// Internal function to stop file drag monitoring
fn stop_file_drag_monitor_internal() -> Result<()> {
    let mut state = FILE_DRAG_STATE.lock().map_err(|_| Error::new(Status::GenericFailure, "Failed to acquire file drag state lock"))?;

    if !state.is_monitoring {
        return Ok(());
    }

    if let Some(mut child) = state.helper_process.take() {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(b"shutdown\n") {
                eprintln!("Failed to send shutdown command to helper: {}", e);
            }
        }
        if let Err(e) = child.wait() {
            eprintln!("Failed to wait for helper process: {}", e);
        }
    }

    if let Some(handle) = state.reader_thread.take() {
        let _ = handle.join();
    }

    state.is_monitoring = false;
    Ok(())
}

#[napi]
pub fn start_file_drag_monitor(helper_path_str: String) -> Result<()> {
    // Configure helper path for dynamic monitoring
    if let Ok(mut drag_state) = DRAG_STATE.lock() {
        drag_state.helper_path = Some(helper_path_str.clone());
        eprintln!("[main] Helper path configured for dynamic monitoring (window will be created on mouse movement)");
    }

    // Don't start monitoring immediately - let it be triggered by mouse events
    eprintln!("[main] Dynamic monitoring enabled - use mouse movement to trigger drag detection");
    Ok(())
}

#[napi]
pub fn stop_file_drag_monitor() -> Result<()> {
    // Clear helper path configuration
    if let Ok(mut drag_state) = DRAG_STATE.lock() {
        drag_state.helper_path = None;
        drag_state.is_dragging = false;
        eprintln!("[main] Helper path cleared, dynamic monitoring disabled");
    }

    // Stop current monitoring
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