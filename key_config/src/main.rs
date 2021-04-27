#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod shortcut_list;
mod side_menu;

use application::*;
use bindings::wrapper::*;
use bindings::Windows::Win32::{
    Controls::*, DisplayDevices::*, Gdi::*, HiDpi::*, MenusAndResources::*, Shell::*, SystemServices::*,
    WindowsAndMessaging::*,
};
use key_map::Keys;
use shortcut_list::*;
use side_menu::*;

fn main() {
    env_logger::init();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
