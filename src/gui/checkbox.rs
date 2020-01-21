use winapi::shared::minwindef::{DWORD, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{BM_GETCHECK, BM_SETCHECK, BS_AUTO3STATE, BS_AUTOCHECKBOX, BS_NOTIFY, BS_TEXT, BST_CHECKED, BST_INDETERMINATE, BST_UNCHECKED, DestroyWindow, SendMessageW, WS_CHILD, WS_DISABLED, WS_VISIBLE};

use crate::gui::{CheckState, Control, ControlT, ControlType};
use crate::gui::errors::KPTempErrors;
use crate::gui::windows_helper::{build_window, WindowParams};

#[derive(Clone)]
pub struct CheckBoxT<S: Clone + Into<String>> {
    pub text: S,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub parent: HWND,
    pub checkstate: CheckState,
    pub tristate: bool,
    pub font: Option<HWND>,
}

impl<S: Clone + Into<String>> ControlT for CheckBoxT<S> {
    fn build(&self) -> Result<Box<dyn Control>, KPTempErrors> {
        let flags: DWORD = WS_CHILD | BS_NOTIFY | BS_TEXT |
            if self.visible { WS_VISIBLE } else { 0 } |
            if self.disabled { WS_DISABLED } else { 0 } |
            if self.tristate { BS_AUTO3STATE } else { BS_AUTOCHECKBOX };

        let params = WindowParams {
            title: self.text.clone().into(),
            class_name: "BUTTON",
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
                    set_checkstate(h, &self.checkstate);
                }
                Ok(Box::new(CheckBox { handle: h }))
            }
            Err(_e) => Err(KPTempErrors::Checkbox("Error Checkbox build".to_string()))
        }
    }
}

/**
    A standard checkbox
*/
pub struct CheckBox {
    handle: HWND
}

impl CheckBox {}

impl Control for CheckBox {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn control_type(&self) -> ControlType {
        ControlType::CheckBox
    }

    fn free(&mut self) {
        unsafe { DestroyWindow(self.handle) };
    }
}

pub unsafe fn set_checkstate(handle: HWND, check: &CheckState) {
    let check_state = match check {
        &CheckState::Checked => BST_CHECKED,
        &CheckState::Indeterminate => BST_INDETERMINATE,
        &CheckState::Unchecked => BST_UNCHECKED
    };

    SendMessageW(handle, BM_SETCHECK, check_state as WPARAM, 0);
}

pub unsafe fn get_checkstate(handle: HWND) -> CheckState {
    match SendMessageW(handle, BM_GETCHECK, 0, 0) as usize {
        BST_CHECKED => CheckState::Checked,
        BST_UNCHECKED => CheckState::Unchecked,
        _ => CheckState::Indeterminate
    }
}
