use std::ptr::null_mut;

use winapi::shared::windef::HWND;

pub const TOTAL_STEP: u32 = 141;
pub const KPTEMP_VERSION: &str = "1.4";
pub static mut LABEL_HANDLE: HWND = null_mut();
pub static mut PROGRESS_HANDLE: HWND = null_mut();
