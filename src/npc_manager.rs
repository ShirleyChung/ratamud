use crate::person::Person;
use std::collections::HashMap;

/// NPC 管理器，負責管理遊戲中的所有 NPC
#[derive(Clone)]
pub struct NpcManager {
    npcs: HashMap<String, Person>,  // NPC ID -> Person
    npc_aliases: HashMap<String, String>,  // 別名 -> NPC ID
    previous_distances: HashMap<String, usize>,  // 用於追蹤 NPC 與 me 的前一次距離（for 靠近/離開檢測）
}

impl Default for NpcManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NpcManager {
    pub fn new() -> Self {
        NpcManager {
            npcs: HashMap::new(),
            npc_aliases: HashMap::new(),
            previous_distances: HashMap::new(),
        }
    }

    /// 添加 NPC
    pub fn add_npc(&mut self, id: String, mut npc: Person, aliases: Vec<String>) {
        // 確保 NPC 有預設的 10000 金幣（如果沒有或為0）
        if !npc.items.contains_key("金幣") || npc.items.get("金幣").copied().unwrap_or(0) == 0 {
            npc.items.insert("金幣".to_string(), 10_000);
        }
        
        self.npcs.insert(id.clone(), npc);
        
        // 添加別名映射
        for alias in aliases {
            self.npc_aliases.insert(alias.to_lowercase(), id.clone());
        }
        
        // 添加 ID 本身作為別名
        self.npc_aliases.insert(id.to_lowercase(), id.clone());
    }

    /// 通過 ID 或別名獲取 NPC
    pub fn get_npc(&self, name_or_id: &str) -> Option<&Person> {
        let key = name_or_id.to_lowercase();
        
        // 先嘗試通過別名查找
        if let Some(id) = self.npc_aliases.get(&key) {
            return self.npcs.get(id);
        }
        
        // 再嘗試通過名稱查找
        self.npcs.values().find(|npc| npc.name.to_lowercase() == key)
    }

    /// 通過 ID 或別名獲取可變 NPC
    pub fn get_npc_mut(&mut self, name_or_id: &str) -> Option<&mut Person> {
        let key = name_or_id.to_lowercase();
        
        // 先嘗試通過別名查找 ID
        let id = if let Some(id) = self.npc_aliases.get(&key) {
            id.clone()
        } else {
            // 嘗試通過名稱查找
            if let Some((id, _)) = self.npcs.iter().find(|(_, npc)| npc.name.to_lowercase() == key) {
                id.clone()
            } else {
                return None;
            }
        };
        
        self.npcs.get_mut(&id)
    }

    /// 獲取所有 NPC
    #[allow(dead_code)]
    pub fn get_all_npcs(&self) -> Vec<&Person> {
        self.npcs.values().collect()
    }
    
    /// 獲取所有 NPC ID
    pub fn get_all_npc_ids(&self) -> Vec<String> {
        self.npcs.keys().cloned().collect()
    }

    /// 獲取指定位置的 NPC（不過濾地圖，保留供特殊用途）
    #[allow(dead_code)]
    pub fn get_npcs_at(&self, x: usize, y: usize) -> Vec<&Person> {
        self.npcs.values()
            .filter(|npc| npc.x == x && npc.y == y)
            .collect()
    }
    
    /// 獲取指定地圖和位置的 NPC
    pub fn get_npcs_at_in_map(&self, map_name: &str, x: usize, y: usize) -> Vec<&Person> {
        self.npcs.values()
            .filter(|npc| npc.map == map_name && npc.x == x && npc.y == y)
            .collect()
    }
    
    /// 獲取指定地圖和位置的 NPC（排除指定 ID）
    pub fn get_npcs_at_in_map_excluding(&self, map_name: &str, x: usize, y: usize, exclude_id: &str) -> Vec<&Person> {
        self.npcs.iter()
            .filter(|(id, npc)| {
                npc.map == map_name && npc.x == x && npc.y == y && id.as_str() != exclude_id
            })
            .map(|(_, npc)| npc)
            .collect()
    }
    
