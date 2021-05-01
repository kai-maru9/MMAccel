#![allow(clippy::mem_discriminant_non_enum)]

use crate::*;
use bindings::Windows::Win32::MenusAndResources::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuItemType {
    Item,
    WithCheck(bool),
}

impl MenuItemType {
    #[inline]
    pub fn as_with_check(&self) -> Option<bool> {
        if let Self::WithCheck(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

pub trait MenuCommand: Sized {
    fn from_command(v: std::mem::Discriminant<Self>, item_type: MenuItemType) -> Self;
}

const ROOT_ID: u32 = 50000;

pub struct MenuBuilder<T> {
    hwnd: HWND,
    menu: HMENU,
    index: u32,
    id: u32,
    table: Vec<(std::mem::Discriminant<T>, std::mem::Discriminant<MenuItemType>)>,
}

impl<T: MenuCommand> MenuBuilder<T> {
    pub fn new(hwnd: HWND, name: impl AsRef<str>) -> Self {
        unsafe {
            let window_menu = GetMenu(hwnd);
            let menu = CreatePopupMenu();
            let name = to_wchar(name);
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_TYPE | MENU_ITEM_MASK::MIIM_SUBMENU | MENU_ITEM_MASK::MIIM_ID,
                fType: MENU_ITEM_TYPE::MFT_STRING,
                dwTypeData: PWSTR(name.as_ptr() as _),
                hSubMenu: menu,
                ..Default::default()
            };
            InsertMenuItemW(window_menu, ROOT_ID, FALSE, &mut info);
            Self {
                hwnd,
                menu,
                index: 0,
                id: 0,
                table: vec![],
            }
        }
    }

    #[inline]
    pub fn item(mut self, v: &T, text: impl AsRef<str>) -> Self {
        unsafe {
            let name = to_wchar(text);
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_TYPE | MENU_ITEM_MASK::MIIM_ID,
                fType: MENU_ITEM_TYPE::MFT_STRING,
                dwTypeData: PWSTR(name.as_ptr() as _),
                wID: ROOT_ID + self.id,
                ..Default::default()
            };
            InsertMenuItemW(self.menu, self.index, TRUE, &mut info);
            self.table
                .push((std::mem::discriminant(v), std::mem::discriminant(&MenuItemType::Item)));
            self.index += 1;
            self.id += 1;
            self
        }
    }

    #[inline]
    pub fn with_check(mut self, v: &T, text: impl AsRef<str>, checked: bool) -> Self {
        unsafe {
            let name = to_wchar(text);
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_TYPE | MENU_ITEM_MASK::MIIM_ID | MENU_ITEM_MASK::MIIM_STATE,
                fType: MENU_ITEM_TYPE::MFT_STRING,
                dwTypeData: PWSTR(name.as_ptr() as _),
                wID: ROOT_ID + self.id,
                fState: if checked {
                    MENU_ITEM_STATE::MFS_CHECKED
                } else {
                    MENU_ITEM_STATE::MFS_UNCHECKED
                },
                ..Default::default()
            };
            InsertMenuItemW(self.menu, self.index, TRUE, &mut info);
            self.table.push((
                std::mem::discriminant(v),
                std::mem::discriminant(&MenuItemType::WithCheck(false)),
            ));
            self.index += 1;
            self.id += 1;
            self
        }
    }

    #[inline]
    pub fn separator(mut self) -> Self {
        unsafe {
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_TYPE,
                fType: MENU_ITEM_TYPE::MFT_SEPARATOR,
                ..Default::default()
            };
            InsertMenuItemW(self.menu, self.index, TRUE, &mut info);
            self.index += 1;
            self
        }
    }

    #[inline]
    pub fn build(self) -> Menu<T> {
        unsafe {
            DrawMenuBar(self.hwnd);
            Menu {
                menu: self.menu,
                table: self.table,
            }
        }
    }
}

pub struct Menu<T> {
    menu: HMENU,
    table: Vec<(std::mem::Discriminant<T>, std::mem::Discriminant<MenuItemType>)>,
}

impl<T: MenuCommand> Menu<T> {
    #[inline]
    fn is_checked_item(&self, id: u32) -> bool {
        unsafe {
            let mut info = MENUITEMINFOW {
                cbSize: std::mem::size_of::<MENUITEMINFOW>() as _,
                fMask: MENU_ITEM_MASK::MIIM_STATE,
                ..Default::default()
            };
            GetMenuItemInfoW(self.menu, ROOT_ID + id, FALSE, &mut info);
            (info.fState & MENU_ITEM_STATE::MFS_CHECKED) == MENU_ITEM_STATE::MFS_CHECKED
        }
    }

    #[inline]
    fn set_check_item(&self, id: u32, checked: bool) {
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
            SetMenuItemInfoW(self.menu, ROOT_ID + id, FALSE, &mut info);
        }
    }

    pub fn recv_command(&self, wparam: WPARAM) -> Option<T> {
        if ((wparam.0 >> 16) & 0xffff) == 0 {
            let id = (wparam.0 & 0xffff) as i32 - ROOT_ID as i32;
            if id < 0 {
                return None;
            }
            let id = id as u32;
            let (t, item_type) = self.table[id as usize];
            let ret = if item_type == std::mem::discriminant(&MenuItemType::Item) {
                T::from_command(t, MenuItemType::Item)
            } else if item_type == std::mem::discriminant(&MenuItemType::WithCheck(false)) {
                let b = !self.is_checked_item(id);
                self.set_check_item(id, b);
                T::from_command(t, MenuItemType::WithCheck(b))
            } else {
                return None;
            };
            Some(ret)
        } else {
            None
        }
    }
}

impl<T> Drop for Menu<T> {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.menu);
        }
    }
}
