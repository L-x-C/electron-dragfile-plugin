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

            // Create a 2x2 window
            let window_width = 2;
            let window_height = 2;
            eprintln!("[helper] Window dimensions: {}x{}", window_width, window_height);

            // Calculate positions for 5x5 grid windows
            let window_positions = if let Some((mouse_x, mouse_y)) = self.initial_position {
                eprintln!("[helper] 🎯 5x5 GRID WINDOW MODE - Using mouse coordinates: ({}, {})", mouse_x, mouse_y);

                // Apply scale factor for HiDPI displays
                // rdev returns logical coordinates, but winit needs physical coordinates
                let scaled_mouse_x = mouse_x * scale_factor;
                let scaled_mouse_y = mouse_y * scale_factor;

                eprintln!("[helper] 🎯 SCALE FACTOR FIX DETECTED!");
                eprintln!("[helper] Original logical coordinates: ({}, {})", mouse_x, mouse_y);
                eprintln!("[helper] Scale factor: {}", scale_factor);
                eprintln!("[helper] Scaled physical coordinates: ({}, {})", scaled_mouse_x, scaled_mouse_y);

                // Calculate grid positions (24 windows in 5x5 grid around center, no center window)
                let spacing = 10.0; // 10 pixels between windows
                let mut positions = Vec::with_capacity(24);

                eprintln!("[helper] 📐 Calculating 5x5 grid positions with {}px spacing", spacing);
                eprintln!("[helper] Grid layout (no center window):");

                for row in -2..=2 {
                    for col in -2..=2 {
                        // Skip the center window [0,0]
                        if row == 0 && col == 0 {
                            continue;
                        }

                        let offset_x = col as f64 * spacing;
                        let offset_y = row as f64 * spacing;
                        let window_x = scaled_mouse_x + offset_x;
                        let window_y = scaled_mouse_y + offset_y;
                        positions.push((window_x, window_y, row, col));

                        // Visual grid representation
                        eprintln!("  [{}{}] ({}, {}) ⬜",
                            match col {
                                -2 => "LL",
                                -1 => "L",
                                0 => "C",
                                1 => "R",
                                2 => "RR",
                                _ => "?"
                            },
                            match row {
                                -2 => "TT",
                                -1 => "T",
                                0 => "M",
                                1 => "B",
                                2 => "BB",
                                _ => "?"
                            },
                            window_x, window_y);
                    }
                }

                // Apply boundary checks and adjustments
                let max_x = (monitor_size.width - window_width) as f64;
                let max_y = (monitor_size.height - window_height) as f64;
                let mut adjusted_positions = Vec::with_capacity(24);
                let mut boundary_adjustments = 0;

                eprintln!("[helper] 🔍 Applying boundary checks...");

                for (window_x, window_y, row, col) in positions {
                    let final_x = window_x.max(0.0).min(max_x);
                    let final_y = window_y.max(0.0).min(max_y);

                    let x_adjusted = (final_x != window_x);
                    let y_adjusted = (final_y != window_y);

                    if x_adjusted || y_adjusted {
                        boundary_adjustments += 1;
                        eprintln!("  ⚠️  Window [{},{}] adjusted from ({}, {}) to ({}, {})",
                            col, row, window_x, window_y, final_x, final_y);
                    }

                    adjusted_positions.push((final_x as u32, final_y as u32, row, col));
                }

                eprintln!("[helper] ✅ Grid calculation complete: {} windows, {} boundary adjustments",
                    adjusted_positions.len(), boundary_adjustments);

                adjusted_positions
            } else {
                eprintln!("[helper] No mouse coordinates available, using centered 5x5 grid");
                // Fallback to centered 5x5 grid
                let center_x = (monitor_size.width - window_width) / 2;
                let center_y = (monitor_size.height - window_height) / 2;
                let spacing = 10.0; // 10 pixels between windows

                let mut positions = Vec::with_capacity(24);

                for row in -2..=2 {
                    for col in -2..=2 {
                        // Skip the center window [0,0]
                        if row == 0 && col == 0 {
                            continue;
                        }

                        let window_x = center_x as f64 + col as f64 * spacing;
                        let window_y = center_y as f64 + row as f64 * spacing;
                        let final_x = window_x.max(0.0).min((monitor_size.width - window_width) as f64);
                        let final_y = window_y.max(0.0).min((monitor_size.height - window_height) as f64);
                        positions.push((final_x as u32, final_y as u32, row, col));
                    }
                }

                eprintln!("[helper] ✅ Centered grid created with {} windows", positions.len());
                positions
            };

            // Create 24 windows in 5x5 grid around mouse position (no center window)
            for (i, (window_x, window_y, row, col)) in window_positions.iter().enumerate() {
                let grid_pos = format!("[{},{}]", col, row);
                let window_num = i + 1;

                eprintln!("[helper] Creating Window {} {} at position ({}, {})",
                    window_num, grid_pos, window_x, window_y);

                let attributes = WindowAttributes::default()
                    .with_title(format!("File Drag Monitor {}", grid_pos))
                    .with_transparent(false) // 不透明，确保能接收拖拽事件
                    .with_decorations(false) // 无边框
                    .with_window_level(WindowLevel::AlwaysOnTop) // 顶层窗口，确保接收事件
                    .with_resizable(false)
                    .with_enabled_buttons(WindowButtons::empty()) // 无窗口按钮
                    .with_visible(true)
                    .with_inner_size(PhysicalSize::new(window_width, window_height)) // 2x2尺寸
                    .with_position(PhysicalPosition::new(*window_x, *window_y))
                    .with_active(i == 0); // First window gets focus

                let window = event_loop.create_window(attributes).unwrap();

                // Request the window to be as unobtrusive as possible
                window.set_cursor_visible(false);

                self.windows.push(window);

                if let Some((mouse_x, mouse_y)) = self.initial_position {
                    let center_offset_x = *col as f64 * 10.0;
                    let center_offset_y = *row as f64 * 10.0;
                    eprintln!("[helper] ✓ Created Window {} {} at offset ({}, {}) from mouse center",
                        window_num, grid_pos, center_offset_x, center_offset_y);
                    eprintln!("[helper] ✓ Original mouse coordinates: ({}, {})", mouse_x, mouse_y);
                } else {
                    eprintln!("[helper] ✓ Created Window {} {} at centered position ({}, {}) (fallback)",
                        window_num, grid_pos, window_x, window_y);
                }
            }

            // Platform-specific information
            #[cfg(target_os = "macos")]
            {
                eprintln!("[helper] macOS windows created - transparency should minimize focus impact");
            }

            // Quick startup signal - indicate windows are ready
            eprintln!("[helper] ✓ {} windows created successfully and ready for drag events", self.windows.len());
            eprintln!("[helper] 🎯 5x5 Grid coverage: ~42x42 pixels centered on mouse position (with 10px spacing)");
            eprintln!("[helper] === END 5x5 GRID WINDOW CREATION DEBUG ===");
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
