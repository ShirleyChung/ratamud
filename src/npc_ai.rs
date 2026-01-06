use rand::Rng;

/// NPC AI 控制器
pub struct NpcAiController;

impl NpcAiController {
    /// 根據 NpcView 決定 NPC 的行為（新架構）
    /// 這個方法只返回意圖，不修改任何狀態
    pub fn decide_action(npc_view: &crate::npc_view::NpcView) -> Option<crate::npc_action::NpcAction> {
        use crate::npc_action::{NpcAction, Direction};
        
        // 如果 NPC 正在互動中，返回 Idle
        if npc_view.is_interacting {
            return Some(NpcAction::Idle);
        }
        
        // 如果 NPC 在戰鬥中，使用戰鬥技能
        if npc_view.in_combat {
            // 隨機選擇戰鬥技能 (punch 或 kick)
            let mut rng = rand::thread_rng();
            let skills = ["punch", "kick"];
            let skill_name = skills[rng.gen_range(0..skills.len())];
            
            // 目標是玩家 "me"
            return Some(NpcAction::UseCombatSkill {
                skill_name: skill_name.to_string(),
                target_id: "me".to_string(),
            });
        }
        
        // 如果 NPC 在隊伍中，不隨意移動
        if npc_view.in_party {
            return Some(NpcAction::Idle);
        }
        
        // 優先檢查生命值，需要使用食物（HP < max_hp / 2）
        if npc_view.self_hp < npc_view.self_max_hp / 2 {
            // 尋找食物
            let food_items = ["蘋果", "乾肉", "麵包"];
            
            for food in &food_items {
                if npc_view.self_items.iter().any(|(name, count)| name == food && *count > 0) {
                    return Some(NpcAction::UseItem(food.to_string()));
                }
            }
        }
        
        // 隨機行為
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..100);
        
        if roll < 20 && !npc_view.visible_items.is_empty() {
            // 20% 機率撿起物品（如果當前位置有物品）
            let item = &npc_view.visible_items[0];
            Some(NpcAction::PickupItem {
                item_name: item.item_name.clone(),
                quantity: 1,
            })
        } else if roll < 30 {
            // 10% 機率隨機移動
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            let direction = directions[rng.gen_range(0..directions.len())].clone();
            return Some(NpcAction::Move(direction));
        } else {
            // 50% 機率閒置
            return Some(NpcAction::Idle);
        }
    }
}

/// Default 實現
impl Default for crate::map::TerrainType {
    fn default() -> Self {
        crate::map::TerrainType::Normal
    }
}

