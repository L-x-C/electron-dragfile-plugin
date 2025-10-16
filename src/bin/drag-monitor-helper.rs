use serde::Serialize;
use std::io::{self, BufRead};
use std::thread;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowLevel, WindowButtons},
    dpi::{PhysicalSize, PhysicalPosition},
};

#[derive(Serialize, Debug)]
struct DragEvent {
    event_type: String,
    path: Option<String>,
    x: f64,
    y: f64,
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    cursor_position: (f64, f64),
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            // Get the primary monitor's dimensions
            let primary_monitor = event_loop.primary_monitor().unwrap_or_else(|| {
                // Fallback to first available monitor
                event_loop.available_monitors().next()
                    .expect("No monitors available")
            });

            let monitor_size = primary_monitor.size();

            // Create a semi-transparent full-screen overlay that can detect file drags
            // Use the successful configuration with visual optimization
            let attributes = WindowAttributes::default()
                .with_title("File Drag Monitor Helper")
                .with_transparent(false) // 不透明，确保能接收拖拽事件
                .with_decorations(false) // 无边框
                .with_window_level(WindowLevel::AlwaysOnTop) // 顶层窗口，确保接收事件
                .with_resizable(false)
                .with_enabled_buttons(WindowButtons::empty()) // 无窗口按钮
                .with_visible(true)
                .with_inner_size(PhysicalSize::new(monitor_size.width, monitor_size.height)) // 全屏尺寸
                .with_position(PhysicalPosition::new(0, 0)) // 覆盖整个屏幕
                .with_active(true); // 获得焦点，才能接收拖拽事件

            let window = event_loop.create_window(attributes).unwrap();

            // Request the window to be as unobtrusive as possible
            window.set_cursor_visible(false);

            // Platform-specific information
            #[cfg(target_os = "macos")]
            {
                eprintln!("[helper] macOS window created - transparency should minimize focus impact");
            }

            eprintln!("[helper] Created semi-transparent full-screen window: {}x{} (ready for drag detection)",
                monitor_size.width, monitor_size.height);

            // Quick startup signal - indicate window is ready
            eprintln!("[helper] Semi-transparent window is ready and waiting for drag events");

            self.window = Some(window);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
        eprintln!("[helper] Shutdown signal received, exiting.");
        event_loop.exit();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                eprintln!("[helper] Window close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x, position.y);
                // Uncomment for debugging: eprintln!("[helper] Cursor moved to: ({}, {})", position.x, position.y);
            }
            WindowEvent::HoveredFile(path) => {
                eprintln!("[helper] 🎯 File hovered: {} at ({}, {})",
                    path.to_string_lossy(), self.cursor_position.0, self.cursor_position.1);
                let event = DragEvent {
                    event_type: "hovered".to_string(),
                    path: Some(path.to_string_lossy().to_string()),
                    x: self.cursor_position.0,
                    y: self.cursor_position.1,
                };
                if let Ok(json) = serde_json::to_string(&event) {
                    println!("{}", json);
                }
            }
            WindowEvent::DroppedFile(path) => {
                eprintln!("[helper] ✅ File dropped: {} at ({}, {})",
                    path.to_string_lossy(), self.cursor_position.0, self.cursor_position.1);
                let event = DragEvent {
                    event_type: "dropped".to_string(),
                    path: Some(path.to_string_lossy().to_string()),
                    x: self.cursor_position.0,
                    y: self.cursor_position.1,
                };
                if let Ok(json) = serde_json::to_string(&event) {
                    println!("{}", json);
                }
                event_loop.exit(); // Exit after a file is dropped
            }
            WindowEvent::HoveredFileCancelled => {
                eprintln!("[helper] ❌ File hover cancelled");
                let event = DragEvent {
                    event_type: "cancelled".to_string(),
                    path: None,
                    x: self.cursor_position.0,
                    y: self.cursor_position.1,
                };
                if let Ok(json) = serde_json::to_string(&event) {
                    println!("{}", json);
                }
            }
            WindowEvent::Focused(focused) => {
                eprintln!("[helper] Window focus changed: {}", focused);
            }
            WindowEvent::RedrawRequested => {
                // eprintln!("[helper] Window redraw requested");
            }
            _ => {
                // Uncomment for debugging all events: eprintln!("[helper] Other window event: {:?}", event);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[helper] Starting drag monitor helper process (fast startup mode).");
    let event_loop = EventLoop::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    // Quick startup indication
    eprintln!("[helper] Event loop created successfully");

    // Start a thread to listen for shutdown command from stdin
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if line.trim() == "shutdown" {
                    let _ = proxy.send_event(());
                    break;
                }
            }
        }
    });

    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    eprintln!("[helper] Helper process finished.");
    Ok(())
}
