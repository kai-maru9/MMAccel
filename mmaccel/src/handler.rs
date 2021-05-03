use crate::mmd_map::*;
use crate::*;
use key_map::*;
use std::collections::HashMap;

pub struct Handler {
    input: Vec<u8>,
    input_keys: Keys,
    handler: HashMap<Keys, ItemKind>,
    key_states: HashMap<u32, bool>,
    folds: Vec<u32>,
    unfolds: Vec<u32>,
}

impl Handler {
    pub fn new(mmd_map: &MmdMap, key_map: KeyMap) -> Self {
        let mut key_states = HashMap::new();
        mmd_map
            .iter()
            .filter(|(_, item)| matches!(item.kind, mmd_map::ItemKind::Key(_)))
            .for_each(|(_, item)| {
                key_states.insert(item.kind.as_key().unwrap(), false);
            });
        let mut folds = vec![];
        let mut unfolds = vec![];
        mmd_map
            .iter()
            .filter(|(_, item)| matches!(item.kind, mmd_map::ItemKind::Fold(_, _)))
            .for_each(|(_, item)| {
                let (hide, show) = item.kind.as_fold().unwrap();
                folds.push(hide);
                unfolds.push(show);
            });
        let mut handler = HashMap::new();
        for (k, v) in key_map.into_iter() {
            if let Some(item) = mmd_map.get(&k) {
                handler.insert(v, item.kind);
            } else {
                log::error!("handler.insert error: {}", k);
            }
        }
        Self {
            input: vec![0; 256],
            input_keys: Keys::with_capacity(3),
            handler,
            key_states,
            folds,
            unfolds,
        }
    }

