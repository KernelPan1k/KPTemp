use winapi::shared::windef::HWND;

use crate::gui::errors::KPTempErrors;

pub mod errors;
pub mod windows_helper;
pub mod windows;
pub mod events;
pub mod button;
pub mod progress_bar;
pub mod label;
pub mod checkbox;

#[derive(Clone, Debug)]
pub enum ControlType {
    Button,
    CheckBox,
    Label,
    ProgressBar,
    Undefined,
}

#[derive(PartialEq, Debug, Clone)]
pub enum HTextAlign {
    Center,
    Left,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CheckState {
    Checked,
    Unchecked,
    Indeterminate,
}

pub trait ControlT<> {
    fn build(&self) -> Result<Box<dyn Control>, KPTempErrors>;
}

pub trait Control {
    fn handle(&self) -> HWND;
    fn control_type(&self) -> ControlType { ControlType::Undefined }
    fn free(&mut self) {}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ProgressBarState {
    Normal,
}