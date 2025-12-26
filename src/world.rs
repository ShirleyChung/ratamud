use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::map::{Map, MapType};
use crate::person::Person;
use crate::time_updatable::{TimeInfo, TimeUpdatable};
use crate::quest::QuestManager;

/// NPC 互動狀態
/// 用於追蹤玩家正在與哪個 NPC 進行什麼類型的互動
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionState {
    None,                                    // 無互動
    Trading { npc_name: String },           // 交易中（買/賣選單）
    Buying { npc_name: String },            // 購買物品選單
    Selling { npc_name: String },           // 出售物品選單
}

// 世界時間結構體
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTime {
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,    // 0-59
    pub day: u32,      // 遊戲日期
    last_update: u64,  // 上次更新的實時時間戳（毫秒）
}

impl Default for WorldTime {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldTime {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // 世界自己的時間，從遊戲設定的時間開始
        WorldTime {
            hour: 9,
            minute: 0,
            second: 0,
            day: 1,  // 遊戲世界的第1天
            last_update: now,
        }
    }

    // 推進時間（與真實世界同步：1實際秒 = 1遊戲秒）
    pub fn advance(&mut self, game_speed: f32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let elapsed_real_ms = now - self.last_update;
        // game_speed = 1.0 表示與真實世界同步
        let elapsed_game_secs = ((elapsed_real_ms as f32 / 1000.0) * game_speed) as u32;
        
        let total_secs = self.second as u32 + elapsed_game_secs;
        
        // 計算分鐘和秒
        let mins = total_secs / 60;
        self.second = (total_secs % 60) as u8;
        
        let total_mins = self.minute as u32 + mins;
        
        // 計算小時和分鐘
        let hours = total_mins / 60;
        self.minute = (total_mins % 60) as u8;
        
        // 計算天數
        let total_hours = self.hour as u32 + hours;
        self.hour = (total_hours % 24) as u8;
        self.day += total_hours / 24;
        
        self.last_update = now;
    }

    pub fn format_time(&self) -> String {
        format!("Day {} {:02}:{:02}:{:02}", self.day, self.hour, self.minute, self.second)
    }
}

// 世界元數據
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub name: String,
    pub description: String,
    pub maps: Vec<String>,
    #[serde(default)]
    pub current_map: String,
}

impl WorldMetadata {
    pub fn new(name: String, description: String) -> Self {
        WorldMetadata {
            name,
            description,
            maps: Vec::new(),
            current_map: String::new(),
        }
    }

    pub fn add_map(&mut self, map_name: String) {
        if !self.maps.contains(&map_name) {
            self.maps.push(map_name);
        }
    }
}

// 遊戲世界 - 管理多個地圖和時間
pub struct GameWorld {
    pub maps: HashMap<String, Map>,
    pub current_map_name: String,
    pub metadata: WorldMetadata,
    pub world_dir: String,
    pub time: WorldTime,
    pub _game_speed: f32,
    pub event_manager: crate::event::EventManager,
    pub event_scheduler: crate::event_scheduler::EventScheduler,
    pub time_thread: Option<crate::time_thread::TimeThread>,
    pub npc_ai_thread: Option<crate::npc_ai_thread::NpcAiThread>,
    pub npc_manager: crate::npc_manager::NpcManager,
    pub quest_manager: QuestManager,
    pub current_controlled_id: Option<String>,  // 當前操控的角色 ID (None = 原始玩家)
    pub original_player: Option<Person>,         // 原始玩家資料備份
    pub player: Person,
    pub interaction_state: InteractionState,     // NPC 互動狀態
}

