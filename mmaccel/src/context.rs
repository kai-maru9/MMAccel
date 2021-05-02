use crate::*;
use bindings::wrapper::*;
use handler::Handler;
use key_map::KeyMap;
use mmd_map::MmdMap;
use std::sync::{atomic, atomic::AtomicBool, Arc};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuItem {
    LaunchConfig,
    RaiseTimerResolution(bool),
    Version,
}

impl MenuCommand for MenuItem {
    fn from_command(v: std::mem::Discriminant<Self>, item_type: MenuItemType) -> Self {
        match v {
            _ if v == std::mem::discriminant(&Self::LaunchConfig) => Self::LaunchConfig,
            _ if v == std::mem::discriminant(&Self::RaiseTimerResolution(false)) => {
                Self::RaiseTimerResolution(item_type.as_with_check().unwrap())
            }
            _ if v == std::mem::discriminant(&Self::Version) => Self::Version,
            _ => unimplemented!(),
        }
    }
}

struct MmdWindow {
    window: HWND,
    menu: Menu<MenuItem>,
}

impl MmdWindow {
    #[inline]
    fn new(window: HWND, settings: &Settings) -> Self {
        Self {
            window,
            menu: MenuBuilder::new(window, "MMAccel")
                .item(&MenuItem::LaunchConfig, "キー設定")
                .separator()
                .with_check(
                    &MenuItem::RaiseTimerResolution(true),
                    "タイマーの精度を上げる",
                    settings.raise_timer_resolution,
                )
                .separator()
                .item(&MenuItem::Version, "バージョン情報")
                .build(),
        }
    }
}

struct TimePeriod(u32);

impl TimePeriod {
    #[inline]
    fn new(n: u32) -> Self {
        unsafe {
            timeBeginPeriod(n);
            Self(n)
        }
    }
}

impl Drop for TimePeriod {
    fn drop(&mut self) {
        unsafe {
            timeEndPeriod(self.0);
        }
    }
}

fn version_info(hwnd: HWND) {
    let text = format!("MMAccel v{}\nby LNSEAB", env!("CARGO_PKG_VERSION"));
    message_box(Some(hwnd), text, "", MESSAGEBOX_STYLE::MB_OK);
}

#[derive(Debug, serde::Serialize)]
struct Settings {
    raise_timer_resolution: bool,
}

impl Settings {
    const PATH: &'static str = "MMAccel/settings.json";

    fn from_file(module_path: &std::path::Path) -> Option<Self> {
        match std::fs::File::open(module_path.join(Self::PATH)) {
            Ok(file) => {
                let data: serde_json::Value = match serde_json::from_reader(std::io::BufReader::new(file)) {
                    Ok(data) => data,
                    Err(_) => return None,
                };
                let obj = data.as_object()?;
                let default = Self::default();
                Some(Self {
                    raise_timer_resolution: obj
                        .get("raise_timer_resolution")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(default.raise_timer_resolution),
                })
            }
            Err(_) => None,
        }
    }

    fn to_file(&self) {
        if let Ok(file) = std::fs::File::create(Self::PATH) {
            serde_json::to_writer_pretty(std::io::BufWriter::new(file), self).ok();
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            raise_timer_resolution: true,
        }
    }
}

const MMD_MAP_PATH: &str = "MMAccel/mmd_map.json";
const KEY_MAP_PATH: &str = "MMAccel/key_map.json";

pub struct Context {
    module_path: std::path::PathBuf,
    settings: Settings,
    _call_window_proc_ret: HookHandle,
    _get_message_handle: HookHandle,
    mmd_window: Option<MmdWindow>,
    handler: Handler,
    file_monitor: FileMonitor,
    latest_key_map: Arc<AtomicBool>,
    key_config: Option<HWND>,
    time_period: Option<TimePeriod>,
}

impl Context {
    #[inline]
    pub fn new(module_path: std::path::PathBuf) -> std::io::Result<Self> {
        let settings = Settings::from_file(&module_path).unwrap_or_default();
        log::debug!("{:?}", settings);
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
        let file_monitor = FileMonitor::new();
        let time_period = settings.raise_timer_resolution.then(|| TimePeriod::new(1));
        Ok(Self {
            module_path,
            settings,
            _call_window_proc_ret: HookHandle::new(
                WINDOWS_HOOK_ID::WH_CALLWNDPROCRET,
                Some(hook_call_window_proc_ret),
                get_current_thread_id(),
            ),
            _get_message_handle: HookHandle::new(
                WINDOWS_HOOK_ID::WH_GETMESSAGE,
                Some(hook_get_message),
                get_current_thread_id(),
            ),
            mmd_window: None,
            handler,
            file_monitor,
            latest_key_map: Arc::new(AtomicBool::new(true)),
            key_config: None,
            time_period,
        })
    }

