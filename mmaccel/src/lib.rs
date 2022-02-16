#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::missing_safety_doc)]

mod context;
mod file_monitor;
mod handler;
mod injection;
mod menu;
mod mmd;
mod mmd_map;

/*
use bindings::Windows::Win32::{
    Debug::*, FileSystem::*, KeyboardAndMouseInput::*, Multimedia::*, SystemServices::*, WindowsAndMessaging::*,
    WindowsProgramming::*,
};
*/
use context::*;
use file_monitor::*;
use injection::*;
use log4rs::append::{console, console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use menu::*;
use once_cell::sync::OnceCell;
use windows::Win32::{
    Foundation::*, Media::*, Storage::FileSystem::*, System::Diagnostics::Debug::*, System::Memory::*,
    System::SystemServices::*, System::WindowsProgramming::*, System::IO::*, UI::Input::KeyboardAndMouse::*,
    UI::WindowsAndMessaging::*,
};
use wrapper::*;

static mut CONTEXT: OnceCell<Context> = OnceCell::new();

fn error(msg: &str) {
    message_box(None, msg, "MMAccelエラー", MB_OK | MB_ICONERROR);
}

unsafe extern "system" fn hook_call_window_proc_ret(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 || code != HC_ACTION as i32 {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }
    let ret = std::panic::catch_unwind(|| {
        CONTEXT
            .get_mut()
            .unwrap()
            .call_window_proc_ret(&*(lparam.0 as *const CWPRETSTRUCT));
    });
    if ret.is_err() {
        PostQuitMessage(1);
        return LRESULT(0);
    }
    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

unsafe extern "system" fn hook_get_message(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }
    let ret = std::panic::catch_unwind(|| {
        let msg = &mut *(lparam.0 as *mut MSG);
        CONTEXT.get_mut().unwrap().get_message(msg)
    });
    match ret {
        Ok(true) => LRESULT(0),
        Ok(false) => CallNextHookEx(HHOOK(0), code, wparam, lparam),
        Err(_) => {
            PostQuitMessage(1);
            LRESULT(0)
        }
    }
}

unsafe extern "system" fn proxy_get_key_state(vk: i32) -> i16 {
    if let Some(ret) = CONTEXT.get().unwrap().get_key_state(vk as _) {
        ret as _
    } else {
        GetKeyState(vk)
    }
}

fn build_logger(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
    const FORMAT: &str = "[{d(%Y-%m-%d %H:%M:%S%z)} {l} (({f}:{L}))] {m}\n";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(FORMAT)))
        .target(console::Target::Stderr)
        .build();
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(FORMAT)))
        .append(false)
        .build(path.join("MMAccel").join("mmaccel.log"))?;
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(log::LevelFilter::Debug),
        )?;
    log4rs::init_config(config)?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "system" fn mmaccel_run(base_addr: usize) {
    let path = get_module_path().parent().unwrap().to_path_buf();
    if let Err(e) = build_logger(&path) {
        error(&format!("MMAccelのログを取れません ({})", e));
    }
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(location) = info.location() {
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                format!("panic!!! {} ({}:{})", s, location.file(), location.line())
            } else {
                format!("panic!!! ({}:{})", location.file(), location.line())
            }
        } else {
            "panic!!! unknown".into()
        };
        log::error!("{}", &msg);
        error(&msg);
        log::info!("MMAccel panic");
    }));
    log::info!("MMAccel start");
    if let Ok(ctx) = Context::new(path) {
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

#[no_mangle]
pub unsafe extern "system" fn mmaccel_end() {
    CONTEXT.take();
    log::info!("MMAccel end");
}
