use crate::*;
use std::cell::RefCell;
use std::os::windows::io::AsRawHandle;
use std::sync::{atomic, atomic::AtomicBool, Arc};
use std::thread::JoinHandle;

pub struct FileMonitor {
    th: RefCell<Option<JoinHandle<()>>>,
    exit_flag: Arc<AtomicBool>,
}

impl FileMonitor {
    pub fn new() -> Self {
        Self {
            th: RefCell::new(None),
            exit_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, dir_path: impl AsRef<std::path::Path>, mut f: impl FnMut(&std::path::Path) + Send + 'static) {
        let dir_path = dir_path.as_ref().to_string_lossy().to_string();
        let exit_flag = self.exit_flag.clone();
        let mut th = self.th.borrow_mut();
        *th = Some(std::thread::spawn(move || unsafe {
            let dir_path = to_wchar(&dir_path);
            let dir = CreateFileW(
                PWSTR(dir_path.as_ptr() as _),
                FILE_ACCESS_FLAGS::FILE_LIST_DIRECTORY,
                FILE_SHARE_MODE::FILE_SHARE_READ,
                std::ptr::null_mut(),
                FILE_CREATION_DISPOSITION::OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES::FILE_FLAG_BACKUP_SEMANTICS,
                HANDLE::NULL,
            );
            let mut buffer = [0u8; 2048];
            loop {
                let mut len = 0;
                let ret = ReadDirectoryChangesW(
                    dir,
                    buffer.as_mut_ptr() as _,
                    buffer.len() as _,
                    FALSE,
                    FILE_NOTIFY_CHANGE::FILE_NOTIFY_CHANGE_LAST_WRITE,
                    &mut len,
                    std::ptr::null_mut(),
                    None,
                );
                if exit_flag.load(atomic::Ordering::SeqCst) {
                    break;
                }
                if ret == FALSE {
                    break;
                }
                let mut data_ptr = buffer.as_ptr() as *const FILE_NOTIFY_INFORMATION;
                loop {
                    let data = data_ptr.as_ref().unwrap();
                    let file_name = std::slice::from_raw_parts(
                        data.FileName.as_ptr() as *const u16,
                        data.FileNameLength as usize / std::mem::size_of::<u16>(),
                    );
                    let file_name = String::from_utf16_lossy(file_name);
                    f(std::path::Path::new(&file_name));
                    if data.NextEntryOffset == 0 {
                        break;
                    }
                    data_ptr = (data_ptr as *const u8).offset(data.NextEntryOffset as _) as *const _;
                }
            }
            CloseHandle(dir);
        }));
    }

    pub fn stop(&mut self) -> Option<JoinHandle<()>> {
        if let Some(th) = self.th.take() {
            self.exit_flag.store(true, atomic::Ordering::SeqCst);
            unsafe {
                CancelSynchronousIo(HANDLE(th.as_raw_handle() as _));
            }
            Some(th)
        } else {
            None
        }
    }
}
