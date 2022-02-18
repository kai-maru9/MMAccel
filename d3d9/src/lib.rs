#![allow(clippy::missing_safety_doc)]

use libloading::Library;
use once_cell::sync::OnceCell;
use windows::Win32::{Foundation::*, System::LibraryLoader::*, System::SystemServices::*, UI::WindowsAndMessaging::*};
use wrapper::*;

static mut MSIMG32: OnceCell<Library> = OnceCell::new();
static mut D3D9: OnceCell<Library> = OnceCell::new();
static mut MME: OnceCell<Library> = OnceCell::new();
static mut MMACCEL: OnceCell<Library> = OnceCell::new();

fn error(msg: &str) {
    message_box(None, msg, "d3d9.dll エラー", MB_OK | MB_ICONERROR);
}

#[inline]
unsafe fn mmaccel_run(base_addr: usize) {
    if let Some(mmaccel) = MMACCEL.get() {
        let f = mmaccel.get::<unsafe fn(usize)>(b"mmaccel_run").unwrap();
        f(base_addr)
    }
}

#[inline]
unsafe fn mmaccel_end() {
    if let Some(mmaccel) = MMACCEL.get() {
        let f = mmaccel.get::<unsafe fn()>(b"mmaccel_end").unwrap();
        f()
    }
}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9(version: u32) -> *mut std::ffi::c_void {
    if let Some(d3d9) = D3D9.get() {
        let f = d3d9
            .get::<unsafe fn(u32) -> *mut std::ffi::c_void>(b"Direct3DCreate9")
            .unwrap();
        f(version)
    } else {
        std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9Ex(version: u32, pp: *mut std::ffi::c_void) -> u32 {
    if let Some(d3d9) = D3D9.get() {
        let f = d3d9
            .get::<unsafe fn(u32, *mut std::ffi::c_void) -> u32>(b"Direct3DCreate9Ex")
            .unwrap();
        f(version, pp)
    } else {
        E_FAIL.0 as _
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_: HINSTANCE, reason: u32, _: *mut std::ffi::c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            let path = get_module_path().parent().unwrap().to_path_buf();
            if path.join("MMPlus.dll").exists() {
                let msimg = Library::new(path.join("MSIMG32.dll"));
                if let Ok(msimg) = msimg {
                    MSIMG32.set(msimg).ok();
                }
            }
            let d3d9 = Library::new(get_system_directory().join("d3d9.dll"));
            match d3d9 {
                Ok(d3d9) => {
                    D3D9.set(d3d9).unwrap();
                }
                Err(e) => {
                    error(&format!("d3d9.dllを読み込めませんでした ({:?})", e));
                    return false.into();
                }
            }
            let mmaccel = Library::new(path.join("MMAccel").join("mmaccel.dll"));
            match mmaccel {
                Ok(mmaccel) => {
                    MMACCEL.set(mmaccel).unwrap();
                }
                Err(e) => {
                    error(&format!("mmaccel.dllを読み込めませんでした ({:?})", e));
                    return false.into();
                }
            }
            let mme = Library::new(path.join("MMHack.dll"));
            if let Ok(mme) = mme {
                MME.set(mme).unwrap();
            }
            let base_addr = GetModuleHandleW(PWSTR::default()).0 as usize;
            mmaccel_run(base_addr);
        }
        DLL_PROCESS_DETACH => {
            mmaccel_end();
            MME.take();
            MMACCEL.take();
            D3D9.take();
            MSIMG32.take();
        }
        _ => {}
    }
    true.into()
}