    /// 獲取指定地圖和位置的 NPC，返回 (id, npc) 元組
    pub fn get_npcs_with_ids_at_in_map(&self, map_name: &str, x: usize, y: usize) -> Vec<(String, &Person)> {
        self.npcs.iter()
            .filter(|(_, npc)| npc.map == map_name && npc.x == x && npc.y == y)
            .map(|(id, npc)| (id.clone(), npc))
            .collect()
    }

    /// 移除 NPC
    #[allow(dead_code)]
    pub fn remove_npc(&mut self, id: &str) -> Option<Person> {
        self.npcs.remove(id)
    }

    /// 通過名稱或別名和位置移除 NPC
    pub fn remove_npc_at(&mut self, name_or_id: &str, x: usize, y: usize) -> Option<(String, Person)> {
        let key = name_or_id.to_lowercase();
        
        // 先嘗試通過別名查找 ID
        if let Some(id) = self.npc_aliases.get(&key) {
            if let Some(npc) = self.npcs.get(id) {
                if npc.x == x && npc.y == y {
                    let id_clone = id.clone();
                    let removed_npc = self.npcs.remove(&id_clone);
                    return removed_npc.map(|npc| (id_clone, npc));
                }
            }
        }
        
        // 嘗試通過名稱查找
        if let Some((id, _npc)) = self.npcs.iter().find(|(_, npc)| {
            npc.name.to_lowercase() == key && npc.x == x && npc.y == y
        }) {
            let id_clone = id.clone();
            let removed_npc = self.npcs.remove(&id_clone);
            return removed_npc.map(|npc| (id_clone, npc));
        }
        
        None
    }
    


    /// 保存所有 NPC
    pub fn save_all(&self, person_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        for (id, npc) in &self.npcs {
            npc.save(person_dir, id)?;
        }
        Ok(())
    }

    /// 載入 NPC
    #[allow(dead_code)]
    pub fn load_npc(&mut self, id: String, person_dir: &str, aliases: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let npc = Person::load(person_dir, &id)?;
        self.add_npc(id, npc, aliases);
        Ok(())
    }

