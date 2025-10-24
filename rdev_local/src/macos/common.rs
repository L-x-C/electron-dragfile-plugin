#![allow(clippy::upper_case_acronyms)]
use crate::rdev::{Button, Event, EventType};
use core::ptr::NonNull;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventType};
use std::time::SystemTime;

pub unsafe fn convert(
    _type: CGEventType,
    cg_event: NonNull<CGEvent>,
) -> Option<Event> {
    unsafe {
        let option_type = match _type {
            CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
            CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
            CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
            CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
            CGEventType::MouseMoved => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::LeftMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::RightMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::ScrollWheel => {
                let delta_y = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventDeltaAxis1,
                );
                let delta_x = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventDeltaAxis2,
                );
                Some(EventType::Wheel { delta_x, delta_y })
            }
            // Ignore all keyboard events
            CGEventType::KeyDown | CGEventType::KeyUp | CGEventType::FlagsChanged => None,
            CGEventType(14) => {
                // Core graphics special events - ignore keyboard subtype 8
                let subtype =
                    CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField(99));
                // Subtype 8 means keyboard event, ignore it
                if subtype == 8 {
                    None
                } else {
                    // Handle other special events (like mouse buttons) if needed
                    None
                }
            }
            _ev => None,
        };

        if let Some(event_type) = option_type {
            return Some(Event {
                event_type,
                time: SystemTime::now(),
                name: None,
            });
        }
    }
    None
}
