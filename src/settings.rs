use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

// 遊戲設定結構體
#[derive(Serialize, Deserialize, Clone)]
pub struct GameSettings {
    // 是否顯示小地圖
    pub show_minimap: bool,
    // 是否顯示日誌視窗
    pub show_log: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            show_minimap: false,
            show_log: true,  // 日誌視窗預設開啟
        }
    }
}

impl GameSettings {
    // 設定文件路徑（相對於資料根目錄，見 crate::paths）
    fn settings_dir() -> String {
        crate::paths::resolve("worlds")
    }

    fn settings_file() -> String {
        crate::paths::resolve("worlds/settings.json")
    }

    // 從文件載入設定
    pub fn load() -> Self {
        let settings_file = Self::settings_file();
        // 如果設定文件存在，嘗試加載它
        if Path::new(&settings_file).exists() {
            match fs::read_to_string(&settings_file) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(settings) => return settings,
                        Err(_) => return Self::default(),
                    }
                }
                Err(_) => return Self::default(),
            }
        }
        
        // 如果文件不存在，返回默認設定
        Self::default()
    }

    // 保存設定到文件
    pub fn save(&self) -> std::io::Result<()> {
        // 確保目錄存在
        fs::create_dir_all(Self::settings_dir())?;

        // 序列化並寫入文件
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        fs::write(Self::settings_file(), json)?;
        Ok(())
    }
}
