::windows::include_bindings!();

pub mod wrapper {
    use super::Windows::Win32::{
        Debug::*, KeyboardAndMouseInput::*, Shell::*, SystemServices::*, WindowsAndMessaging::*,
    };

    #[inline]
    pub fn to_wchar(src: impl AsRef<str>) -> Vec<u16> {
        src.as_ref().encode_utf16().chain(Some(0)).collect()
    }

    #[inline]
    pub fn bstr(src: &[u8]) -> Vec<u8> {
        src.iter().cloned().chain(Some(0)).collect()
    }

    pub fn get_system_directory() -> std::path::PathBuf {
        unsafe {
            let mut buffer = Vec::new();
            buffer.resize(MAX_PATH as _, 0);
            SHGetFolderPathW(
                HWND::NULL,
                CSIDL_SYSTEM as _,
                HANDLE::NULL,
                0,
                PWSTR(buffer.as_mut_ptr()),
            )
            .unwrap();
            let len = buffer.iter().position(|&v| v == 0).unwrap();
            String::from_utf16_lossy(&buffer[..len]).into()
        }
    }

    pub fn get_module_path() -> std::path::PathBuf {
        unsafe {
            let mut buffer = vec![0; MAX_PATH as usize * 2];
            let size = GetModuleFileNameW(0, PWSTR(buffer.as_mut_ptr()), buffer.len() as _);
            String::from_utf16_lossy(&buffer[..size as _]).into()
        }
    }

    pub fn message_box(
        hwnd: Option<HWND>,
        text: impl AsRef<str>,
        caption: impl AsRef<str>,
        style: MESSAGEBOX_STYLE,
    ) -> MESSAGEBOX_RESULT {
        unsafe {
            let text = to_wchar(text.as_ref());
            let caption = to_wchar(caption.as_ref());
            MessageBoxW(
                hwnd.unwrap_or(HWND::NULL),
                PWSTR(text.as_ptr() as _),
                PWSTR(caption.as_ptr() as _),
                style,
            )
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    pub struct HookHandle(HHOOK);

    impl HookHandle {
        pub fn new(id: WINDOWS_HOOK_ID, f: Option<HOOKPROC>, thread_id: u32) -> Self {
            unsafe { Self(SetWindowsHookExA(id, f, HINSTANCE::NULL, thread_id)) }
        }
    }

    impl Drop for HookHandle {
        fn drop(&mut self) {
            unsafe {
                if !self.0.is_null() {
                    UnhookWindowsHookEx(self.0);
                }
            }
        }
    }

    #[inline]
    pub fn get_current_thread_id() -> u32 {
        unsafe { GetCurrentThreadId() }
    }

    #[inline]
    pub fn get_keyboard_state(v: &mut [u8]) {
        unsafe {
            debug_assert_eq!(v.len(), 256);
            GetKeyboardState(v.as_mut_ptr());
        }
    }

    #[inline]
    pub fn get_last_error() -> windows::HRESULT {
        unsafe { windows::HRESULT::from_win32(GetLastError().0) }
    }

    #[inline]
    pub fn enum_windows() -> Vec<HWND> {
        extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let windows = &mut *(lparam.0 as *mut Vec<HWND>);
                windows.push(hwnd);
                TRUE
            }
        }

        unsafe {
            let mut windows = vec![];
            if EnumWindows(Some(callback), LPARAM(&mut windows as *mut _ as _)) == BOOL(0) {
                vec![]
            } else {
                windows
            }
        }
    }

    #[inline]
    pub fn get_window_thread_process_id(hwnd: HWND) -> u32 {
        unsafe {
            let mut id = 0;
            GetWindowThreadProcessId(hwnd, &mut id);
            id
        }
    }

    #[inline]
    pub fn get_class_name(hwnd: HWND) -> String {
        unsafe {
            let mut buffer = vec![0; 256];
            let size = GetClassNameW(hwnd, PWSTR(buffer.as_mut_ptr()), buffer.len() as _);
            if size == 0 {
                return String::new();
            }
            String::from_utf16_lossy(&buffer[..size as usize])
        }
    }

    pub const LVN_ITEMCHANGED: u32 = -101i32 as _;
    pub const LVN_ITEMACTIVATE: u32 = -114i32 as _;
    pub const NM_CLICK: u32 = -2i32 as _;
    pub const NM_DBLCLK: u32 = -3i32 as _;
}
