use crate::*;
use bindings::wrapper::*;
use handler::Handler;
use mmd_map::MmdMap;
use key_map::KeyMap;

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
    pub fn new() -> Self {
        let mmd_map = MmdMap::new("MMAccel/mmd_map.json").unwrap();
        let handler = Handler::new(&mmd_map, &KeyMap::default());
        Self {
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
        }
    }

    pub fn call_window_proc_ret(&mut self, data: &CWPRETSTRUCT) {
        match data.message {
            WM_CREATE if get_class_name(data.hwnd) == "Polygon Movie Maker" => {
                log::debug!("Created MainWindow");
                self.mmd_window = Some(MmdWindow::new(data.hwnd));
            }
            WM_DESTROY
                if self
                    .mmd_window
                    .as_ref()
                    .map_or(false, |mw| mw.window == data.hwnd) =>
            {
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
                        Some(MenuItem::ModelPallete(b)) => println!("model palette({})", b),
                        Some(MenuItem::Version) => version_info(mmd_window.window),
                        _ => {}
                    }
                }
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                self.handler.key_down(
                    data.wParam.0 as u32,
                    self.mmd_window.as_ref().unwrap().window,
                );
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
