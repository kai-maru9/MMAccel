use crate::*;

pub struct ShortcutList {
    hwnd: HWND,
}

impl ShortcutList {
    pub fn new(
        parent: &wita::Window,
        pt: impl Into<wita::LogicalPosition<i32>>,
        size: impl Into<wita::LogicalSize<i32>>,
        columns_size: [i32; 2],
    ) -> Result<Self, Error> {
        let dpi = parent.dpi() as i32;
        let pt = pt.into().to_physical(dpi);
        let size = size.into().to_physical(dpi);
        let class_name = to_wchar("SysListView32");
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(class_name.as_ptr() as _),
                PWSTR::NULL,
                WINDOW_STYLE::WS_CHILD
                    | WINDOW_STYLE::WS_BORDER
                    | WINDOW_STYLE::WS_VISIBLE
                    | WINDOW_STYLE::WS_CLIPCHILDREN
                    | WINDOW_STYLE(LVS_REPORT),
                pt.x,
                pt.y,
                size.width,
                size.height,
                HWND(parent.raw_handle() as _),
                HMENU::NULL,
                HINSTANCE::NULL,
                std::ptr::null_mut(),
            );
            if hwnd == HWND::NULL {
                return Err(Error::hresult(get_last_error().into(), "CreateWindowEx"));
            }
            let ex_style = SendMessageW(hwnd, LVM_GETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(0)).0 as u32;
            let ex_style =
                ex_style | LVS_EX_DOUBLEBUFFER | LVS_EX_GRIDLINES | LVS_EX_FULLROWSELECT | LVS_EX_AUTOSIZECOLUMNS;
            SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(ex_style as _));
            let cx = columns_size[0] * dpi / 96;
            let text = to_wchar("機能");
            let column = LVCOLUMNW {
                mask: LVCOLUMNW_MASK::LVCF_WIDTH
                    | LVCOLUMNW_MASK::LVCF_FMT
                    | LVCOLUMNW_MASK::LVCF_MINWIDTH
                    | LVCOLUMNW_MASK::LVCF_TEXT,
                fmt: LVCOLUMNW_FORMAT::LVCFMT_LEFT,
                cx,
                cxMin: cx,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(0), LPARAM(&column as *const _ as _));
            let cx = columns_size[1] * dpi / 96;
            let text = to_wchar("キー");
            let column = LVCOLUMNW {
                mask: LVCOLUMNW_MASK::LVCF_WIDTH
                    | LVCOLUMNW_MASK::LVCF_FMT
                    | LVCOLUMNW_MASK::LVCF_MINWIDTH
                    | LVCOLUMNW_MASK::LVCF_TEXT,
                fmt: LVCOLUMNW_FORMAT::LVCFMT_LEFT,
                cx,
                cxMin: cx,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(1), LPARAM(&column as *const _ as _));
            let cx = size.width as i32 - (columns_size.iter().sum::<i32>() + 5) * dpi / 96;
            let text = to_wchar("重複");
            let column = LVCOLUMNW {
                mask: LVCOLUMNW_MASK::LVCF_WIDTH | LVCOLUMNW_MASK::LVCF_FMT | LVCOLUMNW_MASK::LVCF_TEXT,
                fmt: LVCOLUMNW_FORMAT::LVCFMT_LEFT,
                cx,
                cxMin: cx,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(2), LPARAM(&column as *const _ as _));
            let theme = to_wchar("Explorer");
            SetWindowTheme(hwnd, PWSTR(theme.as_ptr() as _), PWSTR::NULL).ok().ok();
            Ok(Self { hwnd })
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        unsafe { SendMessageW(self.hwnd, LVM_GETITEMCOUNT, WPARAM(0), LPARAM(0)).0 as _ }
    }

    #[inline]
    pub fn keys_rect(&self, index: usize) -> Option<RECT> {
        unsafe {
            let mut rc = RECT {
                left: LVIR_BOUNDS as _,
                top: 1,
                ..Default::default()
            };
            let ret = SendMessageW(
                self.hwnd,
                LVM_GETSUBITEMRECT,
                WPARAM(index as _),
                LPARAM(&mut rc as *mut _ as _),
            );
            if ret == LRESULT(0) {
                None
            } else {
                Some(rc)
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            SendMessageW(self.hwnd, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
        }
    }

    #[inline]
    pub fn push(&mut self, name: impl AsRef<str>, keys: Option<&Keys>) {
        unsafe {
            let name = to_wchar(name.as_ref());
            let item = LVITEMW {
                iItem: self.size() as _,
                iSubItem: 0,
                mask: LVIF_TEXT,
                pszText: PWSTR(name.as_ptr() as _),
                cchTextMax: name.len() as _,
                ..Default::default()
            };
            SendMessageW(self.hwnd, LVM_INSERTITEMW, WPARAM(0), LPARAM(&item as *const _ as _));
            self.set_keys(self.size() - 1, keys);
        }
    }

    #[inline]
    pub fn set_keys(&mut self, index: usize, keys: Option<&Keys>) {
        unsafe {
            let text = if let Some(keys) = keys {
                to_wchar(&keys.to_strings().join("+"))
            } else {
                to_wchar("")
            };
            let item = LVITEMW {
                iItem: index as _,
                iSubItem: 1,
                mask: LVIF_TEXT,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&item as *const _ as _));
        }
    }

    #[inline]
    pub fn set_dup(&mut self, index: usize, text: Option<&str>) {
        unsafe {
            let text = if let Some(text) = text {
                to_wchar(text)
            } else {
                to_wchar("")
            };
            let item = LVITEMW {
                iItem: index as _,
                iSubItem: 2,
                mask: LVIF_TEXT,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&item as *const _ as _));
        }
    }

    #[inline]
    pub fn resize(
        &mut self,
        position: wita::LogicalPosition<i32>,
        size: wita::LogicalSize<i32>,
        columns_size: [i32; 2],
    ) {
        unsafe {
            let dpi = GetDpiForWindow(self.hwnd) as i32;
            let position = position.to_physical(dpi as _);
            let size = size.to_physical(dpi as _);
            SetWindowPos(
                self.hwnd,
                HWND::NULL,
                position.x,
                position.y,
                size.width as _,
                size.height as _,
                SET_WINDOW_POS_FLAGS::SWP_NOZORDER,
            );
            for (i, &cx) in columns_size.iter().enumerate() {
                let cx = cx * dpi / 96;
                let column = LVCOLUMNW {
                    mask: LVCOLUMNW_MASK::LVCF_WIDTH | LVCOLUMNW_MASK::LVCF_MINWIDTH,
                    cx,
                    cxMin: cx,
                    ..Default::default()
                };
                SendMessageW(self.hwnd, LVM_SETCOLUMNW, WPARAM(i), LPARAM(&column as *const _ as _));
            }
            let width = size.width - (columns_size.iter().sum::<i32>() + 5) * dpi / 96;
            SendMessageW(self.hwnd, LVM_SETCOLUMNWIDTH, WPARAM(2), LPARAM(width as _));
        }
    }

    #[inline]
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
}
