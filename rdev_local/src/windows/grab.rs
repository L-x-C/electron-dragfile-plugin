use crate::rdev::{Event, GrabError};
use crate::windows::common::{HOOK, HookError, convert, set_mouse_hook};
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HC_ACTION};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize {
    unsafe {
        if code == HC_ACTION {
            let opt = convert(param, lpdata);
            if let Some(event_type) = opt {
                let name = None; // No keyboard name for mouse events
                let event = Event {
                    event_type,
                    time: SystemTime::now(),
                    name,
                };
                let ptr = &raw mut GLOBAL_CALLBACK;
                if let Some(callback) = &mut *ptr {
                    if callback(event).is_none() {
                        // https://stackoverflow.com/questions/42756284/blocking-windows-mouse-click-using-setwindowshookex
                        // https://android.developreference.com/article/14560004/Blocking+windows+mouse+click+using+SetWindowsHookEx()
                        // https://cboard.cprogramming.com/windows-programming/99678-setwindowshookex-wm_keyboard_ll.html
                        // let _result = CallNextHookEx(HOOK, code, param, lpdata);
                        return 1;
                    }
                }
            }
        }
        CallNextHookEx(HOOK, code, param, lpdata)
    }
}
impl From<HookError> for GrabError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => GrabError::MouseHookError(code),
        }
    }
}

pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_mouse_hook(raw_callback)?;

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
    Ok(())
}