impl GameWorld {
    pub fn new(player: Person) -> Self {
        // 建立世界資料夾
        let world_dir = "worlds/beginWorld".to_string();
        let _ = fs::create_dir_all(&world_dir);

        // 創建世界元數據
        let metadata = WorldMetadata::new(
            "beginWorld".to_string(),
            "這是一個充滿奇異生物、神秘遺跡和隱藏寶藏的魔幻世界。\
            世界中有多個不同的區域，包括 forest、洞窟、城鎮和古老的廢墟。\
            在這個世界中，你將探索未知的領域，與各種NPC互動，收集物品和知識。".to_string(),
        );

        let time = WorldTime::new();
        let game_speed = 1.0;  // 與真實世界同步：1實際秒 = 1遊戲秒
        let time_thread = crate::time_thread::TimeThread::new(time.clone(), game_speed);

        GameWorld {
            maps: HashMap::new(),
            current_map_name: String::from("beginMap"),
            metadata,
            world_dir,
            time,
            _game_speed: game_speed,
            event_manager: crate::event::EventManager::new(),
            event_scheduler: crate::event_scheduler::EventScheduler::new(),
            time_thread: Some(time_thread),
            npc_ai_thread: None,  // 稍後初始化
            npc_manager: crate::npc_manager::NpcManager::new(),
            quest_manager: QuestManager::new(),
            current_controlled_id: None,
            original_player: Some(player.clone()),
            player,
            interaction_state: InteractionState::None,  // 初始無互動狀態
        }
    }

    // 添加地圖
    pub fn add_map(&mut self, map: Map) {
        let map_name = map.name.clone();
        self.maps.insert(map_name.clone(), map);
        self.metadata.add_map(map_name);
    }

    // 切換地圖
    pub fn change_map(&mut self, map_name: &str) -> bool {
        if self.maps.contains_key(map_name) {
            self.current_map_name = map_name.to_string();
            true
        } else {
            false
        }
    }

    // 獲取當前地圖
    pub fn get_current_map(&self) -> Option<&Map> {
        self.maps.get(&self.current_map_name)
    }

    // 獲取可變的當前地圖
    pub fn get_current_map_mut(&mut self) -> Option<&mut Map> {
        self.maps.get_mut(&self.current_map_name)
    }

    // 獲取 maps 資料夾路徑
    pub fn get_maps_dir(&self) -> String {
        format!("{}/maps", self.world_dir)
    }

    // 保存地圖到檔案
    pub fn save_map(&self, map: &Map) -> Result<(), Box<dyn std::error::Error>> {
        let maps_dir = self.get_maps_dir();
        std::fs::create_dir_all(&maps_dir)?;
        let map_path = format!("{}/{}.json", maps_dir, map.name);
        map.save(&map_path)?;
        Ok(())
    }

    // 從檔案載入地圖
    #[allow(dead_code)]
    pub fn load_map(&mut self, map_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let maps_dir = self.get_maps_dir();
        let map_path = format!("{maps_dir}/{map_name}.json");
        let map = Map::load(&map_path)?;
        self.maps.insert(map_name.to_string(), map);
        Ok(())
    }
    
    /// 初始化並載入所有地圖
    /// 返回 (總地圖數, 日誌訊息列表)
    pub fn initialize_maps(&mut self) -> Result<(usize, Vec<String>), Box<dyn std::error::Error>> {
        // 初始化並載入所有地圖
        let maps_config = vec![
            ("beginMap", MapType::Normal),
            ("forest", MapType::Forest),
            ("洞穴", MapType::Cave),
            ("沙漠", MapType::Desert),
            ("mountain", MapType::Mountain),
        ];
        
        let mut logs = Vec::new();
        
        // 更新世界元數據
        self.metadata.maps = maps_config.iter().map(|(name, _)| name.to_string()).collect();
        
        // 建立 maps 資料夾
        std::fs::create_dir_all(self.get_maps_dir())?;
        
        // 生成並保存地圖
        for (map_name, map_type) in maps_config {
            let map_path = format!("{}/{}.json", self.get_maps_dir(), map_name);
            
            let map = if std::path::Path::new(&map_path).exists() {
                // 如果檔案存在，則加載（不要重新初始化物品）
                Map::load(&map_path)?
            } else {
                // 否則生成新地圖
                let mut new_map = Map::new_with_type(map_name.to_string(), 100, 100, map_type);
                // 只在新地圖時初始化物品
                new_map.initialize_items();
                // 保存新地圖
                new_map.save(&map_path)?;
                new_map
            };
            
            logs.push(format!("地圖已加載: {}", map.name));
            let (walkable, unwalkable) = map.get_stats();
            logs.push(format!("{map_name} - 可行走點: {walkable}, 不可行走點: {unwalkable}"));
            
            self.add_map(map);
        }
        
        // 保存世界元數據
        self.save_metadata()?;
        
        Ok((self.maps.len(), logs))
    }

