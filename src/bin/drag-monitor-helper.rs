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
    windows: Vec<Window>,
    cursor_position: (f64, f64),
    initial_position: Option<(f64, f64)>,
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            eprintln!("[helper] === 4-WINDOW BORDER CREATION DEBUG ===");

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

            // Calculate border window positions around mouse center
            let window_positions = if let Some((mouse_x, mouse_y)) = self.initial_position {
                eprintln!("[helper] 🎯 4-WINDOW BORDER MODE - Using mouse coordinates: ({}, {})", mouse_x, mouse_y);

                // Apply scale factor for HiDPI displays
                // rdev returns logical coordinates, but winit needs physical coordinates
                let scaled_mouse_x = mouse_x * scale_factor;
                let scaled_mouse_y = mouse_y * scale_factor;

                eprintln!("[helper] 🎯 SCALE FACTOR FIX DETECTED!");
                eprintln!("[helper] Original logical coordinates: ({}, {})", mouse_x, mouse_y);
                eprintln!("[helper] Scale factor: {}", scale_factor);
                eprintln!("[helper] Scaled physical coordinates: ({}, {})", scaled_mouse_x, scaled_mouse_y);

                // Calculate border window positions (4 windows: top, bottom, left, right)
                let distance = 15.0; // 15 pixels distance from mouse center

                // Define border window sizes
                let (top_width, top_height) = (40.0, 10.0);  // Top border: 40x10
                let (bottom_width, bottom_height) = (40.0, 10.0);  // Bottom border: 40x10
                let (left_width, left_height) = (10.0, 40.0);  // Left border: 10x40
                let (right_width, right_height) = (10.0, 40.0);  // Right border: 10x40

                // Calculate positions for 4 border windows
                let mut positions = Vec::with_capacity(4);

                eprintln!("[helper] 📐 Calculating 4-window border positions at {}px distance", distance);
                eprintln!("[helper] Border layout (口-shaped):");

                // Top window: positioned above mouse center
                let top_x = scaled_mouse_x - (top_width / 2.0);
                let top_y = scaled_mouse_y - distance - (top_height / 2.0);
                positions.push((top_x, top_y, "top", top_width, top_height));

                // Bottom window: positioned below mouse center
                let bottom_x = scaled_mouse_x - (bottom_width / 2.0);
                let bottom_y = scaled_mouse_y + distance - (bottom_height / 2.0);
                positions.push((bottom_x, bottom_y, "bottom", bottom_width, bottom_height));

                // Left window: positioned to the left of mouse center
                let left_x = scaled_mouse_x - distance - (left_width / 2.0);
                let left_y = scaled_mouse_y - (left_height / 2.0);
                positions.push((left_x, left_y, "left", left_width, left_height));

                // Right window: positioned to the right of mouse center
                let right_x = scaled_mouse_x + distance - (right_width / 2.0);
                let right_y = scaled_mouse_y - (right_height / 2.0);
                positions.push((right_x, right_y, "right", right_width, right_height));

                // Print border window layout
                eprintln!("  [TOP]    ({}, {}) {}x{} ⬜", top_x, top_y, top_width, top_height);
                eprintln!("  [BOTTOM] ({}, {}) {}x{} ⬜", bottom_x, bottom_y, bottom_width, bottom_height);
                eprintln!("  [LEFT]   ({}, {}) {}x{} ⬜", left_x, left_y, left_width, left_height);
                eprintln!("  [RIGHT]  ({}, {}) {}x{} ⬜", right_x, right_y, right_width, right_height);

                // Apply boundary checks and adjustments
                let mut adjusted_positions = Vec::with_capacity(4);
                let mut boundary_adjustments = 0;

                eprintln!("[helper] 🔍 Applying boundary checks...");

                for (window_x, window_y, position_name, window_width, window_height) in positions {
                    let max_x = (monitor_size.width as f64 - window_width);
                    let max_y = (monitor_size.height as f64 - window_height);

                    let final_x = window_x.max(0.0).min(max_x);
                    let final_y = window_y.max(0.0).min(max_y);

                    let x_adjusted = (final_x != window_x);
                    let y_adjusted = (final_y != window_y);

                    if x_adjusted || y_adjusted {
                        boundary_adjustments += 1;
                        eprintln!("  ⚠️  Window [{}] adjusted from ({}, {}) to ({}, {})",
                            position_name, window_x, window_y, final_x, final_y);
                    }

                    adjusted_positions.push((final_x as u32, final_y as u32, position_name, window_width as u32, window_height as u32));
                }

                eprintln!("[helper] ✅ Border calculation complete: {} windows, {} boundary adjustments",
                    adjusted_positions.len(), boundary_adjustments);

                adjusted_positions
            } else {
                eprintln!("[helper] No mouse coordinates available, using centered border layout");
                // Fallback to centered border layout
                let center_x = (monitor_size.width as f64) / 2.0;
                let center_y = (monitor_size.height as f64) / 2.0;
                let distance = 15.0;

                // Define border window sizes
                let (top_width, top_height) = (40.0, 10.0);
                let (bottom_width, bottom_height) = (40.0, 10.0);
                let (left_width, left_height) = (10.0, 40.0);
                let (right_width, right_height) = (10.0, 40.0);

                let mut positions = Vec::with_capacity(4);

                // Calculate centered positions
                positions.push(((center_x - (top_width / 2.0)) as u32, (center_y - distance - (top_height / 2.0)) as u32, "top", top_width as u32, top_height as u32));
                positions.push(((center_x - (bottom_width / 2.0)) as u32, (center_y + distance - (bottom_height / 2.0)) as u32, "bottom", bottom_width as u32, bottom_height as u32));
                positions.push(((center_x - distance - (left_width / 2.0)) as u32, (center_y - (left_height / 2.0)) as u32, "left", left_width as u32, left_height as u32));
                positions.push(((center_x + distance - (right_width / 2.0)) as u32, (center_y - (right_height / 2.0)) as u32, "right", right_width as u32, right_height as u32));

                eprintln!("[helper] ✅ Centered border layout created with {} windows", positions.len());
                positions
            };

            // Create 4 border windows in "口" shape around mouse position
            for (i, (window_x, window_y, position_name, window_width, window_height)) in window_positions.iter().enumerate() {
                let window_num = i + 1;

                eprintln!("[helper] Creating Window {} [{}] at position ({}, {}) with size {}x{}",
                    window_num, position_name, window_x, window_y, window_width, window_height);

                let attributes = WindowAttributes::default()
                    .with_title(format!("File Drag Monitor {}", position_name))
                    .with_transparent(false) // 不透明，确保能接收拖拽事件
                    .with_decorations(false) // 无边框
                    .with_window_level(WindowLevel::AlwaysOnTop) // 顶层窗口，确保接收事件
                    .with_resizable(false)
                    .with_enabled_buttons(WindowButtons::empty()) // 无窗口按钮
                    .with_visible(true)
                    .with_inner_size(PhysicalSize::new(*window_width, *window_height)) // 边框窗口尺寸
                    .with_position(PhysicalPosition::new(*window_x, *window_y))
                    .with_active(i == 0); // First window gets focus

                let window = event_loop.create_window(attributes).unwrap();

                // Request the window to be as unobtrusive as possible
                window.set_cursor_visible(false);

                self.windows.push(window);

                if let Some((mouse_x, mouse_y)) = self.initial_position {
                    eprintln!("[helper] ✓ Created Window {} [{}] at ({}, {}) around mouse center ({}, {})",
                        window_num, position_name, window_x, window_y, mouse_x, mouse_y);
                } else {
                    eprintln!("[helper] ✓ Created Window {} [{}] at centered position ({}, {}) (fallback)",
                        window_num, position_name, window_x, window_y);
                }
            }

            // Platform-specific information
            #[cfg(target_os = "macos")]
            {
                eprintln!("[helper] macOS windows created - border layout should provide optimal coverage");
            }

            // Quick startup signal - indicate windows are ready
            eprintln!("[helper] ✓ {} border windows created successfully and ready for drag events", self.windows.len());
            eprintln!("[helper] 🎯 Border coverage: 口-shaped layout with top (40x10), bottom (40x10), left (10x40), right (10x40) at 15px distance");
            eprintln!("[helper] === END 4-WINDOW BORDER CREATION DEBUG ===");
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
