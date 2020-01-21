use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::process::exit;
use std::ptr::null_mut;

use winapi::_core::mem::zeroed;
use winapi::shared::winerror::S_OK;
use winapi::um::reason::SHTDN_REASON_MINOR_MAINTENANCE;
use winapi::um::shellapi::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHQUERYRBINFO, SHQueryRecycleBinW};
use winapi::um::winuser::{EWX_FORCEIFHUNG, EWX_REBOOT, ExitWindowsEx, MB_ICONERROR, PostMessageW, WM_QUIT};
use winapi::um::winuser::{
    IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_TOPMOST, MB_YESNO, MessageBoxW,
};

use crate::privilege::adjust_privilege;

pub fn restart() {
    adjust_privilege("SeShutdownPrivilege");
    unsafe { ExitWindowsEx(EWX_REBOOT | EWX_FORCEIFHUNG, SHTDN_REASON_MINOR_MAINTENANCE); }
}

pub fn empty_recycle_bin() {
    unsafe {
        SHEmptyRecycleBinW(null_mut(), null_mut(), SHERB_NOCONFIRMATION);
    }
}

pub unsafe fn data_recycle_bin() -> (i64, i64) {
    let mut info: SHQUERYRBINFO = zeroed();
    info.cbSize = size_of::<SHQUERYRBINFO>() as u32;

    let result = SHQueryRecycleBinW(null_mut(), &mut info);

    if result == S_OK {
        (info.i64NumItems, info.i64Size)
    } else {
        (0, 0)
    }
}

pub fn message_box(content: String) {
    let lp_text: Vec<u16> = content.encode_utf16().chain(once(0)).collect();
    let lp_caption: Vec<u16> = "KpTemp".encode_utf16().chain(once(0)).collect();

    unsafe {
        MessageBoxW(
            null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            MB_ICONINFORMATION | MB_OK | MB_TOPMOST,
        );
    };
}

pub fn error_box(content: String) {
    let lp_text: Vec<u16> = content.encode_utf16().chain(once(0)).collect();
    let lp_caption: Vec<u16> = "Error".encode_utf16().chain(once(0)).collect();

    unsafe {
        MessageBoxW(
            null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            MB_ICONERROR | MB_OK | MB_TOPMOST,
        );
    };
}

pub fn eula() {
    let content: &str = "This software is provided \"AS IS\" without warranty of any kind.\n\
    You may use this software at your own risk.\n\
    This software is not permitted for commercial purposes.\n\
    After it runs, your computer will restart, you must save your work.\n\
    Are you sure you want to continue?\n\
    Write Y (Yes) to continue. \
    Write N(No) to exit.\n\
    What's your answer? Y/N\n";

    let lp_text: Vec<u16> = content.encode_utf16().chain(once(0)).collect();
    let lp_caption: Vec<u16> = "KpTemp".encode_utf16().chain(once(0)).collect();

    let answer: i32 = unsafe {
        MessageBoxW(
            null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            MB_ICONQUESTION | MB_YESNO | MB_TOPMOST,
        )
    };

    if answer != IDYES {
        exit(0);
    }
}

pub fn to_utf16(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0u16).into_iter())
        .collect()
}

pub unsafe fn exit_all() {
    PostMessageW(null_mut(), WM_QUIT, 0, 0);
    exit(0);
}
