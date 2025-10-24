// This file has been emptied as part of keyboard functionality removal
// This file previously contained Windows keyboard event listening logic

use crate::rdev::{Event, EventType, ListenError};
use crate::windows::common::{HOOK, HookError, convert, set_key_hook, set_mouse_hook};
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HC_ACTION};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

impl From<HookError> for ListenError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => ListenError::MouseHookError(code),
            HookError::Key(code) => ListenError::KeyHookError(code),
        }
    }
}

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION {
            let opt = convert(param, lpdata);
            if let Some(event_type) = opt {
                let event = Event {
                    event_type,
                    time: SystemTime::now(),
                    name: None,
                };
                let ptr = &raw mut GLOBAL_CALLBACK;
                if let Some(callback) = &mut *ptr {
                    callback(event);
                }
            }
        }
        CallNextHookEx(HOOK, code, param, lpdata)
    }
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_key_hook()?;
        set_mouse_hook()?;
        let mut msg = std::mem::zeroed();
        while GetMessageA(&mut msg, null_mut(), 0, 0) != 0 {
            // Process message
        }
    }
    Ok(())
}