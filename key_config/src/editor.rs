use crate::*;

pub struct Editor {
    hwnd: HWND,
    theme: isize,
    font: HFONT,
}

impl Editor {
    pub fn new(parent: HWND) -> windows::Result<Self> {
        let class_name = to_wchar("EDIT");
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(class_name.as_ptr() as _),
                PWSTR::NULL,
                WINDOW_STYLE::WS_CHILD | WINDOW_STYLE::WS_BORDER,
                0,
                0,
                1,
                1,
                parent,
                HMENU::NULL,
                HINSTANCE::NULL,
                std::ptr::null_mut(),
            );
            let theme_name = to_wchar("TEXTSTYLE");
            let theme = OpenThemeData(hwnd, PWSTR(theme_name.as_ptr() as _));
            let mut font = HFONT::NULL;
            if theme != 0 {
                let mut log_font = LOGFONTW::default();
                let ret = GetThemeFont(
                    theme,
                    HDC::NULL,
                    4,
                    0,
                    THEME_PROPERTY_SYMBOL_ID::TMT_FONT.0 as _,
                    &mut log_font,
                )
                .ok();
                match ret {
                    Ok(_) => {
                        font = CreateFontIndirectW(&log_font);
                        SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as _), LPARAM(0));
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            } else {
                log::error!("{}", get_last_error().message());
            }
            Ok(Self { hwnd, theme, font })
        }
    }

    pub fn show(&mut self, rc: &RECT) {
        unsafe {
            MoveWindow(self.hwnd, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, TRUE);
            ShowWindow(self.hwnd, SHOW_WINDOW_CMD::SW_SHOW);
            SetFocus(self.hwnd);
        }
    }

    pub fn hide(&mut self) {
        unsafe {
            SetFocus(GetParent(self.hwnd));
            ShowWindow(self.hwnd, SHOW_WINDOW_CMD::SW_HIDE);
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        unsafe {
            if !self.font.is_null() {
                DeleteObject(self.font);
            }
            if self.theme != 0 {
                CloseThemeData(self.theme).ok().ok();
            }
        }
    }
}
