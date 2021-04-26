use crate::*;
use bindings::Windows::Win32::MenusAndResources::*;

pub const IDR_MMACCEL_MENU: u16 = 50000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum MenuId {
    LaunchConfig = IDR_MMACCEL_MENU + 1,
    ModelPallete,
    Version,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuItem {
    LaunchConfig,
    ModelPallete(bool),
    Version,
}

#[inline]
fn add_item(m: HMENU, index: u32, id: MenuId, name: &str) -> u32 {
    unsafe {
        let name = to_wchar(name);
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
            fMask: MENU_ITEM_MASK::MIIM_TYPE | MENU_ITEM_MASK::MIIM_ID,
            fType: MENU_ITEM_TYPE::MFT_STRING,
            dwTypeData: PWSTR(name.as_ptr() as _),
            wID: id as _,
            ..Default::default()
        };
        InsertMenuItemW(m, index, TRUE, &mut info);
        index + 1
    }
}

#[inline]
fn separate(m: HMENU, index: u32) -> u32 {
    unsafe {
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
            fMask: MENU_ITEM_MASK::MIIM_TYPE,
            fType: MENU_ITEM_TYPE::MFT_SEPARATOR,
            ..Default::default()
        };
        InsertMenuItemW(m, index, TRUE, &mut info);
        index + 1
    }
}

#[inline]
fn is_checked_item(m: HMENU, id: MenuId) -> bool {
    unsafe {
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
            fMask: MENU_ITEM_MASK::MIIM_STATE,
            ..Default::default()
        };
        GetMenuItemInfoW(m, id as _, FALSE, &mut info);
        (info.fState & MENU_ITEM_STATE::MFS_CHECKED) == MENU_ITEM_STATE::MFS_CHECKED
    }
}

#[inline]
fn set_check_item(m: HMENU, id: MenuId, checked: bool) {
    unsafe {
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
            fMask: MENU_ITEM_MASK::MIIM_STATE,
            fState: if checked {
                MENU_ITEM_STATE::MFS_CHECKED
            } else {
                MENU_ITEM_STATE::MFS_UNCHECKED
            },
            ..Default::default()
        };
        SetMenuItemInfoW(m, id as _, FALSE, &mut info);
    }
}

pub struct Menu {
    m: HMENU,
}

impl Menu {
    pub fn new(hwnd: HWND) -> Self {
        unsafe {
            let wnd_menu = GetMenu(hwnd);
            let m = CreatePopupMenu();

            let name = to_wchar("MMAccel");
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_TYPE
                    | MENU_ITEM_MASK::MIIM_SUBMENU
                    | MENU_ITEM_MASK::MIIM_ID,
                fType: MENU_ITEM_TYPE::MFT_STRING,
                dwTypeData: PWSTR(name.as_ptr() as _),
                hSubMenu: m,
                ..Default::default()
            };
            InsertMenuItemW(wnd_menu, IDR_MMACCEL_MENU as _, FALSE, &mut info);

            let index = 0;
            let index = add_item(m, index, MenuId::LaunchConfig, "キー設定");
            let index = separate(m, index);
            add_item(m, index, MenuId::Version, "バージョン情報");

            DrawMenuBar(hwnd);
            Self { m }
        }
    }

    pub fn recv_command(&self, wparam: WPARAM) -> Option<MenuItem> {
        if ((wparam.0 >> 16) & 0xffff) == 0 {
            let id = (wparam.0 & 0xffff) as u16;
            match id {
                _ if id == MenuId::LaunchConfig as u16 => Some(MenuItem::LaunchConfig),
                _ if id == MenuId::ModelPallete as u16 => {
                    let b = !is_checked_item(self.m, MenuId::ModelPallete);
                    set_check_item(self.m, MenuId::ModelPallete, b);
                    Some(MenuItem::ModelPallete(b))
                }
                _ if id == MenuId::Version as u16 => Some(MenuItem::Version),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.m);
        }
    }
}