    /// 獲取 NPC 數量
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.npcs.len()
    }
    
    /// 更新所有 NPC 的時間
    pub fn update_all_time(&mut self, time_info: &crate::time_updatable::TimeInfo) {
        use crate::time_updatable::TimeUpdatable;
        for npc in self.npcs.values_mut() {
            npc.on_time_update(time_info);
        }
    }

    /// 根據名稱獲取 NPC 的顯示字符
    pub fn get_display_char(name: &str) -> char {
        match name {
            "商人" => 'M',
            "農夫" => 'F',
            "醫生" => 'D',
            "工人" => 'W',
            "路人" | "旅者" => 'T',
            "戰士" => 'R',     // Warrior
            "工程師" => 'E',   // Engineer
            "老師" => 'C',     // Teacher (Coach)
            _ => 'N',  // 預設為 N (NPC)
        }
    }
    
    /// 從目錄載入所有 NPC（包含 "me"）
    pub fn load_all_from_directory(&mut self, person_dir: &str, skip_files: Vec<&str>) -> Result<usize, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(person_dir)?;
        let mut loaded_count = 0;
        
        if let Ok(entries) = std::fs::read_dir(person_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        // 跳過指定的文件
                        if skip_files.contains(&file_stem) {
                            continue;
                        }
                        
                        // 嘗試載入 NPC
                        if let Ok(mut npc) = Person::load(person_dir, file_stem) {
                            // 確保 NPC 有預設的 10000 金幣（me 除外）
                            if file_stem != "me" && (!npc.items.contains_key("金幣") || npc.items.get("金幣").copied().unwrap_or(0) == 0) {
                                npc.items.insert("金幣".to_string(), 10_000);
                            }
                            
                            // 使用文件名作為 ID，名稱作為別名
                            self.add_npc(
                                file_stem.to_string(), 
                                npc.clone(), 
                                vec![npc.name.to_lowercase()]
                            );
                            loaded_count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    /// 獲取 "me" 的不可變引用
    pub fn get_me(&self) -> Option<&Person> {
        self.npcs.get("me")
    }
    
    /// 初始化 NpcManager：載入所有 NPC 並確保 me 存在
    /// 返回 (loaded_count, me)
    pub fn initialize(&mut self, person_dir: &str) -> Result<(usize, Person), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(person_dir)?;
        
        // 載入所有 NPC
        let loaded_count = self.load_all_from_directory(person_dir, vec![])?;
        
        // 確保 me 存在
        let me = self.ensure_me(person_dir)?;
        
        Ok((loaded_count, me))
    }
    
    /// 確保 "me" 存在，如果不存在則創建預設的 me
    /// 返回 me 的 clone
    fn ensure_me(&mut self, person_dir: &str) -> Result<Person, Box<dyn std::error::Error>> {
        if let Some(me) = self.get_me() {
            // me 已存在，直接返回
            Ok(me.clone())
        } else {
            // 創建預設的 me
            let mut new_me = Person::new(
                "創造者".to_string(),
                "創造世界的創造者，探索未知的世界".to_string(),
            );
            new_me.add_ability("劍術".to_string());
            new_me.add_ability("魔法".to_string());
            new_me.add_ability("探險".to_string());
            new_me.add_item("木劍".to_string());
            new_me.add_item("魔法書".to_string());
            new_me.add_item("治療藥水".to_string());
            new_me.set_status("精力充沛".to_string());
            
            // 保存到文件
            new_me.save(person_dir, "me")?;
            
            // 添加到 npc_manager
            self.add_npc("me".to_string(), new_me.clone(), vec!["創造者".to_string()]);
            
            Ok(new_me)
        }
    }
    
    /// 計算兩個位置的曼哈頓距離
    fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
        ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as usize
    }
    
    /// 更新 NPC 距離並返回靠近/離開的通知
    /// current_controlled_id: 當前操控角色的 ID（"me" 或其他 NPC ID）
    /// player_just_moved: 是否是玩家剛執行移動指令（用於區分訊息文字）
    /// 返回 Vec<(npc_id, message, should_greet)> - should_greet 表示是否應該說見面語
    pub fn update_proximity(&mut self, current_controlled_id: &str, current_x: usize, current_y: usize, current_map: &str, player_just_moved: bool) -> Vec<(String, String, bool)> {
        let mut notifications = Vec::new();
        
        for (npc_id, npc) in &self.npcs {
            // 跳過當前操控的角色自己
            if npc_id == current_controlled_id {
                continue;
            }
            
            // 只檢測同地圖的 NPC
            if npc.map != current_map {
                continue;
            }
            
            let current_distance = Self::manhattan_distance(npc.x, npc.y, current_x, current_y);
            
            // 為每個 NPC 使用獨立的 distance key（基於當前操控角色）
            let distance_key = format!("{current_controlled_id}_{npc_id}");
            let previous_distance = self.previous_distances.get(&distance_key).copied();
            
            // 檢測靠近（從 1 → 0）
            if previous_distance == Some(1) && current_distance == 0 {
                let message = if player_just_moved {
                    // 玩家移動到 NPC 位置
                    format!("你看到這裡有 {}", npc.name)
                } else {
                    // NPC 移動到玩家位置
                    format!("{} 往這邊走來", npc.name)
                };
                notifications.push((npc_id.clone(), message, true));
            }
            // 檢測離開（從 0 → 1）- 只在 NPC 主動離開時顯示
            else if previous_distance == Some(0) && current_distance == 1 && !player_just_moved {
                // 只有 NPC 移動離開時才通知，玩家主動離開不通知
                notifications.push((npc_id.clone(), format!("{} 離開了", npc.name), false));
            }
            
            // 更新距離記錄
            self.previous_distances.insert(distance_key, current_distance);
        }
        
        notifications
    }
}
