use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::map::Map;
use crate::person::Person;
use crate::time_updatable::TimeInfo;

// 世界時間結構體
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTime {
    pub hour: u8,      // 0-23
    pub minute: u8,    // 0-59
    pub second: u8,    // 0-59
    pub day: u32,      // 遊戲日期
    last_update: u64,  // 上次更新的實時時間戳（毫秒）
}

impl WorldTime {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        WorldTime {
            hour: 9,
            minute: 0,
            second: 0,
            day: 1,
            last_update: now,
        }
    }

    // 推進時間（1實際秒 = 60遊戲秒）
    pub fn advance(&mut self, game_speed: f32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let elapsed_real_ms = now - self.last_update;
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

// 座標結構體
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }
}

// 物件類型
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ObjectType {
    Weapon,
    Armor,
    Potion,
    Treasure,
}

// 世界物件
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorldObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub object_type: ObjectType,
    pub position: Position,
}

impl WorldObject {
    #[allow(dead_code)]
    pub fn new(id: String, name: String, description: String, object_type: ObjectType, position: Position) -> Self {
        WorldObject {
            id,
            name,
            description,
            object_type,
            position,
        }
    }
}

// NPC 結構體
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub description: String,
    pub position: Position,
    pub health: i32,
    pub max_health: i32,
    pub dialogue: Vec<String>, // NPC 的對話
}

impl NPC {
    #[allow(dead_code)]
    pub fn new(id: String, name: String, description: String, position: Position, max_health: i32) -> Self {
        NPC {
            id,
            name,
            description,
            position,
            health: max_health,
            max_health,
            dialogue: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_dialogue(&mut self, text: String) {
        self.dialogue.push(text);
    }

    #[allow(dead_code)]
    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
    }

    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }
}

// 世界地圖區域
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MapArea {
    pub id: String,
    pub name: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
}

