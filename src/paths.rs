//! 資料路徑解析
//!
//! 遊戲核心以往把 `worlds/...` 這種**相對路徑**直接寫死在程式裡。桌面/終端版
//! 的工作目錄就是專案根目錄，所以沒問題；但 iOS 的行程工作目錄是唯讀的（而且
//! 不是 app bundle 的位置），所有 `create_dir_all` / `fs::write` 都會失敗，
//! 於是「地圖載不進來、狀態存不起來」。
//!
//! 這個模組提供一個全域的「資料根目錄」。所有核心模組改成透過 [`resolve`]
//! 取得實際路徑：
//!
//! * 終端版不設定 → 根目錄是 `.`，行為與以前完全相同。
//! * iOS host 在 `ratamud_init_game` 之前呼叫 `ratamud_set_data_dir()`
//!   指到 `Documents/`（可寫），再用 `ratamud_seed_data_dir()` 把 app bundle
//!   內的唯讀世界資料複製過去。

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static DATA_ROOT: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::from(".")));

/// 設定資料根目錄（所有 `worlds/...` 都會相對於它）。
/// 必須在建立 `GameWorld` 之前呼叫才會生效。
pub fn set_data_root<P: AsRef<Path>>(root: P) {
    if let Ok(mut guard) = DATA_ROOT.lock() {
        *guard = root.as_ref().to_path_buf();
    }
}

/// 取得目前的資料根目錄。
pub fn data_root() -> PathBuf {
    DATA_ROOT
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// 把相對於資料根目錄的路徑解析成實際路徑字串。
///
/// 根目錄是 `.` 時回傳原本的相對路徑，維持終端版既有行為（也讓既有的
/// `worlds/...` 測試資料繼續可用）。
pub fn resolve(relative: &str) -> String {
    let root = data_root();
    if root == Path::new(".") {
        return relative.to_string();
    }
    root.join(relative).to_string_lossy().into_owned()
}

/// 把 `src` 底下的檔案遞迴複製到 `dst`，**不覆蓋**已存在的檔案。
///
/// 給 iOS host 用來把 app bundle（唯讀）內的世界資料安裝到可寫目錄。
/// 因為不覆蓋，所以重複呼叫是安全的：玩家的存檔不會被 bundle 的初始資料蓋掉。
///
/// 回傳實際複製的檔案數量。
pub fn copy_tree_if_missing(src: &Path, dst: &Path) -> std::io::Result<usize> {
    if !src.is_dir() {
        return Ok(0);
    }
    std::fs::create_dir_all(dst)?;

    let mut copied = 0;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if from.is_dir() {
            copied += copy_tree_if_missing(&from, &to)?;
        } else if !to.exists() {
            std::fs::copy(&from, &to)?;
            copied += 1;
        }
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_keeps_relative_path_by_default() {
        // 預設根目錄是 "."，行為必須與重構前一致
        assert_eq!(data_root(), PathBuf::from("."));
        assert_eq!(resolve("worlds/beginWorld"), "worlds/beginWorld");
    }
}
