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
    initial_position: Option<(f64, f64)>,
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            eprintln!("[helper] === WINDOW CREATION DEBUG ===");

            // Get the primary monitor's dimensions
            let primary_monitor = event_loop.primary_monitor().unwrap_or_else(|| {
                // Fallback to first available monitor
                event_loop.available_monitors().next()
                    .expect("No monitors available")
            });

            let monitor_size = primary_monitor.size();
            eprintln!("[helper] Primary monitor size: {}x{}", monitor_size.width, monitor_size.height);

            // Get monitor scale factor for HiDPI displays
            let scale_factor = primary_monitor.scale_factor();
            eprintln!("[helper] Monitor scale factor: {}", scale_factor);

            // Get monitor position
            let position = primary_monitor.position();
            eprintln!("[helper] Monitor position: ({}, {})", position.x, position.y);

            // Create a 100x100 window
            let window_width = 100;
            let window_height = 100;
            eprintln!("[helper] Window dimensions: {}x{}", window_width, window_height);

            // Calculate window position - either at mouse coordinates or centered
            let (window_x, window_y) = if let Some((mouse_x, mouse_y)) = self.initial_position {
                eprintln!("[helper] Using mouse coordinates: ({}, {})", mouse_x, mouse_y);

                // TESTING: Try different positioning strategies
                // Strategy 1: Direct positioning (no offset) - window top-left at mouse
                let strategy1_x = mouse_x;
                let strategy1_y = mouse_y;

                // Strategy 2: Centered positioning (current approach) - window center at mouse
                let half_width = window_width as f64 / 2.0;
                let half_height = window_height as f64 / 2.0;
                let strategy2_x = mouse_x - half_width;
                let strategy2_y = mouse_y - half_height;

                // Strategy 3: Small offset - slight offset from mouse for better visibility
                let offset = 10.0;
                let strategy3_x = mouse_x - offset;
                let strategy3_y = mouse_y - offset;

                eprintln!("[helper] Testing positioning strategies:");
                eprintln!("[helper] Strategy 1 (direct): ({}, {})", strategy1_x, strategy1_y);
                eprintln!("[helper] Strategy 2 (centered): ({}, {})", strategy2_x, strategy2_y);
                eprintln!("[helper] Strategy 3 (offset): ({}, {})", strategy3_x, strategy3_y);

                // CRITICAL FIX: Apply scale factor for HiDPI displays
                // rdev returns logical coordinates, but winit needs physical coordinates
                let scaled_strategy1_x = strategy1_x * scale_factor;
                let scaled_strategy1_y = strategy1_y * scale_factor;

                eprintln!("[helper] 🎯 SCALE FACTOR FIX DETECTED!");
                eprintln!("[helper] Original logical coordinates: ({}, {})", strategy1_x, strategy1_y);
                eprintln!("[helper] Scale factor: {}", scale_factor);
                eprintln!("[helper] Scaled physical coordinates: ({}, {})", scaled_strategy1_x, scaled_strategy1_y);

                let x_before_clamp = scaled_strategy1_x;
                let y_before_clamp = scaled_strategy1_y;

                eprintln!("[helper] Using scaled coordinates for window positioning");
                eprintln!("[helper] Position before clamping: ({}, {})", x_before_clamp, y_before_clamp);

                let x = x_before_clamp.max(0.0);
                let y = y_before_clamp.max(0.0);

                eprintln!("[helper] Position after min clamp: ({}, {})", x, y);

                // Ensure window doesn't go off screen
                let max_x = (monitor_size.width - window_width) as f64;
                let max_y = (monitor_size.height - window_height) as f64;

                eprintln!("[helper] Maximum allowed position: ({}, {})", max_x, max_y);

                let x_final = x.min(max_x);
                let y_final = y.min(max_y);

                eprintln!("[helper] Final calculated position: ({}, {})", x_final, y_final);
                eprintln!("[helper] Position difference from mouse: ({}, {})",
                    x_final - mouse_x, y_final - mouse_y);

                (x_final as u32, y_final as u32)
            } else {
                eprintln!("[helper] No mouse coordinates available, using center positioning");
                // Fallback to centered positioning
                let center_x = (monitor_size.width - window_width) / 2;
                let center_y = (monitor_size.height - window_height) / 2;
                eprintln!("[helper] Center position calculated: ({}, {})", center_x, center_y);
                (center_x, center_y)
            };

            eprintln!("[helper] Final window position: ({}, {})", window_x, window_y);

            // Create a small window positioned at mouse click location (or center as fallback)
            let attributes = WindowAttributes::default()
                .with_title("File Drag Monitor Helper")
                .with_transparent(false) // 不透明，确保能接收拖拽事件
                .with_decorations(false) // 无边框
                .with_window_level(WindowLevel::AlwaysOnTop) // 顶层窗口，确保接收事件
                .with_resizable(false)
                .with_enabled_buttons(WindowButtons::empty()) // 无窗口按钮
                .with_visible(true)
                .with_inner_size(PhysicalSize::new(window_width, window_height)) // 100x100尺寸
                .with_position(PhysicalPosition::new(window_x, window_y)) // 鼠标点击位置或屏幕中央
                .with_active(true); // 获得焦点，才能接收拖拽事件

            let window = event_loop.create_window(attributes).unwrap();

            // Request the window to be as unobtrusive as possible
            window.set_cursor_visible(false);

            // Platform-specific information
            #[cfg(target_os = "macos")]
            {
                eprintln!("[helper] macOS window created - transparency should minimize focus impact");
            }

            if let Some((mouse_x, mouse_y)) = self.initial_position {
                eprintln!("[helper] ✓ Created 100x100 window at calculated position ({}, {})", window_x, window_y);
                eprintln!("[helper] ✓ Original mouse coordinates: ({}, {})", mouse_x, mouse_y);
                eprintln!("[helper] ✓ Window offset from mouse: ({}, {})",
                    window_x as f64 - mouse_x, window_y as f64 - mouse_y);
            } else {
                eprintln!("[helper] ✓ Created 100x100 centered window at position ({}, {}) (fallback)", window_x, window_y);
            }

            // Quick startup signal - indicate window is ready
            eprintln!("[helper] ✓ Window created successfully and ready for drag events");
            eprintln!("[helper] === END WINDOW CREATION DEBUG ===");

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
    let args: Vec<String> = std::env::args().collect();

    eprintln!("[helper] === COMMAND LINE DEBUG ===");
    eprintln!("[helper] Total arguments received: {}", args.len());
    for (i, arg) in args.iter().enumerate() {
        eprintln!("[helper] Arg {}: '{}'", i, arg);
    }

    let initial_position = if args.len() >= 3 {
        // Parse mouse coordinates from command line arguments
        match (args[1].parse::<f64>(), args[2].parse::<f64>()) {
            (Ok(x), Ok(y)) => {
                eprintln!("[helper] ✓ Successfully parsed mouse coordinates: ({}, {})", x, y);
                eprintln!("[helper] ✓ X coordinate type: {}, value: {}", std::any::type_name::<f64>(), x);
                eprintln!("[helper] ✓ Y coordinate type: {}, value: {}", std::any::type_name::<f64>(), y);
                Some((x, y))
            }
            (Err(e_x), Err(e_y)) => {
                eprintln!("[helper] ✗ Failed to parse both coordinates: X error: {}, Y error: {}", e_x, e_y);
                None
            }
            (Err(e), _) => {
                eprintln!("[helper] ✗ Failed to parse X coordinate: {}", e);
                None
            }
            (_, Err(e)) => {
                eprintln!("[helper] ✗ Failed to parse Y coordinate: {}", e);
                None
            }
        }
    } else {
        eprintln!("[helper] ✗ Insufficient arguments (got {}, need at least 3)", args.len());
        eprintln!("[helper] Usage: program <x_coord> <y_coord>");
        None
    };
    eprintln!("[helper] === END COMMAND LINE DEBUG ===");

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
    let mut app = App {
        initial_position,
        ..Default::default()
    };
    event_loop.run_app(&mut app)?;
    eprintln!("[helper] Helper process finished.");
    Ok(())
}
