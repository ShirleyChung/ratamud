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

/// 戰鬥狀態
#[derive(Debug, Clone, PartialEq)]
pub enum CombatState {
    None,                                    // 無戰鬥
    InCombat {
        participants: Vec<String>,           // 參與者列表 (包含 "me" 和 NPC IDs)
        round: i32,                          // 當前回合數
    },
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
#[derive(Clone)]
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
    pub npc_manager: crate::npc_manager::NpcManager,
    pub quest_manager: QuestManager,
    pub current_controlled_id: String,  // 當前操控的角色 ID (預設是 "me")
    pub original_player: Option<Person>,  // 原始玩家資料備份
    pub interaction_state: InteractionState,  // NPC 互動狀態
    pub combat_state: CombatState,       // 戰鬥狀態
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl GameWorld {
    pub fn new() -> Self {
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
            npc_manager: crate::npc_manager::NpcManager::new(),
            quest_manager: QuestManager::new(),
            current_controlled_id: "me".to_string(),
            original_player: None,
            interaction_state: InteractionState::None,
            combat_state: CombatState::None,
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
    
    // ==================== 新架構方法 ====================
    
    /// 建立所有 NPC 的視圖（不可變快照）
    /// 這些視圖將傳送給 NPC AI 執行緒用於決策
    /// me: 當前玩家（用於計算 nearby_entities）
    pub fn build_npc_views(&self, me: &Person) -> std::collections::HashMap<String, crate::npc_view::NpcView> {
        use crate::npc_view::{NpcView, Position, GameTime, TerrainInfo};
        
        let mut views = std::collections::HashMap::new();
        
        // 獲取所有 NPC
        let all_npc_ids = self.npc_manager.get_all_npc_ids();
        
        for npc_id in all_npc_ids {
            // 跳過當前被玩家控制的角色（不應由 AI 控制）
            if npc_id == self.current_controlled_id {
                continue;
            }
            
            if let Some(npc) = self.npc_manager.get_npc(&npc_id) {
                // 建立時間資訊
                let time = GameTime {
                    hour: self.time.hour,
                    minute: self.time.minute,
                    second: self.time.second,
                    day: self.time.day,
                };
                
                // 建立地形資訊
                let terrain = if let Some(map) = self.maps.get(&npc.map) {
                    if let Some(point) = map.get_point(npc.x, npc.y) {
                        TerrainInfo {
                            walkable: point.walkable,
                            description: point.description.clone(),
                        }
                    } else {
                        TerrainInfo {
                            walkable: false,
                            description: "未知區域".to_string(),
                        }
                    }
                } else {
                    TerrainInfo {
                        walkable: false,
                        description: "未知區域".to_string(),
                    }
                };
                
                // 建立附近實體列表
                let nearby_entities = self.get_nearby_entities_for_view(me, &npc.map, npc.x, npc.y, 5);
                
                // 建立可見物品列表
                let visible_items = self.get_visible_items_for_view(&npc.map, npc.x, npc.y);
                
                // 建立 NPC 背包物品列表
                let self_items: Vec<(String, u32)> = npc.items.iter()
                    .map(|(name, count)| (name.clone(), *count))
                    .collect();
                
                // 檢查 NPC 是否在戰鬥中
                let in_combat = match &self.combat_state {
                    CombatState::InCombat { participants, .. } => {
                        participants.contains(&npc_id)
                    },
                    _ => false,
                };
                
                // 建立 NpcView
                let view = NpcView {
                    self_id: npc_id.clone(),
                    self_pos: Position { x: npc.x, y: npc.y },
                    self_hp: npc.hp,
                    self_max_hp: npc.max_hp,
                    self_mp: npc.mp,
                    self_items,
                    current_map: npc.map.clone(),
                    time,
                    nearby_entities,
                    visible_items,
                    terrain,
                    is_interacting: npc.is_interacting,
                    in_party: npc.party_leader.is_some(),
                    in_combat,
                };
                
                views.insert(npc_id, view);
            }
        }
        
        views
    }
    
    /// 獲取附近的實體（用於建立 NpcView）
    /// me: 當前玩家
    fn get_nearby_entities_for_view(&self, me: &Person, map_name: &str, x: usize, y: usize, radius: usize) -> Vec<crate::npc_view::EntityInfo> {
        use crate::npc_view::{EntityInfo, EntityType, Position};
        
        let mut entities = Vec::new();
        
        // 檢查玩家是否在附近
        if me.map == map_name {
            let dist_x = (me.x as i32 - x as i32).unsigned_abs() as usize;
            let dist_y = (me.y as i32 - y as i32).unsigned_abs() as usize;
            
            if dist_x <= radius && dist_y <= radius {
                entities.push(EntityInfo {
                    entity_type: EntityType::Player,
                    id: "player".to_string(),
                    pos: Position { x: me.x, y: me.y },
                    name: me.name.clone(),
                });
            }
        }
        
        // 檢查其他 NPC 是否在附近
        let all_npc_ids = self.npc_manager.get_all_npc_ids();
        for npc_id in all_npc_ids {
            if let Some(npc) = self.npc_manager.get_npc(&npc_id) {
                if npc.map == map_name {
                    let dist_x = (npc.x as i32 - x as i32).unsigned_abs() as usize;
                    let dist_y = (npc.y as i32 - y as i32).unsigned_abs() as usize;
                    
                    if dist_x <= radius && dist_y <= radius && !(npc.x == x && npc.y == y) {
                        entities.push(EntityInfo {
                            entity_type: EntityType::Npc,
                            id: npc_id.clone(),
                            pos: Position { x: npc.x, y: npc.y },
                            name: npc.name.clone(),
                        });
                    }
                }
            }
        }
        
        entities
    }
    
    /// 獲取可見的物品（用於建立 NpcView）
    fn get_visible_items_for_view(&self, map_name: &str, x: usize, y: usize) -> Vec<crate::npc_view::ItemInfo> {
        use crate::npc_view::{ItemInfo, Position};
        
        let mut items = Vec::new();
        
        if let Some(map) = self.maps.get(map_name) {
            if let Some(point) = map.get_point(x, y) {
                for (item_name, count) in &point.objects {
                    items.push(ItemInfo {
                        item_name: item_name.clone(),
                        count: *count,
                        pos: Position { x, y },
                    });
                }
            }
        }
        
        items
    }
    
    /// 套用遊戲事件（新架構的核心方法）
    /// 這是 GameWorld 的單一寫入點
    pub fn apply_event(&mut self, event: crate::game_event::GameEvent) -> Vec<crate::message::Message> {
        use crate::game_event::GameEvent;
        
        match event {
            GameEvent::NpcActions { npc_id, actions } => {
                self.apply_npc_actions(npc_id, actions)
            },
            GameEvent::TimerTick { elapsed_secs } => {
                self.apply_timer_tick(elapsed_secs)
            },
            GameEvent::Input(_input_event) => {
                // 輸入事件暫時不處理（由現有系統處理）
                Vec::new()
            },
        }
    }
    
    /// 套用 NPC 行為
    fn apply_npc_actions(&mut self, npc_id: String, actions: Vec<crate::npc_action::NpcAction>) -> Vec<crate::message::Message> {
        use crate::npc_action::NpcAction;
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        for action in actions {
            let action_messages = match action {
                NpcAction::Say(text) => {
                    if let Some(npc) = self.npc_manager.get_npc(&npc_id) {
                        vec![Message::NpcSay {
                            npc_id: npc_id.clone(),
                            npc_name: npc.name.clone(),
                            text,
                        }]
                    } else {
                        Vec::new()
                    }
                },
                NpcAction::Move(direction) => {
                    self.apply_npc_move(&npc_id, direction)
                },
                NpcAction::PickupItem { item_name, quantity } => {
                    self.apply_npc_pickup(&npc_id, item_name, quantity)
                },
                NpcAction::UseItem(item_name) => {
                    self.apply_npc_use_item(&npc_id, item_name)
                },
                NpcAction::DropItem { item_name, quantity } => {
                    self.apply_npc_drop(&npc_id, item_name, quantity)
                },
                NpcAction::UseCombatSkill { skill_name, target_id } => {
                    self.apply_npc_combat_skill(&npc_id, &skill_name, &target_id)
                },
                NpcAction::Idle => {
                    Vec::new()
                },
                _ => {
                    // 其他行為暫未實現
                    Vec::new()
                }
            };
            
            messages.extend(action_messages);
        }
        
        messages
    }
    
    /// 套用 NPC 移動
    fn apply_npc_move(&mut self, npc_id: &str, direction: crate::npc_action::Direction) -> Vec<crate::message::Message> {
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        if let Some(npc) = self.npc_manager.get_npc(npc_id) {
            let (dx, dy) = direction.to_delta();
            let new_x = (npc.x as i32 + dx) as usize;
            let new_y = (npc.y as i32 + dy) as usize;
            let old_x = npc.x;
            let old_y = npc.y;
            let npc_name = npc.name.clone();
            let npc_map = npc.map.clone();
            
            // 檢查是否可行走
            let can_walk = if let Some(map) = self.maps.get(&npc_map) {
                if new_x < map.width && new_y < map.height {
                    if let Some(point) = map.get_point(new_x, new_y) {
                        point.walkable
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
            
            if can_walk {
                if let Some(npc_mut) = self.npc_manager.get_npc_mut(npc_id) {
                    npc_mut.move_to(new_x, new_y);
                    messages.push(Message::Movement {
                        entity: npc_name,
                        from: (old_x, old_y),
                        to: (new_x, new_y),
                    });
                }
            }
        }
        
        messages
    }
    
    /// 套用 NPC 撿起物品
    fn apply_npc_pickup(&mut self, npc_id: &str, item_name: String, quantity: u32) -> Vec<crate::message::Message> {
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        if let Some(npc) = self.npc_manager.get_npc(npc_id) {
            let npc_x = npc.x;
            let npc_y = npc.y;
            let npc_map = npc.map.clone();
            let npc_name = npc.name.clone();
            
            // 從地圖移除物品
            if let Some(map) = self.maps.get_mut(&npc_map) {
                if let Some(point) = map.get_point_mut(npc_x, npc_y) {
                    if let Some(count) = point.objects.get_mut(&item_name) {
                        let actual_quantity = (*count).min(quantity);
                        *count -= actual_quantity;
                        
                        if *count == 0 {
                            point.objects.remove(&item_name);
                        }
                        
                        // 添加到 NPC 背包
                        if let Some(npc_mut) = self.npc_manager.get_npc_mut(npc_id) {
                            npc_mut.add_items(item_name.clone(), actual_quantity);
                            
                            messages.push(Message::ItemPickup {
                                entity: npc_name,
                                item: item_name,
                                count: actual_quantity,
                            });
                        }
                    }
                }
            }
        }
        
        messages
    }
    
    /// 套用 NPC 使用物品
    fn apply_npc_use_item(&mut self, npc_id: &str, item_name: String) -> Vec<crate::message::Message> {
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        if let Some(npc) = self.npc_manager.get_npc(npc_id) {
            let npc_name = npc.name.clone();
            
            // 檢查是否擁有物品
            if npc.items.contains_key(&item_name) {
                // 檢查是否為食物
                if crate::item_registry::is_food(&item_name) {
                    if let Some(hp_restore) = crate::item_registry::get_food_hp(&item_name) {
                        // 移除物品
                        if let Some(npc_mut) = self.npc_manager.get_npc_mut(npc_id) {
                            npc_mut.drop_items(&item_name, 1);
                            
                            // 回復 HP
                            let old_hp = npc_mut.hp;
                            npc_mut.hp = (npc_mut.hp + hp_restore).min(npc_mut.max_hp);
                            let actual_restore = npc_mut.hp - old_hp;
                            
                            messages.push(Message::ItemUse {
                                entity: npc_name,
                                item: item_name,
                                effect: format!("回復了 {actual_restore} HP"),
                            });
                        }
                    }
                }
            }
        }
        
        messages
    }
    
    /// 套用 NPC 放下物品
    fn apply_npc_drop(&mut self, npc_id: &str, item_name: String, quantity: u32) -> Vec<crate::message::Message> {
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        if let Some(npc) = self.npc_manager.get_npc(npc_id) {
            let npc_x = npc.x;
            let npc_y = npc.y;
            let npc_map = npc.map.clone();
            let npc_name = npc.name.clone();
            
            // 檢查是否擁有足夠的物品
            if let Some(count) = npc.items.get(&item_name) {
                let actual_quantity = (*count).min(quantity);
                
                // 從 NPC 背包移除
                if let Some(npc_mut) = self.npc_manager.get_npc_mut(npc_id) {
                    npc_mut.drop_items(&item_name, actual_quantity);
                    
                    // 添加到地圖
                    if let Some(map) = self.maps.get_mut(&npc_map) {
                        if let Some(point) = map.get_point_mut(npc_x, npc_y) {
                            point.add_objects(item_name.clone(), actual_quantity);
                            
                            messages.push(Message::Log(
                                format!("{npc_name} 放下了 {item_name} x{actual_quantity}")
                            ));
                        }
                    }
                }
            }
        }
        
        messages
    }
    
    /// 套用 NPC 使用戰鬥技能
    fn apply_npc_combat_skill(&mut self, npc_id: &str, skill_name: &str, target_id: &str) -> Vec<crate::message::Message> {
        use crate::message::Message;
        
        let mut messages = Vec::new();
        
        // 檢查NPC是否存在並且冷卻完成
        if let Some(npc) = self.npc_manager.get_npc(npc_id) {
            let cooldown = npc.get_skill_cooldown(skill_name);
            if cooldown > 0 {
                // 技能還在冷卻中，跳過
                return messages;
            }
            
            let skill_dialogue = npc.get_skill_dialogue(skill_name);
            let damage = npc.combat_skills.get(skill_name)
                .map(|s| s.damage)
                .unwrap_or(1);
            let npc_name = npc.name.clone();
            
            // 這裡只記錄消息，實際的HP扣除和技能冷卻需要在主循環中處理
            // 因為這裡只有不可變引用
            messages.push(Message::CombatAction {
                attacker_id: npc_id.to_string(),
                attacker_name: npc_name,
                skill_name: skill_name.to_string(),
                skill_dialogue,
                target_id: target_id.to_string(),
                damage,
            });
        }
        
        messages
    }
    
    /// 套用時間更新
    fn apply_timer_tick(&mut self, _elapsed_secs: u64) -> Vec<crate::message::Message> {
        // 時間更新由現有的 update_time 方法處理
        Vec::new()
    }
}
