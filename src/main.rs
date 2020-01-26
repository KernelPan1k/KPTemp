#![windows_subsystem = "windows"]

extern crate chrono;
extern crate pretty_bytes;
extern crate walkdir;
extern crate winapi;

use crate::gui::windows::build_root_window;
use crate::utils::eula;

mod gui;
mod clean;
mod process;
mod privilege;
mod globals;
mod utils;

trait Ignore: Sized {
    fn ignore(self) -> () {}
}

impl<T, E> Ignore for Result<T, E> {}


fn main() {
    eula();
    unsafe { build_root_window(); }
}
