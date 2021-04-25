#![allow(clippy::fn_to_numeric_cast)]

mod context;
mod injection;
mod mmd_map;
mod menu;
mod mmd;

use bindings::wrapper::*;
use bindings::Windows::Win32::{
    Debug::*, KeyboardAndMouseInput::*, SystemServices::*, WindowsAndMessaging::*,
};
use context::*;
use injection::*;
use menu::*;
use once_cell::sync::OnceCell;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();

fn error(msg: &str) {
    message_box(
        msg,
        "MMAccelエラー",
        MESSAGEBOX_STYLE::MB_OK | MESSAGEBOX_STYLE::MB_ICONERROR,
    );
}

extern "system" fn hook_call_window_proc_ret(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code < 0 || code != HC_ACTION as i32 {
            return CallNextHookEx(HHOOK::NULL, code, wparam, lparam);
        }
        CONTEXT
            .get_mut()
            .unwrap()
            .call_window_proc_ret(&*(lparam.0 as *const CWPRETSTRUCT));
        CallNextHookEx(HHOOK::NULL, code, wparam, lparam)
    }
}

extern "system" fn hook_get_message(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code < 0 {
            return CallNextHookEx(HHOOK::NULL, code, wparam, lparam);
        }
        CONTEXT
            .get_mut()
            .unwrap()
            .get_message(&mut *(lparam.0 as *mut MSG));
        CallNextHookEx(HHOOK::NULL, code, wparam, lparam)
    }
}

extern "system" fn proxy_get_key_state(vk: i32) -> i16 {
    unsafe {
        if let Some(ret) = CONTEXT.get().unwrap().get_key_state(vk as u32) {
            ret
        } else {
            GetKeyState(vk)
        }
    }
}

#[no_mangle]
pub extern "system" fn mmaccel_run(base_addr: usize) {
    env_logger::init();
    log::debug!("mmaccel_run");
    unsafe {
        CONTEXT.set(Context::new()).ok();
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
}
