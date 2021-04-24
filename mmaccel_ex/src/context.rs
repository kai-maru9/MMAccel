use crate::*;
use bindings::wrapper::*;

fn get_mmd_window() -> HWND {
   enum_windows().into_iter().find(|window| {
       get_class_name(*window) == "Polygon Movie Maker"
   }).unwrap()
}

pub struct Context {
    _get_message_handle: HookHandle,
    mmd_window: HWND,
    menu: Menu,
}

impl Context {
    pub fn new() -> Self {
        let mmd_window = get_mmd_window();
        let menu = Menu::new(mmd_window);
        menu.model_palette(true);
        Self {
            _get_message_handle: HookHandle::new(
                SetWindowsHookEx_idHook::WH_GETMESSAGE,
                Some(hook_get_message),
                get_current_thread_id(),
            ),
            mmd_window,
            menu,
        }
    }

    pub fn get_message(&mut self, msg: &mut MSG) {
        match msg.message {
            WM_COMMAND => {
                match self.menu.on_command(msg.wParam) {
                    Some(MenuItem::LaunchConfig) => println!("launch"),
                    Some(MenuItem::ModelPallete(b)) => println!("model palette({})", b),
                    Some(MenuItem::Version) => println!("version"),
                    _ => {}
                }
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
            }
            WM_KEYUP | WM_SYSKEYUP => {
            }
            _ => {}
        }
    }

    pub fn get_key_state(&self, vk: u32) -> Option<i16> {
        if vk >= 0x07 {
            Some(0x0000)
        } else {
            None
        }
    }
}
