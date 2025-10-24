use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGEvent, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMouseButton, CGScrollEventUnit,
};

use crate::rdev::{Button, EventType, SimulateError};

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CFRetained<CGEventSource>,
) -> Option<CFRetained<CGEvent>> {
      match event_type {
        EventType::ButtonPress(button) => {
            let mouse_button = match button {
                Button::Left => CGMouseButton::Left,
                Button::Right => CGMouseButton::Right,
                Button::Middle => CGMouseButton::Center,
                Button::Unknown(_) => CGMouseButton::Left,
            };
            let event = CGEvent::new_mouse_event(
                Some(&source),
                CGEventType::LeftMouseDown,
                CGPoint::new(0.0, 0.0),
                mouse_button,
            )?;
            Some(event)
        }
        EventType::ButtonRelease(button) => {
            let mouse_button = match button {
                Button::Left => CGMouseButton::Left,
                Button::Right => CGMouseButton::Right,
                Button::Middle => CGMouseButton::Center,
                Button::Unknown(_) => CGMouseButton::Left,
            };
            let event = CGEvent::new_mouse_event(
                Some(&source),
                CGEventType::LeftMouseUp,
                CGPoint::new(0.0, 0.0),
                mouse_button,
            )?;
            Some(event)
        }
        EventType::MouseMove { x, y } => {
            let event = CGEvent::new_mouse_event(
                Some(&source),
                CGEventType::MouseMoved,
                CGPoint::new(*x, *y),
                CGMouseButton::Left,
            )?;
            Some(event)
        }
        EventType::Wheel { delta_x, delta_y } => {
            let event = CGEvent::new_scroll_wheel_event2(
                Some(&source),
                CGScrollEventUnit::Pixel,
                1, // wheel count
                *delta_y as i32,
                *delta_x as i32,
                0, // delta1
            )?;
            Some(event)
        }
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .ok_or(SimulateError)?;
        let event = convert_native_with_source(event_type, source)
            .ok_or(SimulateError)?;
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
    Ok(())
}