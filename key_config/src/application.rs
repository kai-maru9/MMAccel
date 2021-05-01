use crate::*;

#[inline]
fn from_file<T>(path: impl AsRef<std::path::Path>) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let file = std::fs::File::open(&path).map_err(|e| Error::file(e, &path))?;
    serde_json::from_reader(std::io::BufReader::new(file)).map_err(|e| Error::json_file(e, path))
}

#[inline]
fn to_file<T>(path: impl AsRef<std::path::Path>, value: &T) -> Result<(), Error>
where
    T: serde::Serialize,
{
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(std::io::BufWriter::new(file), value).map_err(|e| Error::json_file(e, path))
}

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
        key_map_path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Error> {
        let mmd_map: serde_json::Value = from_file(mmd_map_path)?;
        let order: serde_json::Value = from_file(order_path)?;
        let key_map: serde_json::Value = match from_file(&key_map_path) {
            Ok(v) => v,
            Err(Error::FileNotFound(_)) => {
                let key_map = KeyMap::default();
                to_file(key_map_path, &key_map)?;
                serde_json::to_value(&key_map).unwrap()
            }
            Err(e) => return Err(e),
        };
        let mmd_map = mmd_map.as_object().ok_or(Error::InvalidData)?;
        let order = order.as_object().ok_or(Error::InvalidData)?;
        let category_order = order
            .get("categories")
            .and_then(|a| a.as_array())
            .ok_or(Error::InvalidData)?;
        let item_order = order
            .get("items")
            .and_then(|a| a.as_object())
            .ok_or(Error::InvalidData)?;
        let mut table = vec![];
        for category in category_order.iter() {
            let category = category.as_str().ok_or(Error::InvalidData)?.to_string();
            let item = mmd_map
                .get(&category)
                .and_then(|a| a.as_object())
                .ok_or(Error::InvalidData)?;
            let item_order = item_order
                .get(&category)
                .and_then(|a| a.as_array())
                .ok_or(Error::InvalidData)?;
            let mut v = vec![];
            for id in item_order.iter() {
                let id = id.as_str().ok_or(Error::InvalidData)?;
                let name = item
                    .get(id)
                    .and_then(|a| a.as_array())
                    .and_then(|a| a[0].as_str())
                    .ok_or(Error::InvalidData)?;
                let keys = key_map
                    .get(id)
                    .and_then(|v| v.as_array())
                    .and_then(|a| {
                        a.iter()
                            .map(|v| v.as_u64().map(|v| v as u32))
                            .collect::<Option<Vec<_>>>()
                    })
                    .map(|a| Keys::from_slice(&a))
                    .unwrap_or_default();
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

    fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let mut v = KeyMap::new();
        for elem in self.0.iter().flat_map(|cat| &cat.items).filter_map(|item| {
            if item.keys.is_empty() {
                None
            } else {
                Some((&item.id, &item.keys))
            }
        }) {
            v.insert(elem.0, elem.1.clone());
        }
        to_file(path, &v)
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<Category> {
        self.0.iter()
    }

    #[inline]
    fn get(&self, category: usize, item: usize) -> &Keys {
        &(self.0)[category].items[item].keys
    }

    #[inline]
    fn set_keys(&mut self, category: usize, item: usize, keys: Keys) {
        (self.0)[category].items[item].keys = keys;
    }
}

impl std::ops::Index<usize> for KeyTable {
    type Output = Category;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

const SETTINGS_FILE_NAME: &str = "key_config_settrings.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Settings {
    window_position: wita::ScreenPosition,
    window_size: wita::LogicalSize<u32>,
}

impl Settings {
    fn from_file() -> Result<Self, Error> {
        let settings = match from_file(SETTINGS_FILE_NAME) {
            Ok(v) => v,
            Err(Error::FileNotFound(_)) => Self::default(),
            Err(e) => return Err(e),
        };
        Ok(settings)
    }

    fn to_file(&self) -> Result<(), Error> {
        to_file(SETTINGS_FILE_NAME, self)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_position: (0, 0).into(),
            window_size: (710, 526).into(),
        }
    }
}

const MARGIN: i32 = 10;
const SIDE_MENU_WIDTH: i32 = 150;
const SHORTCUT_MENU_NAME_COLUMN_WIDTH: i32 = 187;
const SHORTCUT_MENU_KEYS_COLUMN_WIDTH: i32 = 133;

struct Rect {
    position: wita::LogicalPosition<i32>,
    size: wita::LogicalSize<i32>,
}

struct Layout {
    side_menu: Rect,
    shortcut_list: Rect,
}

fn calc_layout(window_size: wita::LogicalSize<u32>) -> Layout {
    let height = window_size.height as i32 - MARGIN * 2;
    let side_menu = Rect {
        position: (MARGIN, MARGIN).into(),
        size: (SIDE_MENU_WIDTH, height).into(),
    };
    let width = window_size.width as i32 - SIDE_MENU_WIDTH - MARGIN * 3;
    let shortcut_list = Rect {
        position: ((SIDE_MENU_WIDTH + MARGIN * 2) as _, MARGIN as _).into(),
        size: (width, height).into(),
    };
    Layout {
        side_menu,
        shortcut_list,
    }
}

pub struct Application {
    settings: Settings,
    main_window: wita::Window,
    side_menu: SideMenu,
    shortcut_list: ShortcutList,
    editor: Box<Editor>,
    key_table: KeyTable,
    popup_menu: PopupMenu,
}

impl Application {
    pub fn new() -> Result<Box<Self>, Error> {
        let settings = Settings::from_file()?;
        let main_window = wita::WindowBuilder::new()
            .title("MMAccel キー設定")
            .position(settings.window_position)
            .inner_size(settings.window_size)
            .style(
                wita::WindowStyle::default()
                    .has_maximize_box(false)
                    .has_minimize_box(false),
            )
            .build()?;
        if std::env::args().any(|arg| arg == "--mmd") {
            use std::os::windows::io::AsRawHandle;
            let stdout = std::io::stdout();
            let handle = HANDLE(stdout.lock().as_raw_handle() as _);
            let p = main_window.raw_handle() as u64;
            let mut byte = 0;
            unsafe {
                WriteFile(
                    handle,
                    &p as *const _ as _,
                    std::mem::size_of::<u64>() as _,
                    &mut byte,
                    std::ptr::null_mut(),
                );
            }
        }
        let key_table = KeyTable::from_file("mmd_map.json", "order.json", "key_map.json")?;
        let layout = calc_layout(settings.window_size);
        let mut side_menu = SideMenu::new(&main_window, layout.side_menu.position, layout.side_menu.size)?;
        key_table.iter().for_each(|cat| side_menu.push(&cat.name));
        side_menu.set_index(0);
        let mut shortcut_list = ShortcutList::new(
            &main_window,
            layout.shortcut_list.position,
            layout.shortcut_list.size,
            [SHORTCUT_MENU_NAME_COLUMN_WIDTH, SHORTCUT_MENU_KEYS_COLUMN_WIDTH],
        )?;
        key_table[0]
            .items
            .iter()
            .for_each(|item| shortcut_list.push(&item.name, &item.keys));
        let editor = Editor::new(shortcut_list.handle())?;
        let mut app = Box::new(Self {
            settings,
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
            SetWindowSubclass(hwnd, Some(main_window_proc), app_ptr as _, app_ptr as _);
        }
        Ok(app)
    }

    fn update_keys(&mut self, category: usize, item: usize, keys: Keys) {
        if category == self.side_menu.current_index() {
            self.shortcut_list.set_keys(item, &keys);
        }
        self.key_table.set_keys(category, item, keys);
        self.key_table.to_file("key_map.json").ok();
        self.update_shortcut_list();
    }

    fn update_shortcut_list(&mut self) {
        let category = self.side_menu.current_index();
        for (index, item) in self.key_table[category].items.iter().enumerate() {
            if item.keys.is_empty() {
                self.shortcut_list.set_dup(index, None);
                continue;
            }
            let dup = self
                .key_table
                .iter()
                .flat_map(|cat| &cat.items)
                .filter(|i| i.id != item.id && !i.keys.is_empty() && i.keys == item.keys)
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>();
            if dup.is_empty() {
                self.shortcut_list.set_dup(index, None);
            } else {
                self.shortcut_list.set_dup(index, Some(&dup.join(", ")));
            }
        }
    }
}

impl wita::EventHandler for Box<Application> {
    fn resizing(&mut self, window: &wita::Window, size: wita::PhysicalSize<u32>) {
        let dpi = window.dpi();
        let mut window_size = size.to_logical(dpi);
        const WIDTH: u32 = (SHORTCUT_MENU_NAME_COLUMN_WIDTH + SHORTCUT_MENU_KEYS_COLUMN_WIDTH) as _;
        if window_size.width < WIDTH {
            window_size.width = WIDTH;
            window.set_inner_size(window_size);
        }
        if window_size.height < 240 {
            window_size.height = 240;
            window.set_inner_size(window_size);
        }
        let layout = calc_layout(window_size);
        self.side_menu.resize(layout.side_menu.position, layout.side_menu.size);
        self.shortcut_list.resize(
            layout.shortcut_list.position,
            layout.shortcut_list.size,
            [SHORTCUT_MENU_NAME_COLUMN_WIDTH, SHORTCUT_MENU_KEYS_COLUMN_WIDTH],
        );
    }

    fn dpi_changed(&mut self, _: &wita::Window) {
        self.editor.resize();
    }

    fn closed(&mut self, _: &wita::Window) {
        self.settings.window_position = self.main_window.position();
        self.settings.window_size = self.main_window.inner_size().to_logical(self.main_window.dpi());
        if let Err(e) = self.settings.to_file() {
            log::error!("{}", e);
        }
    }
}

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
            if nmhdr.hwndFrom == app.side_menu.handle() {
                match nmhdr.code {
                    LVN_ITEMCHANGED => {
                        let nlv = (lparam.0 as *const NMLISTVIEW).as_ref().unwrap();
                        if app.editor.is_visible() {
                            if let Some(ret) = app.editor.end() {
                                app.update_keys(ret.category, ret.item, ret.keys);
                            }
                        }
                        if nlv.uNewState & LVIS_SELECTED != 0 {
                            app.shortcut_list.clear();
                            for item in app.key_table[app.side_menu.current_index()].items.iter() {
                                app.shortcut_list.push(&item.name, &item.keys);
                            }
                            app.update_shortcut_list();
                        }
                    }
                    NM_SETFOCUS => {
                        if app.editor.is_visible() {
                            if let Some(ret) = app.editor.end() {
                                app.update_keys(ret.category, ret.item, ret.keys);
                            }
                        }
                    }
                    _ => {}
                }
                LRESULT(0)
            } else if nmhdr.hwndFrom == app.shortcut_list.handle() {
                match nmhdr.code {
                    NM_CUSTOMDRAW => {
                        let subitem_stage = (NMCUSTOMDRAW_DRAW_STAGE::CDDS_ITEMPREPAINT
                            | NMCUSTOMDRAW_DRAW_STAGE::CDDS_SUBITEM)
                            .0 as u32;
                        let mut ncd = (lparam.0 as *mut NMLVCUSTOMDRAW).as_mut().unwrap();
                        match ncd.nmcd.dwDrawStage {
                            NMCUSTOMDRAW_DRAW_STAGE::CDDS_PREPAINT => {
                                return LRESULT(CDRF_NOTIFYITEMDRAW as _);
                            }
                            NMCUSTOMDRAW_DRAW_STAGE::CDDS_ITEMPREPAINT => {
                                return LRESULT(CDRF_NOTIFYSUBITEMDRAW as _);
                            }
                            stage if (stage.0 & subitem_stage) != 0 => {
                                if ncd.iSubItem == 2 {
                                    ncd.clrText = 0x0000ff;
                                    return LRESULT(CDRF_NEWFONT as _);
                                }
                                return LRESULT(CDRF_DODEFAULT as _);
                            }
                            _ => {}
                        }
                    }
                    NM_CLICK => {
                        if app.editor.is_visible() {
                            if let Some(ret) = app.editor.end() {
                                app.update_keys(ret.category, ret.item, ret.keys);
                            }
                        }
                    }
                    NM_DBLCLK => {
                        let nia = (lparam.0 as *const NMITEMACTIVATE).as_ref().unwrap();
                        if nia.iItem != -1 {
                            if let Some(rc) = app.shortcut_list.keys_rect(nia.iItem as _) {
                                let category = app.side_menu.current_index();
                                let item = nia.iItem as _;
                                app.editor.begin(&rc, category, item, app.key_table.get(category, item));
                            }
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
                    NM_SETFOCUS => {
                        let lbutton = (GetKeyState(VK_LBUTTON as _) & 0x80) != 0;
                        if app.editor.is_visible() && lbutton {
                            if let Some(ret) = app.editor.end() {
                                app.update_keys(ret.category, ret.item, ret.keys);
                            }
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
                app.update_keys(app.popup_menu.category(), app.popup_menu.item(), Keys::new());
            }
            LRESULT(0)
        }
        WM_KEY_CONFIG_EDIT_APPLY => {
            if app.editor.is_visible() {
                if let Some(ret) = app.editor.end() {
                    app.update_keys(ret.category, ret.item, ret.keys);
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
