use crate::event::{EventManager, GameEvent};
use std::fs;
use std::path::Path;

/// 事件加載器
pub struct EventLoader;

impl EventLoader {
    /// 從目錄加載所有事件腳本
    pub fn load_from_directory(
        event_manager: &mut EventManager,
        dir_path: &str,
    ) -> Result<(usize, Vec<String>), Box<dyn std::error::Error>> {
        let mut loaded_events = Vec::new();
        
        if !Path::new(dir_path).exists() {
            fs::create_dir_all(dir_path)?;
            return Ok((0, loaded_events));
        }
        
        // 遞迴讀取目錄
        Self::load_recursive(event_manager, dir_path, &mut loaded_events)?;
        
        let count = loaded_events.len();
        Ok((count, loaded_events))
    }
    
    fn load_recursive(
        event_manager: &mut EventManager,
        dir_path: &str,
        loaded_events: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // 遞迴處理子目錄
                Self::load_recursive(event_manager, path.to_str().unwrap(), loaded_events)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // 加載 JSON 事件檔案
                match Self::load_event_file(&path) {
                    Ok(event) => {
                        let event_info = format!("{} ({})", event.name, event.id);
                        loaded_events.push(event_info);
                        event_manager.add_event(event);
                    }
                    Err(e) => {
                        eprintln!("載入事件檔案 {path:?} 失敗: {e}");
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 從文件加載單個事件
    fn load_event_file(path: &Path) -> Result<GameEvent, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let event: GameEvent = serde_json::from_str(&content)?;
        Ok(event)
    }
    
    /// 保存事件到文件
    #[allow(dead_code)]
    pub fn save_event_file(
        event: &GameEvent,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(event)?;
        
        // 確保目錄存在
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(file_path, json)?;
        Ok(())
    }
    
    /// 創建範例事件腳本
    #[allow(dead_code)]
    pub fn create_example_events(world_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let events_dir = format!("{world_dir}/events");
        
        // 創建目錄結構
        fs::create_dir_all(format!("{events_dir}/time_based"))?;
        fs::create_dir_all(format!("{events_dir}/random"))?;
        fs::create_dir_all(format!("{events_dir}/location"))?;
        
        // 範例1: 每10分鐘的商人到訪
        let merchant_event = serde_json::json!({
            "id": "merchant_visit",
            "name": "商人到訪",
            "description": "旅行商人定期來到城鎮廣場",
            "trigger": {
                "type": "time",
                "schedule": "*/10 * * * *",
                "time_range": ["09:00:00", "18:00:00"]
            },
            "where": {
                "map": "初始之地"
            },
            "actions": [
                {
                    "type": "message",
                    "text": "一位商人來到了廣場"
                },
                {
                    "type": "spawn_npc",
                    "npc_id": "merchant_01",
                    "position": [50, 50],
                    "dialogue": "歡迎！看看我的商品吧！"
                }
            ],
            "state": {
                "repeatable": true,
                "cooldown": 0,
                "max_triggers": -1,
                "prerequisites": []
            }
        });
        
        fs::write(
            format!("{events_dir}/time_based/merchant_visit.json"),
            serde_json::to_string_pretty(&merchant_event)?
        )?;
        
        // 範例2: 隨機寶藏出現
        let treasure_event = serde_json::json!({
            "id": "random_treasure",
            "name": "神秘寶藏",
            "description": "隨機地點出現寶藏",
            "trigger": {
                "type": "time",
                "schedule": "*/5 * * * *",
                "random_chance": 0.3
            },
            "where": {
                "map": "初始之地"
            },
            "actions": [
                {
                    "type": "add_item",
                    "item": "神秘寶箱",
                    "position": [30, 30]
                },
                {
                    "type": "message",
                    "text": "你感覺到附近有什麼特別的東西..."
                }
            ],
            "state": {
                "repeatable": true,
                "cooldown": 300,
                "max_triggers": -1,
                "prerequisites": []
            }
        });
        
        fs::write(
            format!("{events_dir}/random/treasure_spawn.json"),
            serde_json::to_string_pretty(&treasure_event)?
        )?;
        
        // 範例3: 進入特定位置觸發
        let location_event = serde_json::json!({
            "id": "discover_shrine",
            "name": "發現神殿",
            "description": "玩家發現古老的神殿",
            "trigger": {
                "type": "location",
                "positions": [[25, 25]]
            },
            "actions": [
                {
                    "type": "message",
                    "text": "你發現了一座古老的神殿！"
                }
            ],
            "state": {
                "repeatable": false,
                "cooldown": 0,
                "max_triggers": 1,
                "prerequisites": []
            }
        });
        
        fs::write(
            format!("{events_dir}/location/discover_shrine.json"),
            serde_json::to_string_pretty(&location_event)?
        )?;
        
        Ok(())
    }
}
