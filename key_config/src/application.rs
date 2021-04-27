use crate::*;

#[derive(Debug)]
struct Item {
    id: String,
    name: String,
    keys: Keys,
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
                v.push(Item{ 
                    id: id.to_string(),
                    name: name.to_string(), 
                    keys: Keys::new()
                });
                println!("{}", id);
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
    key_table: KeyTable,
}

impl Application {
    pub fn new() -> anyhow::Result<Box<Self>> {
        let key_table = KeyTable::from_file("mmd_map.json", "order.json")?;
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
        key_table[0].items.iter().for_each(|item| shortcut_list.push(&item.name));
        let mut app = Box::new(Self {
            main_window,
            side_menu,
            shortcut_list,
            key_table,
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
            WM_COMMAND => {
                if app.side_menu.handle() == HWND(lparam.0) && ((wparam.0 >> 16) & 0xffff) as u32 == LBN_SELCHANGE {
                    app.shortcut_list.clear();
                    for item in app.key_table[app.side_menu.current_index()].items.iter() {
                        app.shortcut_list.push(&item.name);
                    }
                    app.main_window.redraw();
                }
                LRESULT(0)
            }
            WM_ERASEBKGND => {
                let mut rc = RECT::default();
                GetClientRect(hwnd, &mut rc);
                FillRect(
                    HDC(wparam.0 as _),
                    &rc,
                    HBRUSH(GetStockObject(GetStockObject_iFlags(GetSysColor_nIndexFlags::COLOR_BTNFACE.0 + 1)).0),
                );
                LRESULT(1)
            }
            _ => DefSubclassProc(hwnd, msg, wparam, lparam),
        }
    }
}
