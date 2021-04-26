use bindings::wrapper::*;
use bindings::Windows::Win32::{SystemServices::*, WindowsAndMessaging::*};
use libloading::Library;
use once_cell::sync::OnceCell;

static mut D3D9: OnceCell<Library> = OnceCell::new();
static mut MME: OnceCell<Library> = OnceCell::new();
static mut MMACCEL: OnceCell<Library> = OnceCell::new();

fn error(msg: &str) {
    message_box(
        None,
        msg,
        "d3d9.dllエラー",
        MESSAGEBOX_STYLE::MB_OK | MESSAGEBOX_STYLE::MB_ICONERROR,
    );
}

#[inline]
fn mmaccel_run(base_addr: usize) {
    unsafe {
        if let Some(mmaccel) = MMACCEL.get() {
            let f = mmaccel.get::<unsafe fn(usize)>(b"mmaccel_run").unwrap();
            f(base_addr)
        }
    }
}

#[no_mangle]
pub extern "system" fn Direct3DCreate9(version: u32) -> *mut std::ffi::c_void {
    unsafe {
        if let Some(d3d9) = D3D9.get() {
            let f = d3d9
                .get::<unsafe fn(u32) -> *mut std::ffi::c_void>(b"Direct3DCreate9")
                .unwrap();
            f(version)
        } else {
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Direct3DCreate9Ex(version: u32, pp: *mut std::ffi::c_void) -> u32 {
    unsafe {
        if let Some(d3d9) = D3D9.get() {
            let f = d3d9
                .get::<unsafe fn(u32, *mut std::ffi::c_void) -> u32>(b"Direct3DCreate9Ex")
                .unwrap();
            f(version, pp)
        } else {
            E_FAIL.0
        }
    }
}

#[no_mangle]
pub extern "system" fn DllMain(_: HINSTANCE, reason: u32, _: *mut std::ffi::c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => unsafe {
            let path = get_module_path().parent().unwrap().to_path_buf();
            let d3d9 = Library::new(get_system_directory().join("d3d9.dll"));
            if let Ok(d3d9) = d3d9 {
                D3D9.set(d3d9).unwrap();
            } else {
                error("d3d9.dllを読み込めませんでした");
                return FALSE;
            }
            let mmaccel = Library::new(path.join("MMAccel").join("mmaccel.dll"));
            if let Ok(mmaccel) = mmaccel {
                MMACCEL.set(mmaccel).unwrap();
            } else {
                error("mmaccel.dllを読み込めませんでした。");
            }
            let mme = Library::new(path.join("MMHack.dll"));
            if let Ok(mme) = mme {
                MME.set(mme).unwrap();
            }
            let base_addr = GetModuleHandleW(PWSTR::NULL) as usize;
            mmaccel_run(base_addr);
        },
        DLL_PROCESS_DETACH => unsafe {
            MME.take();
            MMACCEL.take();
            D3D9.take();
        },
        _ => {}
    }
    TRUE
}
