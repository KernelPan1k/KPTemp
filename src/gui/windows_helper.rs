use std::mem;
use std::ptr::{null, null_mut};

use winapi::ctypes::c_int;
use winapi::shared::minwindef::{DWORD, HINSTANCE};
use winapi::shared::windef::{HBRUSH, HMENU, HWND, RECT};
use winapi::shared::winerror::ERROR_CLASS_ALREADY_EXISTS;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{COLOR_WINDOW, CreateWindowExW, CW_USEDEFAULT, EnableWindow, GetClientRect, GetDesktopWindow, GetWindowRect, IDI_APPLICATION, LoadCursorW, LoadIconW, RegisterClassW, SetWindowPos, SetWindowTextW, SWP_NOMOVE, SWP_NOZORDER, WNDCLASSW, WS_EX_COMPOSITED};

use crate::gui::errors::KPTempErrors;
use crate::gui::windows::window_proc;
use crate::utils::to_utf16;

pub const WINDOW_CLASS_NAME: &'static str = "KP_TEMP_BUILTIN_WINDOW";
pub const CENTER_POSITION: c_int = CW_USEDEFAULT + 1;

pub struct WindowParams<S1: Into<String>, S2: Into<String>> {
    pub title: S1,
    pub class_name: S2,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub flags: DWORD,
    pub ex_flags: Option<DWORD>,
    pub parent: HWND,
    pub h_menu: Option<HMENU>,
}

pub unsafe fn build_sysclass() -> Result<(), KPTempErrors> {
    let class_name = to_utf16(WINDOW_CLASS_NAME);
    let hmod = GetModuleHandleW(null_mut());

    if hmod.is_null() {
        return Err(KPTempErrors::SystemClassCreation);
    }

    let class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hmod,
        hIcon: LoadIconW(0 as HINSTANCE, IDI_APPLICATION),
        hCursor: LoadCursorW(0 as HINSTANCE, IDI_APPLICATION),
        hbrBackground: COLOR_WINDOW as HBRUSH,
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
    };

    let class_token: u16 = RegisterClassW(&class);

    if class_token == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
        Err(KPTempErrors::SystemClassCreation)
    } else {
        Ok(())
    }
}

pub unsafe fn build_window<S1: Into<String>, S2: Into<String>>(p: WindowParams<S1, S2>) -> Result<HWND, KPTempErrors> {
    let hmod = GetModuleHandleW(null_mut());

    if hmod.is_null() {
        return Err(KPTempErrors::WindowCreationFail);
    }

    let class_name = to_utf16(p.class_name.into().as_ref());
    let window_name = to_utf16(p.title.into().as_ref());

    let px = match p.position.0 {
        CENTER_POSITION => {
            let mut rect: RECT = mem::zeroed();

            let parent = if p.parent.is_null() {
                GetDesktopWindow()
            } else {
                p.parent
            };

            GetWindowRect(parent, &mut rect);

            (rect.right / 2) - ((p.size.0 / 2) as i32)
        }

        x => x
    };

    let py = match p.position.1 {
        CENTER_POSITION => {
            let mut rect: RECT = mem::zeroed();

            let parent = if p.parent.is_null() {
                GetDesktopWindow()
            } else {
                p.parent
            };

            GetWindowRect(parent, &mut rect);

            (rect.bottom / 2) - ((p.size.1 / 2) as i32)
        }
        y => y
    };

    let ex_flags = match p.ex_flags {
        Some(ex) => ex,
        None => WS_EX_COMPOSITED
    };

    let h_menu = match p.h_menu {
        Some(h) => h,
        None => null_mut()
    };

    let handle = CreateWindowExW(
        ex_flags,
        class_name.as_ptr(),
        window_name.as_ptr(),
        p.flags,
        px,
        py,
        p.size.0 as i32,
        p.size.1 as i32,
        p.parent,
        h_menu,
        hmod,
        null_mut(),
    );

    if handle.is_null() {
        Err(KPTempErrors::WindowCreationFail)
    } else {
        fix_overlapped_window_size(handle, p.size);
        Ok(handle)
    }
}

unsafe fn fix_overlapped_window_size(handle: HWND, size: (u32, u32)) {
    let mut rect: RECT = mem::zeroed();

    GetClientRect(handle, &mut rect);

    let (w, h) = (size.0 as c_int, size.1 as c_int);
    let delta_width = w - rect.right;
    let delta_height = h - rect.bottom;

    SetWindowPos(
        handle, null_mut(),
        0,
        0,
        w + delta_width,
        h + delta_height,
        SWP_NOMOVE | SWP_NOZORDER,
    );
}

pub unsafe fn set_window_enabled(handle: HWND, enabled: i32) {
    EnableWindow(handle, enabled);
}

pub unsafe fn set_window_text(handle: HWND, text: &str) {
    let text = to_utf16(text);
    SetWindowTextW(handle, text.as_ptr());
}