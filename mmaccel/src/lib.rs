// mmaccel/src/lib.rs

// configモジュールを追加
mod backup;
mod config;
mod context;
mod file_monitor;
mod handler;
mod injection;
mod menu;
mod mmd;
mod mmd_map;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use context::Context;
use windows::Win32::Foundation::{BOOL, HINSTANCE, TRUE};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

static mut CONTEXT: Option<Context> = None;
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(h_inst: HINSTANCE, reason: u32, _: u64) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        if IS_INITIALIZED.load(Ordering::Relaxed) {
            return TRUE;
        }
        IS_INITIALIZED.store(true, Ordering::Relaxed);
        thread::spawn(move || {
            // 設定ファイルを読み込む
            let config = config::load_config();
            // 定期バックアップスレッドを開始
            backup::start_backup_thread(config);

            let context = Context::new(h_inst);
            unsafe {
                injection::initialize(&context);
                file_monitor::initialize(&context);
                CONTEXT = Some(context);
            }
        });
    }
    TRUE
}
