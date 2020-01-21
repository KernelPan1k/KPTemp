use winapi::shared::minwindef::{DWORD, LPARAM, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::commctrl::{PBM_DELTAPOS, PBM_SETPOS, PBM_SETRANGE32, PBM_SETSTATE, PBM_SETSTEP, PBS_VERTICAL, PBST_NORMAL};
use winapi::um::winuser::{DestroyWindow, SendMessageW, WS_CHILD, WS_DISABLED, WS_VISIBLE};

use crate::gui::{Control, ControlT, ControlType, ProgressBarState};
use crate::gui::errors::KPTempErrors;
use crate::gui::windows_helper::{build_window, WindowParams};

#[derive(Clone)]
pub struct ProgressBarT {
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub range: (u32, u32),
    pub value: u32,
    pub step: u32,
    pub state: ProgressBarState,
    pub vertical: bool,
    pub parent: HWND,
}

impl ControlT for ProgressBarT {
    fn build(&self) -> Result<Box<dyn Control>, KPTempErrors> {
        if self.range.1 <= self.range.0 {
            let msg = "The progress bar range maximum value must be greater than the minimum value";
            return Err(KPTempErrors::ProgressBar(msg.to_string()));
        }

        let flags: DWORD = WS_CHILD |
            if self.visible { WS_VISIBLE } else { 0 } |
            if self.disabled { WS_DISABLED } else { 0 } |
            if self.vertical { PBS_VERTICAL } else { 0 };

        let params = WindowParams {
            title: "",
            class_name: "msctls_progress32",
            position: self.position.clone(),
            size: self.size.clone(),
            flags,
            ex_flags: Some(0),
            parent: self.parent,
            h_menu: None,
        };

        match unsafe { build_window(params) } {
            Ok(h) => {
                unsafe {
                    set_range(h, self.range.0, self.range.1);
                    set_step(h, self.step);
                    set_value(h, self.value);
                    set_state(h, &self.state);
                }
                Ok(Box::new(ProgressBar { handle: h }))
            }
            Err(_e) => Err(KPTempErrors::ProgressBar("Error build".to_string()))
        }
    }
}

/**
    A standard progress bar
*/
pub struct ProgressBar {
    handle: HWND
}

impl ProgressBar {}

impl Control for ProgressBar {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn control_type(&self) -> ControlType {
        ControlType::ProgressBar
    }

    fn free(&mut self) {
        unsafe { DestroyWindow(self.handle) };
    }
}


unsafe fn set_range(handle: HWND, min: u32, max: u32) {
    SendMessageW(handle, PBM_SETRANGE32, min as WPARAM, max as LPARAM);
}

unsafe fn set_step(handle: HWND, step: u32) {
    SendMessageW(handle, PBM_SETSTEP, step as WPARAM, 0);
}

unsafe fn set_value(handle: HWND, val: u32) {
    SendMessageW(handle, PBM_SETPOS, val as WPARAM, 0);
}

unsafe fn set_state(handle: HWND, state: &ProgressBarState) {
    let state = match state {
        &ProgressBarState::Normal => PBST_NORMAL,
    };

    SendMessageW(handle, PBM_SETSTATE, state as WPARAM, 0);
}

pub fn advance_progress_bar(handle: HWND, amount: u32) {
    unsafe { SendMessageW(handle, PBM_DELTAPOS, amount as WPARAM, 0); }
}