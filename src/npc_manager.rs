use crate::person::Person;
use std::collections::HashMap;

/// NPC 管理器，負責管理遊戲中的所有 NPC
#[derive(Clone)]
pub struct NpcManager {
    npcs: HashMap<String, Person>,  // NPC ID -> Person
    npc_aliases: HashMap<String, String>,  // 別名 -> NPC ID
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
        }
    }

    /// 添加 NPC
    pub fn add_npc(&mut self, id: String, npc: Person, aliases: Vec<String>) {
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

    /// 獲取指定位置的 NPC
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
}