    // 保存世界元數據
    pub fn save_metadata(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = format!("{}/world.json", self.world_dir);
        let mut metadata = self.metadata.clone();
        metadata.current_map = self.current_map_name.clone();
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, json)?;
        Ok(())
    }

    // 加載世界元數據
    pub fn load_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = format!("{}/world.json", self.world_dir);
        if Path::new(&metadata_path).exists() {
            let json = fs::read_to_string(metadata_path)?;
            self.metadata = serde_json::from_str(&json)?;
            if !self.metadata.current_map.is_empty() {
                self.current_map_name = self.metadata.current_map.clone();
            }
        }
        Ok(())
    }

    // 更新世界時間 (從時鐘線程同步)
    pub fn update_time(&mut self) {
        if let Some(ref time_thread) = self.time_thread {
            self.time = time_thread.get_time();
        }
        
        // 更新所有地圖上的物品年齡
        let time_info = self.get_time_info();
        for map in self.maps.values_mut() {
            map.on_time_update(&time_info);
        }
        
        // 更新所有 NPC 的年齡
        self.npc_manager.update_all_time(&time_info);
    }

    // 獲取當前時間信息
    pub fn get_time_info(&self) -> TimeInfo {
        TimeInfo::new_with_seconds(self.time.hour, self.time.minute, self.time.second, self.time.day)
    }

    // 獲取格式化的時間字符串
    pub fn format_time(&self) -> String {
        self.time.format_time()
    }

    // 保存世界時間
    pub fn save_time(&self) -> Result<(), Box<dyn std::error::Error>> {
        let time_path = format!("{}/time.json", self.world_dir);
        let json = serde_json::to_string_pretty(&self.time)?;
        fs::write(time_path, json)?;
        Ok(())
    }

    // 加載世界時間
    pub fn load_time(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let time_path = format!("{}/time.json", self.world_dir);
        if Path::new(&time_path).exists() {
            let json = fs::read_to_string(time_path)?;
            let mut loaded_time: WorldTime = serde_json::from_str(&json)?;
            // 重置 last_update 為當前時間，避免時間跳躍
            loaded_time.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            
            self.time = loaded_time.clone();
            
            // 重新啟動時鐘線程
            if let Some(ref mut time_thread) = self.time_thread {
                time_thread.set_time(loaded_time);
            }
        }
        Ok(())
    }
    
    // 保存物品流水號計數器
    #[allow(dead_code)]
    pub fn save_item_counter(&self) -> Result<(), Box<dyn std::error::Error>> {
        let counter_path = format!("{}/item_counter.json", self.world_dir);
        let counter = crate::item::get_item_id_counter();
        let json = serde_json::to_string(&counter)?;
        fs::write(counter_path, json)?;
        Ok(())
    }
    
    // 載入物品流水號計數器
    #[allow(dead_code)]
    pub fn load_item_counter(&self) -> Result<(), Box<dyn std::error::Error>> {
        let counter_path = format!("{}/item_counter.json", self.world_dir);
        if Path::new(&counter_path).exists() {
            let json = fs::read_to_string(counter_path)?;
            let counter: u64 = serde_json::from_str(&json)?;
            crate::item::set_item_id_counter(counter);
        }
        Ok(())
    }
    
    /// 從 NPC AI 執行緒獲取日誌
    pub fn get_npc_ai_logs(&self) -> Vec<String> {
        if let Some(ref ai_thread) = self.npc_ai_thread {
            ai_thread.get_logs()
        } else {
            Vec::new()
        }
    }
}

impl Drop for GameWorld {
    fn drop(&mut self) {
        // 停止時間執行緒
        if let Some(ref mut thread) = self.time_thread {
            thread.stop();
        }
        // 停止 NPC AI 執行緒
        if let Some(ref mut thread) = self.npc_ai_thread {
            thread.stop();
        }
    }
}
