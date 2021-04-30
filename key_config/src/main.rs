#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod editor;
mod popup_menu;
mod shortcut_list;
mod side_menu;
mod error;

use application::*;
use bindings::wrapper::*;
use bindings::Windows::Win32::{
    Controls::*, DisplayDevices::*, Gdi::*, HiDpi::*, KeyboardAndMouseInput::*, MenusAndResources::*, Shell::*,
    SystemServices::*, WindowsAndMessaging::*,
};
use editor::*;
use key_map::*;
use popup_menu::*;
use shortcut_list::*;
use side_menu::*;
use error::*;

fn error_mesage_box(text: impl AsRef<str>) {
    message_box(None, text, "MMAccel キー設定", MESSAGEBOX_STYLE::MB_OK | MESSAGEBOX_STYLE::MB_ICONERROR);
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        error_mesage_box(&info.to_string());
    }));
    env_logger::init();
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
