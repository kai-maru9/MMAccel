use crate::*;

pub struct ShortcutList {
    hwnd: HWND,
}

impl ShortcutList {
    pub fn new(
        parent: &wita::Window,
        pt: impl Into<wita::LogicalPosition<i32>>,
        size: impl Into<wita::LogicalSize<i32>>,
    ) -> windows::Result<Self> {
        let dpi = parent.dpi();
        let pt = pt.into().to_physical(dpi as _);
        let size = size.into().to_physical(dpi as _);
        let class_name = to_wchar("SysListView32");
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(class_name.as_ptr() as _),
                PWSTR::NULL,
                WINDOW_STYLE::WS_CHILD
                    | WINDOW_STYLE::WS_BORDER
                    | WINDOW_STYLE::WS_VISIBLE
                    | WINDOW_STYLE(LVS_REPORT)
                    | WINDOW_STYLE(LVS_NOCOLUMNHEADER),
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
                return Err(get_last_error().into());
            }
            let ex_style = SendMessageW(hwnd, LVM_GETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(0)).0 as u32;
            let ex_style = ex_style | LVS_EX_DOUBLEBUFFER | LVS_EX_GRIDLINES | LVS_EX_FULLROWSELECT | LVS_EX_AUTOSIZECOLUMNS;
            SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(ex_style as _));
            let column = LVCOLUMNW {
                mask: LVCOLUMNW_mask::LVCF_WIDTH | LVCOLUMNW_mask::LVCF_FMT,
                fmt: LVCOLUMNW_fmt::LVCFMT_LEFT | LVCOLUMNW_fmt::LVCFMT_FIXED_WIDTH,
                cx: size.width / 2,
                ..Default::default()
            };
            for i in 0..2 {
                SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(i), LPARAM(&column as *const _ as _));
            }
            Ok(Self { hwnd })
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        unsafe { SendMessageW(self.hwnd, LVM_GETITEMCOUNT, WPARAM(0), LPARAM(0)).0 as _ }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            SendMessageW(self.hwnd, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
        }
    }

    #[inline]
    pub fn push(&mut self, text: impl AsRef<str>) {
        unsafe {
            let text = to_wchar(text.as_ref());
            let item = LVITEMW {
                iItem: self.size() as _,
                iSubItem: 0,
                mask: LVIF_TEXT,
                pszText: PWSTR(text.as_ptr() as _),
                cchTextMax: text.len() as _,
                ..Default::default()
            };
            SendMessageW(self.hwnd, LVM_INSERTITEMW, WPARAM(0), LPARAM(&item as *const _ as _));
        }
    }
}
