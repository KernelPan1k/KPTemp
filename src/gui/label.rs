use winapi::shared::minwindef::DWORD;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{DestroyWindow, SS_CENTER, SS_LEFT, SS_NOPREFIX, SS_NOTIFY, WS_CHILD, WS_DISABLED, WS_VISIBLE};

use crate::gui::{Control, ControlT, ControlType, HTextAlign};
use crate::gui::errors::KPTempErrors;
use crate::gui::windows_helper::{build_window, WindowParams};

#[derive(Clone)]
pub struct LabelT<S: Clone + Into<String>> {
    pub text: S,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub align: HTextAlign,
    pub parent: HWND,
    pub font: Option<HWND>,
}

impl<S: Clone + Into<String>> ControlT for LabelT<S> {
    fn build(&self) -> Result<Box<dyn Control>, KPTempErrors> {
        let flags: DWORD = WS_CHILD | SS_NOTIFY | SS_NOPREFIX |
            if self.visible { WS_VISIBLE } else { 0 } |
            if self.disabled { WS_DISABLED } else { 0 } |
            match self.align {
                HTextAlign::Center => SS_CENTER,
                HTextAlign::Left => SS_LEFT,
            };

        let params = WindowParams {
            title: self.text.clone().into(),
            class_name: "STATIC",
            position: self.position.clone(),
            size: self.size.clone(),
            flags,
            ex_flags: Some(0),
            parent: self.parent,
            h_menu: None,
        };

        match unsafe { build_window(params) } {
            Ok(h) => {
                Ok(Box::new(Label { handle: h }))
            }
            Err(_e) => Err(KPTempErrors::Label("Error build".to_string()))
        }
    }
}

/**
    A standard label
*/
pub struct Label {
    handle: HWND
}

impl Label {}

impl Control for Label {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn control_type(&self) -> ControlType {
        ControlType::Label
    }

    fn free(&mut self) {
        unsafe { DestroyWindow(self.handle) };
    }
}