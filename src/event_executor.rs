use crate::event::{EventAction, GameEvent};
use crate::world::GameWorld;
use crate::output::OutputManager;

/// 事件執行器
pub struct EventExecutor;

impl EventExecutor {
    /// 執行事件的所有動作
    pub fn execute_event(
        event: &GameEvent,
        game_world: &mut GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        output_manager.print(format!("🎭 事件觸發: {}", event.name));
        
        for action in &event.actions {
            if let Err(e) = Self::execute_action(action, game_world, output_manager) {
                return Err(format!("執行動作失敗: {e}"));
            }
        }
        
        Ok(())
    }
    
    /// 執行單個動作
    fn execute_action(
        action: &EventAction,
        game_world: &mut GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        match action {
            EventAction::SpawnNpc { npc_id, position, dialogue } => {
                Self::spawn_npc(npc_id, position, dialogue.as_deref(), game_world, output_manager)
            }
            EventAction::RemoveNpc { npc_id } => {
                Self::remove_npc(npc_id, output_manager)
            }
            EventAction::Message { text } => {
                Self::show_message(text, output_manager)
            }
            EventAction::Dialogue { npc_id, text } => {
                Self::show_dialogue(npc_id, text, output_manager)
            }
            EventAction::AddItem { item, position } => {
                Self::add_item(item, position, game_world, output_manager)
            }
            EventAction::RemoveItem { item, position } => {
                Self::remove_item(item, position, game_world, output_manager)
            }
            EventAction::Teleport { map, position } => {
                Self::teleport_player(map, position, game_world, output_manager)
            }
        }
    }
    
    fn spawn_npc(
        npc_id: &str,
        position: &crate::event::Position,
        dialogue: Option<&str>,
        game_world: &GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        output_manager.print(format!(
            "👤 NPC {} 出現在 ({}, {})",
            npc_id, resolved_pos[0], resolved_pos[1]
        ));
        
        if let Some(text) = dialogue {
            output_manager.print(format!("💬 {npc_id}: \"{text}\""));
        }
        
        // TODO: 實際生成 NPC 到遊戲世界
        Ok(())
    }
    
    fn remove_npc(npc_id: &str, output_manager: &mut OutputManager) -> Result<(), String> {
        output_manager.print(format!("👤 NPC {npc_id} 離開了"));
        // TODO: 從遊戲世界移除 NPC
        Ok(())
    }
    
    fn show_message(text: &str, output_manager: &mut OutputManager) -> Result<(), String> {
        output_manager.print(format!("📢 {text}"));
        Ok(())
    }
    
    fn show_dialogue(
        npc_id: &str,
        text: &str,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        output_manager.print(format!("💬 {npc_id}: \"{text}\""));
        Ok(())
    }
    
    fn add_item(
        item: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(resolved_pos[0], resolved_pos[1]) {
                point.add_object(item.to_string());
                output_manager.print(format!(
                    "🎁 {} 出現在 ({}, {})",
                    item, resolved_pos[0], resolved_pos[1]
                ));
                return Ok(());
            }
        }
        Err(format!("無法在位置 ({}, {}) 添加物品", resolved_pos[0], resolved_pos[1]))
    }
    
    fn remove_item(
        item: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        let current_map = game_world.get_current_map()
            .ok_or("無法獲取當前地圖")?;
        
        let resolved_pos = position.resolve(current_map)
            .ok_or("無法解析位置")?;
        
        if let Some(current_map) = game_world.get_current_map_mut() {
            if let Some(point) = current_map.get_point_mut(resolved_pos[0], resolved_pos[1]) {
                if point.remove_object(item) {
                    output_manager.print(format!(
                        "🗑️  {} 從 ({}, {}) 消失了",
                        item, resolved_pos[0], resolved_pos[1]
                    ));
                    return Ok(());
                }
            }
        }
        Err(format!("無法在位置 ({}, {}) 移除物品 {}", resolved_pos[0], resolved_pos[1], item))
    }
    
    fn teleport_player(
        map: &str,
        position: &crate::event::Position,
        game_world: &mut GameWorld,
        output_manager: &mut OutputManager,
    ) -> Result<(), String> {
        if game_world.change_map(map) {
            let current_map = game_world.get_current_map()
                .ok_or("無法獲取目標地圖")?;
            
            let resolved_pos = position.resolve(current_map)
                .ok_or("無法解析目標位置")?;
            
            game_world.player.move_to(resolved_pos[0], resolved_pos[1]);
            output_manager.print(format!(
                "✨ 你被傳送到 {} ({}, {})",
                map, resolved_pos[0], resolved_pos[1]
            ));
            Ok(())
        } else {
            Err(format!("地圖 {map} 不存在"))
        }
    }
}
