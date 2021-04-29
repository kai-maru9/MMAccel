use crate::*;

pub const IDM_MENU_DETACH: u32 = 10;

pub struct PopupMenu {
    menu: HMENU,
    category: usize,
    item: usize,
}

impl PopupMenu {
    pub fn new() -> Self {
        unsafe {
            let menu = CreatePopupMenu();
            let text = to_wchar("解除");
            AppendMenuW(
                menu,
                MENU_ITEM_FLAGS::MF_STRING,
                IDM_MENU_DETACH as _,
                PWSTR(text.as_ptr() as _),
            );
            Self {
                menu,
                category: 0,
                item: 0,
            }
        }
    }

    #[inline]
    pub fn track(&mut self, window: &wita::Window, category: usize, item: usize, pt: wita::ScreenPosition) {
        unsafe {
            self.category = category;
            self.item = item;
            TrackPopupMenu(
                self.menu,
                TRACK_POPUP_MENU_FLAGS::TPM_LEFTALIGN | TRACK_POPUP_MENU_FLAGS::TPM_VCENTERALIGN,
                pt.x,
                pt.y,
                0,
                HWND(window.raw_handle() as _),
                std::ptr::null_mut(),
            );
        }
    }

    #[inline]
    pub fn category(&self) -> usize {
        self.category
    }

    #[inline]
    pub fn item(&self) -> usize {
        self.item
    }
}

impl Drop for PopupMenu {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.menu);
        }
    }
}
