use crate::npc_action::NpcAiStrategyComposer;

/// NPC AI 控制器
pub struct NpcAiController {
    strategy_composer: NpcAiStrategyComposer,
}

impl NpcAiController {
    /// 創建新的 NPC AI 控制器
    pub fn new() -> Self {
        Self {
            strategy_composer: NpcAiStrategyComposer::default(),
        }
    }
    
    /// 根據 NpcView 決定 NPC 的行為（使用 Strategy 模式）
    /// 這個方法只返回意圖，不修改任何狀態
    pub fn decide_action(&self, npc_view: &crate::npc_view::NpcView) -> Option<crate::npc_action::NpcAction> {
        self.strategy_composer.decide_action(npc_view)
    }
}

impl Default for NpcAiController {
    fn default() -> Self {
        Self::new()
    }
}

/// Default 實現
impl Default for crate::map::TerrainType {
    fn default() -> Self {
        crate::map::TerrainType::Normal
    }
}

