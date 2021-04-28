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
            let file = std::fs::File::open(key_map_path)?;
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
                let keys = key_map.get(id).and_then(|v| v.as_array()).and_then(|a| {
                    a.iter()
                        .map(|v| v.as_u64().map(|v| v as u32))
                        .collect::<Option<Vec<_>>>()
                }).map(|a| Keys::from_slice(&a));
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

    fn iter(&self) -> std::slice::Iter<Category> {
        self.0.iter()
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
    editor: Editor,
    key_table: KeyTable,
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
        });
        unsafe {
            let hwnd = HWND(app.main_window.raw_handle() as _);
            let app_ptr = app.as_mut() as *mut Self;
            SetWindowSubclass(hwnd, Some(main_window_proc), app_ptr as _, app_ptr as _).ok()?;
        }
        Ok(app)
    }
}

impl wita::EventHandler for Box<Application> {}

extern "system" fn main_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data_ptr: usize,
) -> LRESULT {
    unsafe {
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
                } else if nmhdr.hwndFrom == app.shortcut_list.handle() {
                    match nmhdr.code {
                        NM_DBLCLK => {
                            let nia = (lparam.0 as *const NMITEMACTIVATE).as_ref().unwrap();
                            if let Some(rc) = app.shortcut_list.keys_rect(nia.iItem as _) {
                                app.editor.show(&rc);
                            }
                        }
                        LVN_ITEMCHANGED => {
                            let nlv = (lparam.0 as *const NMLISTVIEW).as_ref().unwrap();
                            let state = (nlv.uChanged & LVIF_STATE) != 0
                                && (nlv.uOldState & LVIS_SELECTED) != 0
                                && (nlv.uNewState & LVIS_SELECTED) == 0;
                            if state {
                                app.editor.hide();
                            }
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
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
            _ => DefSubclassProc(hwnd, msg, wparam, lparam),
        }
    }
}
