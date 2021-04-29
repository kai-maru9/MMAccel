use crate::*;

#[derive(Debug)]
struct Item {
    id: String,
    name: String,
    keys: Option<Keys>,
}

#[derive(Debug)]
struct Category {
    name: String,
    items: Vec<Item>,
}

#[derive(Debug)]
struct KeyTable(Vec<Category>);

impl KeyTable {
    fn from_file(
        mmd_map_path: impl AsRef<std::path::Path>,
        order_path: impl AsRef<std::path::Path>,
        key_map_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<Self> {
        const INVALID_DATA: std::io::ErrorKind = std::io::ErrorKind::InvalidData;
        let mmd_map: serde_json::Value = {
            let file = std::fs::File::open(mmd_map_path)?;
            serde_json::from_reader(std::io::BufReader::new(file))?
        };
        let order: serde_json::Value = {
            let file = std::fs::File::open(order_path)?;
            serde_json::from_reader(std::io::BufReader::new(file))?
        };
        let key_map: serde_json::Value = {
            let file = match std::fs::File::open(key_map_path.as_ref()) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    {
                        let file = std::fs::File::create(key_map_path.as_ref())?;
                        serde_json::to_writer_pretty(std::io::BufWriter::new(file), &KeyMap::default())?;
                    }
                    std::fs::File::open(key_map_path.as_ref())?
                },
                Err(e) => return Err(e),
            };
            serde_json::from_reader(std::io::BufReader::new(file))?
        };
        let mmd_map = mmd_map.as_object().ok_or(INVALID_DATA)?;
        let order = order.as_object().ok_or(INVALID_DATA)?;
        let category_order = order.get("categories").and_then(|a| a.as_array()).ok_or(INVALID_DATA)?;
        let item_order = order.get("items").and_then(|a| a.as_object()).ok_or(INVALID_DATA)?;
        let mut table = vec![];
        for category in category_order.iter() {
            let category = category.as_str().ok_or(INVALID_DATA)?.to_string();
            let item = mmd_map.get(&category).and_then(|a| a.as_object()).ok_or(INVALID_DATA)?;
            let item_order = item_order
                .get(&category)
                .and_then(|a| a.as_array())
                .ok_or(INVALID_DATA)?;
            let mut v = vec![];
            for id in item_order.iter() {
                let id = id.as_str().ok_or(INVALID_DATA)?;
                let name = item
                    .get(id)
                    .and_then(|a| a.as_array())
                    .and_then(|a| a[0].as_str())
                    .ok_or(INVALID_DATA)?;
                let keys = key_map
                    .get(id)
                    .and_then(|v| v.as_array())
                    .and_then(|a| {
                        a.iter()
                            .map(|v| v.as_u64().map(|v| v as u32))
                            .collect::<Option<Vec<_>>>()
                    })
                    .map(|a| Keys::from_slice(&a));
                v.push(Item {
                    id: id.to_string(),
                    name: name.to_string(),
                    keys,
                });
            }
            table.push(Category {
                name: category,
                items: v,
            });
        }
        Ok(Self(table))
    }

    fn to_file(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut v = KeyMap::new();
        for elem in self
            .0
            .iter()
            .flat_map(|cat| &cat.items)
            .filter_map(|item| item.keys.as_ref().map(|keys| (&item.id, keys)))
        {
            v.insert(elem.0, elem.1.clone());
        }
        serde_json::to_writer_pretty(std::io::BufWriter::new(file), &v)?;
        Ok(())
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<Category> {
        self.0.iter()
    }

    #[inline]
    fn get(&self, category: usize, item: usize) -> Option<&Keys> {
        (self.0)[category].items[item].keys.as_ref()
    }

    #[inline]
    fn set_keys(&mut self, category: usize, item: usize, keys: Option<Keys>) {
        (self.0)[category].items[item].keys = keys;
    }
}

impl std::ops::Index<usize> for KeyTable {
    type Output = Category;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct Application {
    main_window: wita::Window,
    side_menu: SideMenu,
    shortcut_list: ShortcutList,
    editor: Box<Editor>,
    key_table: KeyTable,
    popup_menu: PopupMenu,
}

impl Application {
    pub fn new() -> anyhow::Result<Box<Self>> {
        let key_table = KeyTable::from_file("mmd_map.json", "order.json", "key_map.json")?;
        let main_window = wita::WindowBuilder::new()
            .title("MMAccel キー設定")
            .style(
                wita::WindowStyle::default()
                    .has_maximize_box(false)
                    .has_minimize_box(false),
            )
            .build()?;
        let mut side_menu = SideMenu::new(&main_window, (10, 10), (150, 460))?;
        key_table.iter().for_each(|cat| side_menu.push(&cat.name));
        side_menu.set_index(0);
        let mut shortcut_list = ShortcutList::new(&main_window, (170, 10), (455, 460))?;
        key_table[0]
            .items
            .iter()
            .for_each(|item| shortcut_list.push(&item.name, item.keys.as_ref()));
        let editor = Editor::new(shortcut_list.handle())?;
        let mut app = Box::new(Self {
            main_window,
            side_menu,
            shortcut_list,
            key_table,
            editor,
            popup_menu: PopupMenu::new(),
        });
        unsafe {
            let hwnd = HWND(app.main_window.raw_handle() as _);
            let app_ptr = app.as_mut() as *mut Self;
            SetWindowSubclass(hwnd, Some(main_window_proc), app_ptr as _, app_ptr as _).ok()?;
        }
        Ok(app)
    }

