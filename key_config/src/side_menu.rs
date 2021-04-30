use crate::*;

pub struct SideMenu {
    hwnd: HWND,
}

impl SideMenu {
    pub fn new(
        parent: &wita::Window,
        pt: impl Into<wita::LogicalPosition<i32>>,
        size: impl Into<wita::LogicalSize<i32>>,
    ) -> Result<Self, Error> {
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
                    | WINDOW_STYLE(LVS_SHOWSELALWAYS)
                    | WINDOW_STYLE(LVS_SINGLESEL)
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
            let ex_style = SendMessageW(hwnd, LVM_GETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(0)).0 as u32;
            let ex_style = ex_style | LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT | LVS_EX_AUTOSIZECOLUMNS;
            SendMessageW(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, WPARAM(0), LPARAM(ex_style as _));
            let column = LVCOLUMNW {
                mask: LVCOLUMNW_MASK::LVCF_WIDTH | LVCOLUMNW_MASK::LVCF_FMT,
                fmt: LVCOLUMNW_FORMAT::LVCFMT_LEFT,
                cx: size.width,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTCOLUMNW, WPARAM(0), LPARAM(&column as *const _ as _));
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
    pub fn current_index(&self) -> usize {
        unsafe {
            SendMessageW(
                self.hwnd,
                LVM_GETNEXTITEM,
                WPARAM(std::usize::MAX),
                LPARAM((LVNI_ALL | LVNI_SELECTED) as _),
            )
            .0 as _
        }
    }

    #[inline]
    pub fn set_index(&mut self, index: u32) {
        unsafe {
            const STATES: u32 = LVIS_SELECTED | LVIS_FOCUSED;
            let item = LVITEMW {
                iItem: index as _,
                mask: LVIF_STATE,
                stateMask: STATES,
                state: STATES,
                ..Default::default()
            };
            SendMessageW(self.hwnd, LVM_SETITEMW, WPARAM(0), LPARAM(&item as *const _ as _));
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

    #[inline]
    pub fn resize(&mut self, position: wita::LogicalPosition<i32>, size: wita::LogicalSize<i32>) {
        unsafe {
            let dpi = GetDpiForWindow(self.hwnd);
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
        }
    }

    #[inline]
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
}