    pub fn call_window_proc_ret(&mut self, data: &CWPRETSTRUCT) {
        match data.message {
            WM_CREATE if get_class_name(data.hwnd) == "Polygon Movie Maker" => {
                log::debug!("created MainWindow");
                self.mmd_window = Some(MmdWindow::new(data.hwnd, &self.settings));
                let latest_key_map = self.latest_key_map.clone();
                let mmd_window = self.mmd_window.as_ref().unwrap().window;
                self.file_monitor.start("MMAccel", move |path| unsafe {
                    if path.file_name() == Some(std::ffi::OsStr::new("key_map.json")) {
                        latest_key_map.store(false, atomic::Ordering::SeqCst);
                        PostMessageW(mmd_window, WM_APP, WPARAM(0), LPARAM(0));
                        log::debug!("update key_map.json");
                    }
                });
            }
            WM_DESTROY if self.mmd_window.as_ref().map_or(false, |mw| mw.window == data.hwnd) => {
                if let Some(kc) = self.key_config {
                    unsafe {
                        if IsWindow(kc) == TRUE {
                            PostMessageW(self.key_config, WM_CLOSE, WPARAM(0), LPARAM(0));
                        }
                    }
                }
                if let Some(jh) = self.file_monitor.stop() {
                    jh.join().ok();
                    log::debug!("stop FileMonitor");
                }
                log::debug!("destroyed MainWindow");
            }
            _ => {}
        }
    }

    pub fn get_message(&mut self, data: &mut MSG) {
        match data.message {
            WM_COMMAND => {
                if let Some(mmd_window) = self.mmd_window.as_ref() {
                    match mmd_window.menu.recv_command(data.wParam) {
                        Some(MenuItem::LaunchConfig) => {
                            let path = self.module_path.join("MMAccel/key_config.exe");
                            let key_config_process = std::process::Command::new(&path)
                                .current_dir(self.module_path.join("MMAccel"))
                                .arg("--mmd")
                                .stdout(std::process::Stdio::piped())
                                .spawn();
                            match key_config_process {
                                Ok(process) => {
                                    use std::os::windows::io::AsRawHandle;
                                    let mut p = 0u64;
                                    let mut byte = 0;
                                    unsafe {
                                        let handle = HANDLE(process.stdout.as_ref().unwrap().as_raw_handle() as _);
                                        if ReadFile(
                                            handle,
                                            &mut p as *mut _ as _,
                                            std::mem::size_of::<u64>() as _,
                                            &mut byte,
                                            std::ptr::null_mut(),
                                        ) != FALSE
                                        {
                                            self.key_config = Some(HWND(p as _));
                                        }
                                    }
                                }
                                Err(e) => log::error!("LaunchCconfig: {:?}", e),
                            }
                        }
                        Some(MenuItem::RaiseTimerResolution(b)) => {
                            self.time_period = if b { Some(TimePeriod::new(1)) } else { None };
                        }
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
            WM_APP => {
                if !self.latest_key_map.swap(true, atomic::Ordering::SeqCst) {
                    let mmd_map = MmdMap::from_file(MMD_MAP_PATH);
                    if mmd_map.is_err() {
                        return;
                    }
                    let key_map = KeyMap::from_file(KEY_MAP_PATH).unwrap_or_else(|_| {
                        let m = KeyMap::default();
                        if let Ok(file) = std::fs::File::create(KEY_MAP_PATH) {
                            serde_json::to_writer_pretty(std::io::BufWriter::new(file), &m).ok();
                            log::debug!("written key_map.json");
                        }
                        m
                    });
                    self.handler = Handler::new(mmd_map.unwrap(), key_map);
                }
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

impl Drop for Context {
    fn drop(&mut self) {
        self.settings.raise_timer_resolution = self.time_period.is_some();
        self.settings.to_file();
        log::debug!("drop Context");
    }
}
