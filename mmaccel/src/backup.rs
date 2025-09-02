// mmaccel/src/backup.rs

use crate::config::Config;
use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use windows::core::{PCWSTR, HSTRING};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowTextW};

// MMDのウィンドウクラス名
const MMD_CLASS_NAME: &str = "MMD_CLASS";

// 定期バックアップを開始する関数
pub fn start_backup_thread(config: Config) {
    thread::spawn(move || {
        loop {
            // 指定された時間だけスリープ
            thread::sleep(Duration::from_secs(config.backup_interval_minutes * 60));

            // 現在開いているPMMファイルのパスを取得
            if let Some(pmm_path) = get_current_pmm_path() {
                // バックアップを作成
                create_backup(&pmm_path, &config);
            }
        }
    });
}

// ウィンドウタイトルから現在開いているPMMファイルのパスを取得する
fn get_current_pmm_path() -> Option<PathBuf> {
    unsafe {
        let class_name = HSTRING::from(MMD_CLASS_NAME);
        let hwnd = FindWindowW(&class_name, None);

        if hwnd.0 == 0 {
            return None;
        }

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return None;
        }

        let title_str = String::from_utf16_lossy(&title[..len as usize]);
        // タイトル形式 "MikuMikuDance - [C:\path\to\your\file.pmm]" からパスを抽出
        if let Some(start) = title_str.find('[') {
            if let Some(end) = title_str.rfind(']') {
                let path_str = &title_str[start + 1..end];
                let path = PathBuf::from(path_str);
                if path.exists() && path.extension().map_or(false, |ext| ext == "pmm") {
                    return Some(path);
                }
            }
        }
    }
    None
}

// バックアップを作成する関数
fn create_backup(pmm_path: &Path, config: &Config) {
    let backup_dir = PathBuf::from(&config.backup_dir);
    if !backup_dir.exists() {
        if let Err(e) = fs::create_dir_all(&backup_dir) {
            eprintln!("[MMD Backup] Failed to create backup directory: {}", e);
            return;
        }
    }

    let now: DateTime<Local> = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let pmm_filename = pmm_path.file_stem().unwrap().to_str().unwrap();
    let backup_filename = format!("{}_{}.pmm.bak", pmm_filename, timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    match fs::copy(pmm_path, &backup_path) {
        Ok(_) => {
            println!("[MMD Backup] Created backup: {}", backup_path.display());
            cleanup_old_backups(&backup_dir, pmm_filename, config.max_backups);
        }
        Err(e) => eprintln!("[MMD Backup] Failed to create backup: {}", e),
    }
}

// 古いバックアップを整理する関数 (変更なし)
fn cleanup_old_backups(backup_dir: &Path, base_filename: &str, max_backups: usize) {
    // (この関数の内容は前回と同じなので省略)
}
