use std::ptr::null_mut;
use std::thread;

use winapi::shared::minwindef::{LOWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::winuser::{
    DefWindowProcW,
    WM_COMMAND,
    WM_DESTROY,
    WS_CAPTION,
    WS_CLIPCHILDREN,
    WS_MAXIMIZEBOX,
    WS_MINIMIZEBOX,
    WS_OVERLAPPED,
    WS_SYSMENU,
    WS_VISIBLE,
};

use crate::clean::clean;
use crate::globals::{LABEL_HANDLE, PROGRESS_HANDLE, TOTAL_STEP};
use crate::gui::{CheckState, ControlT, HTextAlign, ProgressBarState};
use crate::gui::button::ButtonT;
use crate::gui::checkbox::{CheckBoxT, get_checkstate};
use crate::gui::events::dispatch_events;
use crate::gui::label::LabelT;
use crate::gui::progress_bar::{advance_progress_bar, ProgressBarT};
use crate::gui::windows_helper::{build_sysclass, build_window, CENTER_POSITION, set_window_enabled, set_window_text, WINDOW_CLASS_NAME, WindowParams};
use crate::process::kill_process;
use crate::utils::{exit_all, restart};

const BUTTON_EVENT: u16 = 1;
static mut STATE_RUNNING: bool = false;
static mut WINDOWS_OLD_HANDLE: HWND = null_mut();
static mut RUN_HANDLE: HWND = null_mut();

pub unsafe extern "system" fn window_proc(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        exit_all();
        return 1;
    } else if msg == WM_COMMAND {
        if LOWORD(w_param as u32) == BUTTON_EVENT {
            if STATE_RUNNING == false {
                STATE_RUNNING = true;
                set_window_enabled(RUN_HANDLE, 0);
                set_window_enabled(WINDOWS_OLD_HANDLE, 0);
                advance_progress_bar(PROGRESS_HANDLE, 1);
                set_window_text(LABEL_HANDLE, "Kill process ...");
                kill_process();
                advance_progress_bar(PROGRESS_HANDLE, 1);
                thread::spawn(move || {
                    let old_check: bool = get_checkstate(WINDOWS_OLD_HANDLE) == CheckState::Checked;
                    set_window_text(LABEL_HANDLE, "Start clean ...");
                    clean(old_check);
                    set_window_text(LABEL_HANDLE, "Restart ...");
                    restart();
                });

                return 1;
            }
        }
    }

    return DefWindowProcW(h_wnd, msg, w_param, l_param);
}

pub unsafe fn build_root_window() {
    match build_sysclass() {
        Err(e) => panic!("{:?}", e),
        _ => {}
    }

    let flags: u32 = WS_CLIPCHILDREN | WS_SYSMENU | WS_CAPTION | WS_OVERLAPPED | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE;

    let window_params = WindowParams {
        title: "KpTemp",
        class_name: WINDOW_CLASS_NAME,
        position: (CENTER_POSITION, CENTER_POSITION),
        size: (500, 180),
        flags,
        ex_flags: None,
        parent: null_mut(),
        h_menu: None,
    };

    let windows_handle = build_window(window_params).expect("Err");

    let label = LabelT {
        text: "KPTemp files temporary cleaner",
        position: (140, 10),
        size: (220, 20),
        visible: true,
        disabled: false,
        align: HTextAlign::Center,
        parent: windows_handle,
        font: None,
    };

    let windows_old = CheckBoxT {
        text: "Remove Windows.old",
        position: (170, 50),
        size: (160, 20),
        visible: true,
        disabled: false,
        parent: windows_handle,
        checkstate: CheckState::Unchecked,
        tristate: false,
        font: None,
    };

    let progressbar = ProgressBarT {
        position: (15, 80),
        size: (470, 25),
        visible: true,
        disabled: false,
        range: (0, TOTAL_STEP),
        value: 0,
        step: 0,
        state: ProgressBarState::Normal,
        vertical: false,
        parent: windows_handle,
    };

    let status_label = LabelT {
        text: "Ready ...",
        position: (15, 115),
        size: (470, 15),
        visible: true,
        disabled: false,
        align: HTextAlign::Left,
        parent: windows_handle,
        font: None,
    };

    let run_button = ButtonT {
        text: "Clean now",
        position: (175, 145),
        size: (150, 25),
        visible: true,
        disabled: false,
        parent: windows_handle,
        font: None,
        h_menu: Some(BUTTON_EVENT as HMENU),
    };

    label.build().expect("Fail");
    let label_handle = status_label.build().expect("Fail");
    let run_button_handle = run_button.build().expect("Fail");
    let windows_old_handle = windows_old.build().expect("Fail");
    let progressbar_handle = progressbar.build().expect("Fail");

    LABEL_HANDLE = label_handle.handle();
    RUN_HANDLE = run_button_handle.handle();
    WINDOWS_OLD_HANDLE = windows_old_handle.handle();
    PROGRESS_HANDLE = progressbar_handle.handle();

    dispatch_events()
}