    fn update_keys(&mut self, category: usize, item: usize, keys: Option<Keys>) {
        if category == self.side_menu.current_index() {
            self.shortcut_list.set_keys(item, keys.as_ref());
        }
        self.key_table.set_keys(category, item, keys);
        self.key_table.to_file("key_map.json").ok();
    }
}

impl wita::EventHandler for Box<Application> {}

pub const WM_KEY_CONFIG_EDIT_APPLY: u32 = WM_APP + 10;
pub const WM_KEY_CONFIG_EDIT_CANCEL: u32 = WM_APP + 11;

unsafe extern "system" fn main_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data_ptr: usize,
) -> LRESULT {
    let app = (data_ptr as *mut Application).as_mut().unwrap();
    match msg {
        WM_NOTIFY => {
            let nmhdr = (lparam.0 as *const NMHDR).as_ref().unwrap();
            if nmhdr.hwndFrom == app.side_menu.handle() && nmhdr.code == LVN_ITEMCHANGED {
                let nlv = (lparam.0 as *const NMLISTVIEW).as_ref().unwrap();
                if nlv.uNewState & LVIS_SELECTED != 0 {
                    app.shortcut_list.clear();
                    for item in app.key_table[app.side_menu.current_index()].items.iter() {
                        app.shortcut_list.push(&item.name, item.keys.as_ref());
                    }
                }
                LRESULT(0)
            } else if nmhdr.hwndFrom == app.shortcut_list.handle() {
                match nmhdr.code {
                    NM_CLICK => {
                        if app.editor.is_visible() {
                            if let Some(ret) = app.editor.end() {
                                app.update_keys(ret.category, ret.item, Some(ret.keys));
                            }
                        }
                    }
                    NM_DBLCLK => {
                        let nia = (lparam.0 as *const NMITEMACTIVATE).as_ref().unwrap();
                        if let Some(rc) = app.shortcut_list.keys_rect(nia.iItem as _) {
                            let category = app.side_menu.current_index();
                            let item = nia.iItem as _;
                            app.editor.begin(&rc, category, item, app.key_table.get(category, item));
                        }
                    }
                    NM_RCLICK => {
                        if app.editor.is_visible() {
                            app.editor.end();
                        } else {
                            let nia = (lparam.0 as *const NMITEMACTIVATE).as_ref().unwrap();
                            let mut pt = POINT {
                                x: nia.ptAction.x,
                                y: nia.ptAction.y,
                            };
                            ClientToScreen(app.shortcut_list.handle(), &mut pt);
                            app.popup_menu.track(
                                &app.main_window,
                                app.side_menu.current_index(),
                                nia.iItem as _,
                                wita::ScreenPosition::new(pt.x, pt.y),
                            );
                        }
                    }
                    _ => {}
                }
                LRESULT(0)
            } else {
                DefSubclassProc(hwnd, msg, wparam, lparam)
            }
        }
        WM_ERASEBKGND => {
            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc);
            FillRect(
                HDC(wparam.0 as _),
                &rc,
                HBRUSH(GetStockObject(GET_STOCK_OBJECT_FLAGS(SYS_COLOR_INDEX::COLOR_BTNFACE.0 + 1)).0 as _),
            );
            LRESULT(1)
        }
        WM_COMMAND => {
            if (wparam.0 & 0xffff) as u32 == IDM_MENU_DETACH {
                app.update_keys(app.popup_menu.category(), app.popup_menu.item(), None);
            }
            LRESULT(0)
        }
        WM_KEY_CONFIG_EDIT_APPLY => {
            if app.editor.is_visible() {
                if let Some(ret) = app.editor.end() {
                    app.update_keys(ret.category, ret.item, Some(ret.keys));
                }
            }
            LRESULT(0)
        }
        WM_KEY_CONFIG_EDIT_CANCEL => {
            app.editor.end();
            LRESULT(0)
        }
        _ => DefSubclassProc(hwnd, msg, wparam, lparam),
    }
}
