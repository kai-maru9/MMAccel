#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::missing_safety_doc)]

mod context;
mod handler;
mod injection;
mod menu;
mod mmd;
mod mmd_map;

use bindings::wrapper::*;
use bindings::Windows::Win32::{Debug::*, KeyboardAndMouseInput::*, SystemServices::*, WindowsAndMessaging::*};
use context::*;
use injection::*;
use menu::*;
use once_cell::sync::OnceCell;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();

fn error(msg: &str) {
    message_box(
        None,
        msg,
        "MMAccelエラー",
        MESSAGEBOX_STYLE::MB_OK | MESSAGEBOX_STYLE::MB_ICONERROR,
    );
}

unsafe extern "system" fn hook_call_window_proc_ret(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 || code != HC_ACTION as i32 {
        return CallNextHookEx(HHOOK::NULL, code, wparam, lparam);
    }
    CONTEXT
        .get_mut()
        .unwrap()
        .call_window_proc_ret(&*(lparam.0 as *const CWPRETSTRUCT));
    CallNextHookEx(HHOOK::NULL, code, wparam, lparam)
}

unsafe extern "system" fn hook_get_message(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HHOOK::NULL, code, wparam, lparam);
    }
    CONTEXT.get_mut().unwrap().get_message(&mut *(lparam.0 as *mut MSG));
    CallNextHookEx(HHOOK::NULL, code, wparam, lparam)
}

unsafe extern "system" fn proxy_get_key_state(vk: i32) -> i16 {
    if let Some(ret) = CONTEXT.get().unwrap().get_key_state(vk as _) {
        ret as _
    } else {
        GetKeyState(vk)
    }
}

#[no_mangle]
pub unsafe extern "system" fn mmaccel_run(base_addr: usize) {
    env_logger::init();
    log::debug!("mmaccel_run");
    if let Ok(ctx) = Context::new() {
        CONTEXT.set(ctx).ok();
    } else {
        error("MMAccelの読み込みに失敗しました");
        return;
    }
    let user32 = image_import_desc(base_addr, b"user32.dll");
    if user32.is_err() {
        error("MMAccelの読み込みに失敗しました");
        return;
    }
    let user32 = user32.unwrap();
    let functions: &[(&[u8], u64)] = &[(b"GetKeyState", proxy_get_key_state as u64)];
    if inject_functions(base_addr, &user32, &functions).is_err() {
        error("MMAccelの読み込みに失敗しました");
    }
}
