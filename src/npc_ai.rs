use crate::person::Person;
use crate::world::GameWorld;
use crate::map::TerrainType;
use rand::Rng;

/// NPC 行為類型
#[derive(Clone, Debug, PartialEq)]
pub enum NpcBehavior {
    Idle,           // 閒置
    Wander,         // 漫遊
    PickupItems,    // 撿拾物品
    UseFood,        // 使用食物
    Farm,           // 耕作（農夫專屬）
    Trade,          // 交易（商人專屬）
}

/// NPC AI 控制器
pub struct NpcAiController;

#[allow(dead_code)]
impl NpcAiController {
    /// 執行所有 NPC 的 AI 行為
    pub fn update_all_npcs(game_world: &mut GameWorld) -> Vec<String> {
        let mut log_messages = Vec::new();
        
        // 獲取所有 NPC ID
        let npc_ids: Vec<String> = game_world.npc_manager.get_all_npc_ids();
        
        for npc_id in npc_ids {
            if let Some(msg) = Self::update_npc(game_world, &npc_id) {
                log_messages.push(msg);
            }
        }
        
        log_messages
    }
    
    /// 更新單個 NPC 的行為，返回日誌訊息
    fn update_npc(game_world: &mut GameWorld, npc_id: &str) -> Option<String> {
        // 獲取 NPC 副本以避免借用衝突
        let npc = match game_world.npc_manager.get_npc(npc_id) {
            Some(n) => n.clone(),
            None => return None,
        };
        
        // 根據 NPC 描述判斷類型和行為
        let behavior = Self::determine_behavior(&npc);
        
        match behavior {
            NpcBehavior::UseFood => {
                Self::try_use_food(game_world, npc_id, &npc)
            },
            NpcBehavior::PickupItems => {
                Self::try_pickup_items(game_world, npc_id, &npc)
            },
            NpcBehavior::Wander => {
                Self::try_wander(game_world, npc_id, &npc)
            },
            NpcBehavior::Farm => {
                Self::try_farm(game_world, npc_id, &npc)
            },
            NpcBehavior::Trade => {
                // 商人暫時不主動交易，等待玩家互動
                None
            },
            NpcBehavior::Idle => {
                // 閒置，不做任何事
                None
            },
        }
    }
    
    /// 判斷 NPC 應該執行的行為（公開方法供執行緒使用）
    pub fn determine_behavior(npc: &Person) -> NpcBehavior {
        let desc = npc.description.to_lowercase();
        
        // 優先檢查生命值，需要使用食物（HP < max_hp / 2）
        if npc.hp < npc.max_hp / 2 {
            return NpcBehavior::UseFood;
        }
        
        // 根據 NPC 類型決定行為
        if desc.contains("農") || desc.contains("farm") {
            // 農夫：耕作
            NpcBehavior::Farm
        } else if desc.contains("商") || desc.contains("merchant") || desc.contains("trader") {
            // 商人：交易（被動行為）
            NpcBehavior::Trade
        } else {
            // 其他 NPC：隨機行為
            let mut rng = rand::thread_rng();
            let roll = rng.gen_range(0..100);
            
            if roll < 30 {
                NpcBehavior::PickupItems
            } else if roll < 60 {
                NpcBehavior::Wander
            } else {
                NpcBehavior::Idle
            }
        }
    }
    
    /// 嘗試使用食物恢復 HP
    fn try_use_food(game_world: &mut GameWorld, npc_id: &str, npc: &Person) -> Option<String> {
        // 尋找食物
        let food_items = ["蘋果", "乾肉", "麵包"];
        
        for food in &food_items {
            if let Some(count) = npc.items.get(*food) {
                if *count > 0 {
                    // 使用食物
                    if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(npc_id) {
                        // 移除食物
                        if let Some(item_count) = npc_mut.items.get_mut(*food) {
                            *item_count -= 1;
                            if *item_count == 0 {
                                npc_mut.items.remove(*food);
                            }
                        }
                        
                        // 恢復 HP
                        let heal_amount = 20;
                        npc_mut.hp = (npc_mut.hp + heal_amount).min(npc_mut.max_hp);
                        
                        return Some(format!("{} 使用了 {} 恢復 HP", npc_mut.name, food));
                    }
                    return None;
                }
            }
        }
        None
    }
    
