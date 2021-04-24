use bindings::wrapper::*;
use bindings::Windows::Win32::{SystemServices::*, WindowsAndMessaging::*};
use libloading::Library;
use once_cell::sync::OnceCell;

static mut D3D9: OnceCell<Library> = OnceCell::new();
static mut MME: OnceCell<Library> = OnceCell::new();
static mut MMACCEL_EX: OnceCell<Library> = OnceCell::new();

fn error(msg: &str) {
    message_box(
        msg,
        "d3d9.dllエラー",
        MESSAGEBOX_STYLE::MB_OK | MESSAGEBOX_STYLE::MB_ICONERROR,
    );
}

#[inline]
fn mmaccel_ex_run(base_addr: usize) {
    unsafe {
        if let Some(mmaccel_ex) = MMACCEL_EX.get() {
            let f = mmaccel_ex
                .get::<unsafe fn(usize)>(b"mmaccel_ex_run")
                .unwrap();
            f(base_addr)
        }
    }
}

#[no_mangle]
pub extern "system" fn Direct3DCreate9(version: u32) -> *mut std::ffi::c_void {
    unsafe {
        let base_addr = GetModuleHandleW(PWSTR::NULL) as usize;
        mmaccel_ex_run(base_addr);
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
            let mmaccel_ex = Library::new(path.join("MMAccelEx").join("mmaccel_ex.dll"));
            if let Ok(mmaccel_ex) = mmaccel_ex {
                MMACCEL_EX.set(mmaccel_ex).unwrap();
            } else {
                error("mmaccel_ex.dllを読み込めませんでした。");
            }
            let mme = Library::new(path.join("MMHack.dll"));
            if let Ok(mme) = mme {
                MME.set(mme).unwrap();
            }
        },
        DLL_PROCESS_DETACH => unsafe {
            MME.take();
            MMACCEL_EX.take();
            D3D9.take();
        },
        _ => {}
    }
    TRUE
}