impl MapArea {
    #[allow(dead_code)]
    pub fn new(id: String, name: String, description: String, width: i32, height: i32) -> Self {
        MapArea {
            id,
            name,
            description,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn contains_position(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }
}

// 主世界結構體
#[allow(dead_code)]
pub struct World {
    pub time: WorldTime,
    pub game_speed: f32, // 遊戲速度（實際秒數 = 遊戲分鐘數 / game_speed）
    pub map_areas: HashMap<String, MapArea>,
    pub objects: HashMap<String, WorldObject>,
    pub npcs: HashMap<String, NPC>,
    current_area: String,
}

impl World {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut world = World {
            time: WorldTime::new(),
            game_speed: 60.0, // 預設：1 實際秒 = 60 遊戲秒
            map_areas: HashMap::new(),
            objects: HashMap::new(),
            npcs: HashMap::new(),
            current_area: String::from("main"),
        };

        // 初始化主要區域
        world.add_area(MapArea::new(
            "main".to_string(),
            "Main Hall".to_string(),
            "A vast hall with torches lighting the walls".to_string(),
            50,
            50,
        ));

        world
    }

    #[allow(dead_code)]
    pub fn add_area(&mut self, area: MapArea) {
        self.map_areas.insert(area.id.clone(), area);
    }

    #[allow(dead_code)]
    pub fn add_object(&mut self, obj: WorldObject) {
        self.objects.insert(obj.id.clone(), obj);
    }

    #[allow(dead_code)]
    pub fn add_npc(&mut self, npc: NPC) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    #[allow(dead_code)]
    pub fn update(&mut self) {
        self.time.advance(self.game_speed);
    }

    #[allow(dead_code)]
    pub fn get_status(&self) -> String {
        let current_area = self.map_areas.get(&self.current_area)
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown");
        
        format!(
            "Time: {} | Area: {} | NPCs: {} | Objects: {}",
            self.time.format_time(),
            current_area,
            self.npcs.len(),
            self.objects.len()
        )
    }

    #[allow(dead_code)]
    pub fn get_npcs_at_position(&self, pos: Position) -> Vec<&NPC> {
        self.npcs
            .values()
            .filter(|npc| npc.position == pos && npc.is_alive())
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_objects_at_position(&self, pos: Position) -> Vec<&WorldObject> {
        self.objects
            .values()
            .filter(|obj| obj.position == pos)
            .collect()
    }

    #[allow(dead_code)]
    pub fn move_npc(&mut self, npc_id: &str, new_pos: Position) -> bool {
        if let Some(area) = self.map_areas.get(&self.current_area) {
            if area.contains_position(new_pos) {
                if let Some(npc) = self.npcs.get_mut(npc_id) {
                    npc.position = new_pos;
                    return true;
                }
            }
        }
        false
    }

    #[allow(dead_code)]
    pub fn get_npc_mut(&mut self, npc_id: &str) -> Option<&mut NPC> {
        self.npcs.get_mut(npc_id)
    }

    #[allow(dead_code)]
    pub fn get_npc(&self, npc_id: &str) -> Option<&NPC> {
        self.npcs.get(npc_id)
    }
}

// 世界元數據
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub name: String,
    pub description: String,
    pub maps: Vec<String>,
}

impl WorldMetadata {
    pub fn new(name: String, description: String) -> Self {
        WorldMetadata {
            name,
            description,
            maps: Vec::new(),
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
    pub current_map: String,
    pub metadata: WorldMetadata,
    pub world_dir: String,
    pub time: WorldTime,
    pub game_speed: f32,
    pub event_manager: crate::event::EventManager,
    pub event_scheduler: crate::event_scheduler::EventScheduler,
}

impl GameWorld {
    pub fn new() -> Self {
        // 建立世界資料夾
        let world_dir = "worlds/初始世界".to_string();
        let _ = fs::create_dir_all(&world_dir);

        // 創建世界元數據
        let metadata = WorldMetadata::new(
            "初始世界".to_string(),
            "這是一個充滿奇異生物、神秘遺跡和隱藏寶藏的魔幻世界。\
            世界中有多個不同的區域，包括森林、洞窟、城鎮和古老的廢墟。\
            在這個世界中，你將探索未知的領域，與各種NPC互動，收集物品和知識。".to_string(),
        );

        GameWorld {
            maps: HashMap::new(),
            current_map: String::from("初始之地"),
            metadata,
            world_dir,
            time: WorldTime::new(),
            game_speed: 60.0,
            event_manager: crate::event::EventManager::new(),
            event_scheduler: crate::event_scheduler::EventScheduler::new(),
        }
    }

    // 添加地圖
    pub fn add_map(&mut self, map: Map) {
        let map_name = map.name.clone();
        self.maps.insert(map_name.clone(), map);
        self.metadata.add_map(map_name);
    }

    // 切換地圖
    #[allow(dead_code)]
    pub fn change_map(&mut self, map_name: &str) -> bool {
        if self.maps.contains_key(map_name) {
            self.current_map = map_name.to_string();
            true
        } else {
            false
        }
    }

    // 獲取當前地圖
    pub fn get_current_map(&self) -> Option<&Map> {
        self.maps.get(&self.current_map)
    }

    // 獲取可變的當前地圖
    pub fn get_current_map_mut(&mut self) -> Option<&mut Map> {
        self.maps.get_mut(&self.current_map)
    }

    // 列出所有地圖
    #[allow(dead_code)]
    pub fn list_maps(&self) -> Vec<String> {
        self.maps.keys().cloned().collect()
    }

    // 獲取地圖總數
    pub fn map_count(&self) -> usize {
        self.maps.len()
    }

    // 獲取 maps 資料夾路徑
    pub fn get_maps_dir(&self) -> String {
        format!("{}/maps", self.world_dir)
    }

    // 保存地圖到檔案
    #[allow(dead_code)]
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
        let map_path = format!("{}/{}.json", maps_dir, map_name);
        let map = Map::load(&map_path)?;
        self.maps.insert(map_name.to_string(), map);
        Ok(())
    }

    // 從元數據加載所有地圖
    #[allow(dead_code)]
    pub fn load_all_maps_from_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for map_name in self.metadata.maps.clone() {
            let _ = self.load_map(&map_name);
        }
        Ok(())
    }

    // 保存世界元數據
    pub fn save_metadata(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = format!("{}/world.json", self.world_dir);
        let json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, json)?;
        Ok(())
    }

    // 加載世界元數據
    pub fn load_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = format!("{}/world.json", self.world_dir);
        if Path::new(&metadata_path).exists() {
            let json = fs::read_to_string(metadata_path)?;
            self.metadata = serde_json::from_str(&json)?;
        }
        Ok(())
    }

    // 獲取世界信息
    #[allow(dead_code)]
    pub fn get_world_info(&self) -> String {
        format!(
            "世界: {}\n\n{}\n\n地圖列表: {}",
            self.metadata.name,
            self.metadata.description,
            if self.metadata.maps.is_empty() {
                "無".to_string()
            } else {
                self.metadata.maps.join(", ")
            }
        )
    }

    // 獲取 persons 資料夾路徑
    #[allow(dead_code)]
    pub fn get_persons_dir(&self) -> String {
        format!("{}/persons", self.world_dir)
    }

    // 加載所有 persons
    #[allow(dead_code)]
    pub fn load_all_persons(&self) -> Vec<Person> {
        let person_dir = self.get_persons_dir();
        let mut persons = Vec::new();

        if let Ok(entries) = fs::read_dir(&person_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(person) = Person::load(&person_dir, filename) {
                                persons.push(person);
                            }
                        }
                    }
                }
            }
        }

        persons
    }

    // 更新世界時間
    pub fn update_time(&mut self) {
        self.time.advance(self.game_speed);
    }

    // 獲取當前時間信息
    #[allow(dead_code)]
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
            self.time = serde_json::from_str(&json)?;
            // 重置 last_update 為當前時間，避免時間跳躍
            self.time.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
        Ok(())
    }
}