    /// 嘗試撿拾物品
    fn try_pickup_items(game_world: &mut GameWorld, npc_id: &str, npc: &Person) -> Option<String> {
        let map_name = game_world.current_map_name.clone();
        
        // 獲取當前位置的物品
        let items_at_pos: Vec<(String, u32)> = if let Some(map) = game_world.maps.get(&map_name) {
            if let Some(point) = map.get_point(npc.x, npc.y) {
                point.objects.iter().map(|(k, v)| (k.clone(), *v)).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        if items_at_pos.is_empty() {
            return None;
        }
        
        // 隨機選擇一個物品撿起
        let mut rng = rand::thread_rng();
        if let Some((item_name, _)) = items_at_pos.get(rng.gen_range(0..items_at_pos.len())) {
            let pickup_amount = 1;
            
            // 從地圖移除
            if let Some(map) = game_world.get_current_map_mut() {
                if let Some(point) = map.get_point_mut(npc.x, npc.y) {
                    if let Some(count) = point.objects.get_mut(item_name) {
                        if *count >= pickup_amount {
                            *count -= pickup_amount;
                            if *count == 0 {
                                point.objects.remove(item_name);
                            }
                            
                            // 添加到 NPC 背包
                            if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(npc_id) {
                                *npc_mut.items.entry(item_name.clone()).or_insert(0) += pickup_amount;
                                
                                return Some(format!("{} 撿起了 {}", npc_mut.name, item_name));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// 嘗試漫遊
    fn try_wander(game_world: &mut GameWorld, npc_id: &str, npc: &Person) -> Option<String> {
        let mut rng = rand::thread_rng();
        
        // 隨機選擇方向
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        let (dx, dy) = directions[rng.gen_range(0..directions.len())];
        
        let new_x = (npc.x as i32 + dx) as usize;
        let new_y = (npc.y as i32 + dy) as usize;
        
        // 檢查是否可行走
        let can_walk = if let Some(map) = game_world.get_current_map() {
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
            if let Some(npc_mut) = game_world.npc_manager.get_npc_mut(npc_id) {
                let npc_name = npc_mut.name.clone();
                npc_mut.move_to(new_x, new_y);
                return Some(format!("{npc_name} 移動到 ({new_x}, {new_y})"));
            }
        }
        None
    }
    
    /// 農夫耕作：將周圍 3x3 變成農地
    fn try_farm(game_world: &mut GameWorld, npc_id: &str, npc: &Person) -> Option<String> {
        let map_name = game_world.current_map_name.clone();
        let mut converted = false;
        
        // 檢查周圍 3x3 範圍
        for dy in -1..=1 {
            for dx in -1..=1 {
                let target_x = (npc.x as i32 + dx) as usize;
                let target_y = (npc.y as i32 + dy) as usize;
                
                if let Some(map) = game_world.get_current_map_mut() {
                    if target_x < map.width && target_y < map.height {
                        if let Some(point) = map.get_point_mut(target_x, target_y) {
                            // 如果是普通地形且可行走，轉換為農地
                            if point.terrain_type == TerrainType::Normal && point.walkable {
                                point.terrain_type = TerrainType::Farmland;
                                point.description = "肥沃的農地".to_string();
                                converted = true;
                            }
                        }
                    }
                }
            }
        }
        
        if converted {
            // 保存地圖
            if let Some(map) = game_world.maps.get(&map_name) {
                let _ = game_world.save_map(map);
            }
            
            if let Some(npc) = game_world.npc_manager.get_npc(npc_id) {
                return Some(format!("{} 正在開墾農地", npc.name));
            }
        }
        
        None
    }
}

/// Default 實現
impl Default for TerrainType {
    fn default() -> Self {
        TerrainType::Normal
    }
}

