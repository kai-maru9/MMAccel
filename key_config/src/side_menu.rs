use crate::*;

pub struct SideMenu {
    hwnd: HWND,
}

impl SideMenu {
    pub fn new(
        parent: &wita::Window,
        pt: impl Into<wita::LogicalPosition<i32>>,
        size: impl Into<wita::LogicalSize<i32>>,
    ) -> anyhow::Result<Self> {
        let dpi = parent.dpi();
        let pt = pt.into().to_physical(dpi as _);
        let size = size.into().to_physical(dpi as _);
        let class_name = to_wchar("LISTBOX");
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(class_name.as_ptr() as _),
                PWSTR::NULL,
                WINDOW_STYLE::WS_CHILD | WINDOW_STYLE::WS_BORDER | WINDOW_STYLE::WS_VISIBLE | WINDOW_STYLE(LISTBOX_STYLE::LBS_NOTIFY.0),
                pt.x,
                pt.y,
                size.width,
                size.height,
                HWND(parent.raw_handle() as _),
                HMENU::NULL,
                HINSTANCE::NULL,
                std::ptr::null_mut(),
            );
            SetWindowSubclass(hwnd, Some(proc), hwnd.0 as _, hwnd.0 as _).ok()?;
            Ok(Self { hwnd })
        }
    }

    #[inline]
    pub fn current_index(&self) -> usize {
        unsafe { SendMessageW(self.hwnd, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as _ }
    }
    
    #[inline]
    pub fn set_index(&mut self, index: u32) {
        unsafe {
            SendMessageW(self.hwnd, LB_SETCURSEL, WPARAM(index as _), LPARAM(0));
        }
    }

    #[inline]
    pub fn push(&mut self, text: impl AsRef<str>) {
        let text = to_wchar(text.as_ref());
        unsafe {
            SendMessageW(self.hwnd, LB_ADDSTRING, WPARAM(0), LPARAM(text.as_ptr() as _));
        }
    }

    #[inline]
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
}

extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _id: usize, _data_ptr: usize) -> LRESULT {
    unsafe {
        match msg {
            _ => DefSubclassProc(hwnd, msg, wparam, lparam),
        }
    }
}
