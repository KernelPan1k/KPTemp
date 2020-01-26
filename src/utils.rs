use std::env::var;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, Write};
use std::iter::once;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::null_mut;

use chrono::{DateTime, Local};
use pretty_bytes::converter::convert;
use winapi::shared::winerror::S_OK;
use winapi::um::reason::SHTDN_REASON_MINOR_MAINTENANCE;
use winapi::um::shellapi::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHQUERYRBINFO, SHQueryRecycleBinW};
use winapi::um::winuser::{EWX_FORCEIFHUNG, EWX_REBOOT, ExitWindowsEx, MB_ICONERROR, PostMessageW, WM_QUIT};
use winapi::um::winuser::{
    IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_TOPMOST, MB_YESNO, MessageBoxW,
};

use crate::clean::TempComponent;
use crate::globals::KPTEMP_VERSION;
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

pub unsafe fn write_report(
    temp_components: &Vec<TempComponent>,
    r_len: i64,
    r_size: i64,
    total_len: u64,
    total_size: u64,
) -> Result<(), Error> {
    let local: DateTime<Local> = Local::now();
    let user_profile: PathBuf = PathBuf::from(var("USERPROFILE").unwrap_or("C:\\".to_string()));
    let local_datetime = local.format("%a %b %e %T %Y");
    let report: PathBuf = user_profile.join(format!(
        "Desktop\\KpTemp_{}.txt", local.format("%Y-%m-%d_%H-%M-%S").to_string()
    ));
    let mut output = File::create(report);

    write!(output, "{}", format!("KpTemp v{} by kernel-panik\n", KPTEMP_VERSION))?;
    write!(output, "{}", format!("Date: {}\n\n", local_datetime.to_string()))?;

    if 0 == temp_components.len() {
        write!(output, "No records found\n")?;
    } else {
        for temp_component in temp_components {
            write!(output, "{}", format!(
                "{} : {} files => {} deleted\n",
                temp_component.path.display(),
                temp_component.len,
                convert(temp_component.size as f64)
            ))?;
        }
    }

    write!(output, "{}", format!(
        "RecycleBin : {} files => {} deleted\n",
        r_len,
        convert(r_size as f64)
    ))?;

    write!(output, "{}", format!(
        "Total : {} files => {} deleted\n",
        total_len,
        convert(total_size as f64)
    ))?;

    Ok(())
}
