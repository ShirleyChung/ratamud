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
    // 設定文件路徑
    const SETTINGS_DIR: &'static str = "worlds";
    const SETTINGS_FILE: &'static str = "worlds/settings.json";

    // 從文件載入設定
    pub fn load() -> Self {
        // 如果設定文件存在，嘗試加載它
        if Path::new(Self::SETTINGS_FILE).exists() {
            match fs::read_to_string(Self::SETTINGS_FILE) {
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
        fs::create_dir_all(Self::SETTINGS_DIR)?;
        
        // 序列化並寫入文件
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(Self::SETTINGS_FILE, json)?;
        Ok(())
    }
}
