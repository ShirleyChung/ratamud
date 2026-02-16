use crate::event::{EventAction, GameEvent};
use crate::world::GameWorld;

/// Output trait for event executor (works with both UI and non-UI modes)
pub trait EventOutput {
    fn print(&mut self, message: String);
}

#[cfg(feature = "terminal-ui")]
impl EventOutput for crate::output::OutputManager {
    fn print(&mut self, message: String) {
        self.print(message);
    }
}

impl EventOutput for crate::core_output::CoreOutputManager {
    fn print(&mut self, message: String) {
        self.add_message(message);
    }
}

/// 事件執行器
pub struct EventExecutor;

impl EventExecutor {
    /// 執行事件的所有動作
    /// 從 game_world.npc_manager 獲取當前控制的角色
    pub fn execute_event<O: EventOutput>(
        event: &GameEvent,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        output.print(format!("🎭 事件觸發: {}", event.name));
        
        for action in &event.actions {
            if let Err(e) = Self::execute_action(action, game_world, output) {
                return Err(format!("執行動作失敗: {e}"));
            }
        }
        
        Ok(())
    }
    
    /// 執行單個動作
    fn execute_action<O: EventOutput>(
        action: &EventAction,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        match action {
            EventAction::SpawnNpc { npc_id, position, dialogue } => {
                Self::spawn_npc(npc_id, position, dialogue.as_deref(), game_world, output)
            }
            EventAction::RemoveNpc { npc_id } => {
                Self::remove_npc(npc_id, output)
            }
            EventAction::Message { text } => {
                Self::show_message(text, output)
            }
            EventAction::Dialogue { npc_id, text } => {
                Self::show_dialogue(npc_id, text, output)
            }
            EventAction::AddItem { item, position } => {
                Self::add_item(item, position, game_world, output)
            }
            EventAction::RemoveItem { item, position } => {
                Self::remove_item(item, position, game_world, output)
            }
            EventAction::Teleport { map, position } => {
                Self::teleport_player(map, position, game_world, output)
            }
            EventAction::SetMapProperty { map, property, value } => {
                Self::set_map_property(map, property, value, game_world, output)
            }
            EventAction::RandomAction { actions } => {
                Self::execute_random_action(actions, game_world, output)
            }
        }
    }
    
    fn spawn_npc<O: EventOutput>(
        npc_id: &str,
        position: &crate::event::Position,
        dialogue: Option<&str>,
        game_world: &GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        output.print(format!(
            "👤 NPC {} 出現在 ({}, {})",
            npc_id, resolved_pos[0], resolved_pos[1]
        ));
        
        if let Some(text) = dialogue {
            output.print(format!("💬 {npc_id}: \"{text}\""));
        }
        
        // TODO: 實際生成 NPC 到遊戲世界
        Ok(())
    }
    
    fn remove_npc<O: EventOutput>(npc_id: &str, output: &mut O) -> Result<(), String> {
        output.print(format!("👤 NPC {npc_id} 離開了"));
        // TODO: 從遊戲世界移除 NPC
        Ok(())
    }
    
    fn show_message<O: EventOutput>(text: &str, output: &mut O) -> Result<(), String> {
        output.print(format!("📢 {text}"));
        Ok(())
    }
    
    fn show_dialogue<O: EventOutput>(
        npc_id: &str,
        text: &str,
        output: &mut O,
    ) -> Result<(), String> {
        output.print(format!("💬 {npc_id}: \"{text}\""));
        Ok(())
    }
    
    fn add_item<O: EventOutput>(
        item: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(resolved_pos[0], resolved_pos[1]) {
                point.add_object(item.to_string());
                output.print(format!(
                    "🎁 {} 出現在 ({}, {})",
                    item, resolved_pos[0], resolved_pos[1]
                ));
                return Ok(());
            }
        }
        Err(format!("無法在位置 ({}, {}) 添加物品", resolved_pos[0], resolved_pos[1]))
    }
    
    fn remove_item<O: EventOutput>(
        item: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(resolved_pos[0], resolved_pos[1]) {
                if point.remove_object(item) {
                    output.print(format!(
                        "🗑️  {} 從 ({}, {}) 消失了",
                        item, resolved_pos[0], resolved_pos[1]
                    ));
                    return Ok(());
                }
            }
        }
        Err(format!("無法在位置 ({}, {}) 移除物品 {}", resolved_pos[0], resolved_pos[1], item))
    }
    
    /// 傳送玩家到指定地圖位置
    /// 從 game_world.npc_manager 獲取當前控制的角色
    fn teleport_player<O: EventOutput>(
        map: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        if game_world.change_map(map) {
            let current_map = game_world.get_current_map()
                .ok_or("無法獲取目標地圖")?;
            
            let resolved_pos = position.resolve(current_map)
                .ok_or("無法解析目標位置")?;
            
            // 從 NpcManager 獲取當前控制的角色並傳送
            if let Some(me) = game_world.npc_manager.get_npc_mut(&game_world.current_controlled_id) {
                me.move_to(resolved_pos[0], resolved_pos[1]);
                output.print(format!(
                    "✨ 你被傳送到 {} ({}, {})",
                    map, resolved_pos[0], resolved_pos[1]
                ));
                Ok(())
            } else {
                Err("無法獲取當前角色".to_string())
            }
        } else {
            Err(format!("地圖 {map} 不存在"))
        }
    }

    fn set_map_property<O: EventOutput>(
        map_name: &str,
        property: &str,
        value: &str,
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        if let Some(map) = game_world.maps.get_mut(map_name) {
            map.set_property(property.to_string(), value.to_string());
            output.print(format!(
                "🗺️  {map_name}現在{property}是{value}",
            ));
            Ok(())
        } else {
            Err(format!("地圖 {map_name} 不存在"))
        }
    }

    /// 執行隨機動作
    fn execute_random_action<O: EventOutput>(
        weighted_actions: &[crate::event::WeightedAction],
        game_world: &mut GameWorld,
        output: &mut O,
    ) -> Result<(), String> {
        if weighted_actions.is_empty() {
            return Err("沒有可執行的隨機動作".to_string());
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // 計算總權重
        let total_weight: f32 = weighted_actions.iter().map(|a| a.weight).sum();
        
        if total_weight <= 0.0 {
            return Err("總權重必須大於0".to_string());
        }

        // 隨機選擇一個動作
        let mut random_value = rng.gen::<f32>() * total_weight;
        
        for weighted_action in weighted_actions {
            random_value -= weighted_action.weight;
            if random_value <= 0.0 {
                return Self::execute_action(&weighted_action.action, game_world, output);
            }
        }

        // 如果沒有選中任何動作（理論上不應該發生），執行最後一個
        if let Some(last) = weighted_actions.last() {
            Self::execute_action(&last.action, game_world, output)
        } else {
            Err("無法選擇隨機動作".to_string())
        }
    }
}

