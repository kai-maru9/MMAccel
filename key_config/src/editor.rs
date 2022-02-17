use crate::*;

fn theme_font(hwnd: HWND) -> windows::core::Result<HFONT> {
    unsafe {
        let theme_name = to_wchar("TEXTSTYLE");
        let theme = OpenThemeData(hwnd, PWSTR(theme_name.as_ptr() as _));
        if theme == 0 {
            return Err(get_last_error().into());
        }
        let log_font = GetThemeFont(theme, HDC(0), 4, 0, TMT_FONT.0 as _)?;
        let font = CreateFontIndirectW(&log_font);
        if font == HFONT(0) {
            return Err(get_last_error().into());
        }
        CloseThemeData(theme)?;
        Ok(font)
    }
}

pub struct EditResult {
    pub category: usize,
    pub item: usize,
    pub keys: Keys,
}

pub struct Editor {
    hwnd: HWND,
    font: Option<HFONT>,
    input_keys: Vec<u8>,
    result: Option<EditResult>,
}

impl Editor {
    pub fn new(parent: HWND) -> Result<Box<Self>, Error> {
        let class_name = to_wchar("EDIT");
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(class_name.as_ptr() as _),
                PWSTR::default(),
                WS_CHILD | WS_BORDER,
                0,
                0,
                1,
                1,
                parent,
                HMENU(0),
                HINSTANCE(0),
                std::ptr::null(),
            );
            let font = theme_font(hwnd);
            if let Ok(font) = font.as_ref() {
                SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as _), LPARAM(0));
            }
            let editor = Box::new(Self {
                hwnd,
                font: font.ok(),
                input_keys: vec![0; 256],
                result: None,
            });
            SetWindowSubclass(hwnd, Some(proc), 0, editor.as_ref() as *const _ as _);
            Ok(editor)
        }
    }

    #[inline]
    pub fn begin(&mut self, rc: &RECT, category: usize, item: usize, keys: &Keys) {
        unsafe {
            MoveWindow(self.hwnd, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, true);
            ShowWindow(self.hwnd, SW_SHOW);
            SetFocus(self.hwnd);
            let text = if !keys.is_empty() {
                to_wchar(keys.to_strings().join("+"))
            } else {
                to_wchar("")
            };
            SetWindowTextW(self.hwnd, PWSTR(text.as_ptr() as _));
            self.result = Some(EditResult {
                category,
                item,
                keys: keys.clone(),
            });
        }
    }

    #[inline]
    pub fn end(&mut self) -> Option<EditResult> {
        unsafe {
            SetFocus(GetParent(self.hwnd));
            ShowWindow(self.hwnd, SW_HIDE);
            self.result.take().and_then(|ret| {
                (ret.keys != Keys::from_slice(&[VK_SHIFT.0 as u32])
                    && ret.keys != Keys::from_slice(&[VK_CONTROL.0 as u32]))
                .then(|| ret)
            })
        }
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }

    #[inline]
    pub fn resize(&mut self) {
        unsafe {
            let font = theme_font(self.hwnd);
            if let Ok(font) = font {
                SendMessageW(self.hwnd, WM_SETFONT, WPARAM(font.0 as _), LPARAM(0));
                if let Some(font) = self.font {
                    DeleteObject(font);
                }
                self.font = Some(font);
            }
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        unsafe {
            if let Some(font) = self.font {
                DeleteObject(font);
            }
        }
    }
}

unsafe extern "system" fn proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data_ptr: usize,
) -> LRESULT {
    let editor = (data_ptr as *mut Editor).as_mut().unwrap();
    match msg {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let result = editor.result.as_mut().unwrap();
            get_keyboard_state(&mut editor.input_keys);
            result.keys.keyboard_state(&editor.input_keys);
            if !result.keys.is_empty() {
                let keys = to_wchar(result.keys.to_strings().join("+"));
                SetWindowTextW(editor.hwnd, PWSTR(keys.as_ptr() as _));
            }
            LRESULT(0)
        }
        WM_CHAR => LRESULT(0),
        WM_LBUTTONDOWN => {
            PostMessageW(
                GetParent(GetParent(editor.hwnd)),
                WM_KEY_CONFIG_EDIT_APPLY,
                WPARAM(0),
                LPARAM(0),
            );
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            PostMessageW(
                GetParent(GetParent(editor.hwnd)),
                WM_KEY_CONFIG_EDIT_CANCEL,
                WPARAM(0),
                LPARAM(0),
            );
            LRESULT(0)
        }
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}
