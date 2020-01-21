use std::mem;
use std::ptr::null_mut;

use winapi::um::winuser::{DispatchMessageW, GetMessageW, TranslateMessage};
use winapi::um::winuser::MSG;

#[inline(always)]
pub unsafe fn dispatch_events() {
    let mut msg: MSG = mem::zeroed();

    while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