    pub fn key_down(&mut self, vk: u32, mmd_window: HWND) {
        fn handle(
            item: &ItemKind,
            key_states: &mut HashMap<u32, bool>,
            folds: &[u32],
            unfolds: &[u32],
            mmd_window: HWND,
        ) {
            match item {
                ItemKind::Key(k) => {
                    if let Some(ks) = key_states.get_mut(k) {
                        *ks = true;
                        log::debug!("Key: 0x{:x}", k);
                    }
                }
                ItemKind::Button(id) => unsafe {
                    let hwnd = GetDlgItem(mmd_window, *id as _);
                    if IsWindowVisible(hwnd) == TRUE && IsWindowEnabled(hwnd) == TRUE {
                        PostMessageA(hwnd, BM_CLICK, WPARAM(0), LPARAM(0));
                        log::debug!("Button: 0x{:x}", id);
                    }
                },
                ItemKind::Edit(id) => unsafe {
                    let hwnd = GetDlgItem(mmd_window, *id as _);
                    if IsWindowVisible(hwnd) == TRUE && IsWindowEnabled(hwnd) == TRUE {
                        SetFocus(hwnd);
                        log::debug!("Edit: 0x{:x}", id);
                    }
                },
                ItemKind::Combo(dir, id) => unsafe {
                    #[inline]
                    unsafe fn post_set_cur_sel(hwnd: HWND, id: u32, parent: HWND, index: i32) {
                        PostMessageW(hwnd, CB_SETCURSEL, WPARAM(index as _), LPARAM(0));
                        PostMessageW(
                            parent,
                            WM_COMMAND,
                            WPARAM(((id & 0xffff) | (CBN_SELCHANGE << 16)) as _),
                            LPARAM(hwnd.0),
                        );
                        log::debug!("Combo: 0x{:x}", id);
                    }

                    let hwnd = GetDlgItem(mmd_window, *id as _);
                    if IsWindowVisible(hwnd) == FALSE || IsWindowEnabled(hwnd) == FALSE {
                        return;
                    }
                    let index = SendMessageA(hwnd, CB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
                    let size = SendMessageA(hwnd, CB_GETCOUNT, WPARAM(0), LPARAM(0)).0;
                    match dir {
                        ComboDir::Prev if index >= 1 => post_set_cur_sel(hwnd, *id, mmd_window, index - 1),
                        ComboDir::Next if index < size - 1 => post_set_cur_sel(hwnd, *id, mmd_window, index + 1),
                        _ => {}
                    }
                },
                ItemKind::Menu(index, sub_index) => unsafe {
                    let m = GetSubMenu(GetMenu(mmd_window), *index as _);
                    let state = GetMenuState(m, *sub_index as _, MENU_ITEM_FLAGS::MF_BYPOSITION);
                    if (state & MENU_ITEM_STATE::MFS_DISABLED.0) == 0 {
                        PostMessageA(
                            mmd_window,
                            WM_COMMAND,
                            WPARAM(GetMenuItemID(m, *sub_index as _) as _),
                            LPARAM(0),
                        );
                        log::debug!("Menu: {}, {}", index, sub_index);
                    }
                },
                ItemKind::Fold(hide_id, show_id) => unsafe {
                    let hide = GetDlgItem(mmd_window, *hide_id as _);
                    if IsWindowVisible(hide) == TRUE {
                        PostMessageW(hide, BM_CLICK, WPARAM(0), LPARAM(0));
                        log::debug!("Fold: 0x{:x}", hide_id);
                    } else {
                        let show = GetDlgItem(mmd_window, *show_id as _);
                        PostMessageW(show, BM_CLICK, WPARAM(0), LPARAM(0));
                        log::debug!("Fold: 0x{:x}", show_id);
                    }
                },
                ItemKind::KillFocus => unsafe {
                    SetFocus(mmd_window);
                    log::debug!("KillFocus");
                },
                ItemKind::FoldAll => unsafe {
                    for id in folds {
                        let hwnd = GetDlgItem(mmd_window, *id as _);
                        if IsWindowVisible(hwnd) == TRUE {
                            PostMessageW(hwnd, BM_CLICK, WPARAM(0), LPARAM(0));
                        }
                    }
                    log::debug!("FoldAll");
                },
                ItemKind::UnfoldAll => unsafe {
                    for id in unfolds {
                        let hwnd = GetDlgItem(mmd_window, *id as _);
                        if IsWindowVisible(hwnd) == TRUE {
                            PostMessageW(hwnd, BM_CLICK, WPARAM(0), LPARAM(0));
                        }
                    }
                    log::debug!("UnfoldAll");
                },
            }
        }

        self.input[vk as usize] = 0x80;
        self.input_keys.keyboard_state(&self.input);
        if let Some(item) = self.handler.get(&self.input_keys) {
            handle(item, &mut self.key_states, &self.folds, &self.unfolds, mmd_window);
            return;
        }
        self.input_keys.vk(vk);
        if let Some(item) = self.handler.get(&self.input_keys) {
            handle(item, &mut self.key_states, &self.folds, &self.unfolds, mmd_window);
        }
    }

    pub fn key_up(&mut self, vk: u32) {
        self.input[vk as usize] = 0x00;
        self.input_keys.keyboard_state(&self.input);
        for (keys, kind) in self.handler.iter() {
            if let ItemKind::Key(k) = kind {
                if !keys.is_included(&self.input_keys) {
                    if let Some(ks) = self.key_states.get_mut(k) {
                        *ks = false;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn is_pressed(&self, vk: u32) -> bool {
        *self.key_states.get(&vk).unwrap_or(&false)
    }
}

#[cfg(test)]
#[allow(clippy::eq_op)]
mod tests {
    use super::*;

    #[test]
    fn keys_eq() {
        let a = Keys::from_slice(&[VK_LEFT, VK_CONTROL]);
        let b = Keys::from_slice(&[VK_RIGHT, VK_CONTROL]);
        let c = Keys::from_slice(&[VK_LEFT, VK_CONTROL]);
        assert!(a == a);
        assert!(a != b);
        assert!(a == c);
    }
}
