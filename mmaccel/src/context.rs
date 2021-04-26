use crate::*;
use bindings::wrapper::*;
use handler::Handler;
use key_map::KeyMap;
use mmd_map::MmdMap;

struct MmdWindow {
    window: HWND,
    menu: Menu,
}

impl MmdWindow {
    #[inline]
    fn new(window: HWND) -> Self {
        Self {
            window,
            menu: Menu::new(window),
        }
    }
}

fn version_info(hwnd: HWND) {
    let text = format!("MMAccel v{}\nby LNSEAB", env!("CARGO_PKG_VERSION"));
    message_box(Some(hwnd), text, "", MESSAGEBOX_STYLE::MB_OK);
}

pub struct Context {
    _call_window_proc_ret: HookHandle,
    _get_message_handle: HookHandle,
    mmd_window: Option<MmdWindow>,
    handler: Handler,
}

impl Context {
    #[inline]
    pub fn new() -> std::io::Result<Self> {
        const MMD_MAP_PATH: &str = "MMAccel/mmd_map.json";
        const KEY_MAP_PATH: &str = "MMAccel/key_map.json";
        let mmd_map = MmdMap::from_file(MMD_MAP_PATH)?;
        let key_map = KeyMap::from_file(KEY_MAP_PATH).unwrap_or_else(|_| {
            let m = KeyMap::default();
            if let Ok(file) = std::fs::File::create(KEY_MAP_PATH) {
                serde_json::to_writer_pretty(std::io::BufWriter::new(file), &m).ok();
                log::debug!("written key_map.json");
            }
            m
        });
        let handler = Handler::new(mmd_map, key_map);
        Ok(Self {
            _call_window_proc_ret: HookHandle::new(
                SetWindowsHookEx_idHook::WH_CALLWNDPROCRET,
                Some(hook_call_window_proc_ret),
                get_current_thread_id(),
            ),
            _get_message_handle: HookHandle::new(
                SetWindowsHookEx_idHook::WH_GETMESSAGE,
                Some(hook_get_message),
                get_current_thread_id(),
            ),
            mmd_window: None,
            handler,
        })
    }

    pub fn call_window_proc_ret(&mut self, data: &CWPRETSTRUCT) {
        match data.message {
            WM_CREATE if get_class_name(data.hwnd) == "Polygon Movie Maker" => {
                log::debug!("Created MainWindow");
                self.mmd_window = Some(MmdWindow::new(data.hwnd));
            }
            WM_DESTROY if self.mmd_window.as_ref().map_or(false, |mw| mw.window == data.hwnd) => {
                log::debug!("Destroyed MainWindow");
            }
            _ => {}
        }
    }

    pub fn get_message(&mut self, data: &mut MSG) {
        match data.message {
            WM_COMMAND => {
                if let Some(mmd_window) = self.mmd_window.as_ref() {
                    match mmd_window.menu.recv_command(data.wParam) {
                        Some(MenuItem::LaunchConfig) => println!("launch"),
                        Some(MenuItem::Version) => version_info(mmd_window.window),
                        _ => {}
                    }
                }
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                self.handler
                    .key_down(data.wParam.0 as u32, self.mmd_window.as_ref().unwrap().window);
            }
            WM_KEYUP | WM_SYSKEYUP => {
                self.handler.key_up(data.wParam.0 as u32);
            }
            _ => {}
        }
    }

    pub fn get_key_state(&self, vk: u32) -> Option<u16> {
        if vk >= 0x07 {
            if self.handler.is_pressed(vk) {
                Some(0xff80)
            } else {
                Some(0x0000)
            }
        } else {
            None
        }
    }
}
