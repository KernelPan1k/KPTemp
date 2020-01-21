use winapi::shared::minwindef::DWORD;
use winapi::shared::windef::HMENU;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{BS_NOTIFY, BS_TEXT, DestroyWindow, WS_CHILD, WS_DISABLED, WS_VISIBLE};

use crate::gui::{Control, ControlT, ControlType};
use crate::gui::errors::KPTempErrors;
use crate::gui::windows_helper::{build_window, WindowParams};

#[derive(Clone)]
pub struct ButtonT<S: Clone + Into<String>> {
    pub text: S,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub parent: HWND,
    pub font: Option<HWND>,
    pub h_menu: Option<HMENU>,
}

impl<S: Clone + Into<String>> ControlT for ButtonT<S> {
    fn build(&self) -> Result<Box<dyn Control>, KPTempErrors> {
        let flags: DWORD = WS_CHILD | BS_NOTIFY | BS_TEXT |
            if self.visible { WS_VISIBLE } else { 0 } |
            if self.disabled { WS_DISABLED } else { 0 };


        let params = WindowParams {
            title: self.text.clone().into(),
            class_name: "BUTTON",
            position: self.position.clone(),
            size: self.size.clone(),
            flags,
            ex_flags: Some(0),
            parent: self.parent,
            h_menu: self.h_menu,
        };

        match unsafe { build_window(params) } {
            Ok(h) => {
                Ok(Box::new(Button { handle: h }))
            }
            Err(_e) => Err(KPTempErrors::Button("Build windows".to_string()))
        }
    }
}

/**
    A standard button
*/
pub struct Button {
    handle: HWND
}

impl Button {}

impl Control for Button {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn control_type(&self) -> ControlType {
        ControlType::Button
    }

    fn free(&mut self) {
        unsafe { DestroyWindow(self.handle) };
    }
}