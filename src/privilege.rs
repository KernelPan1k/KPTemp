extern crate winapi;

use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use winapi::shared::minwindef::DWORD;
use winapi::um::processthreadsapi::{GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::winnt::{SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES};

pub fn adjust_privilege(privilege_name: &str) -> bool {
    let mut token_handle = null_mut();

    let mut status: i32 = unsafe {
        OpenThreadToken(
            GetCurrentThread(),
            TOKEN_ADJUST_PRIVILEGES,
            1,
            &mut token_handle,
        )
    };

    if status == 0 {
        status = unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES,
                &mut token_handle,
            )
        };

        if status == 0 {
            return false;
        }
    }

    let mut token_privilege: TOKEN_PRIVILEGES = unsafe { zeroed() };
    let name: &OsStr = privilege_name.as_ref();
    let name = name.encode_wide().chain(Some(0)).collect::<Vec<_>>();

    status = unsafe {
        LookupPrivilegeValueW(
            null_mut(),
            name.as_ptr(),
            &mut token_privilege.Privileges[0].Luid,
        )
    };

    if status == 0 {
        return false;
    }

    token_privilege.PrivilegeCount = 1;
    token_privilege.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    let size = size_of::<TOKEN_PRIVILEGES>() as DWORD;

    status = unsafe {
        AdjustTokenPrivileges(
            token_handle,
            0,
            &mut token_privilege,
            size,
            null_mut(),
            null_mut(),
        )
    };

    return status != 0;
}