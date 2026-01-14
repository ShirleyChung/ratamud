use serde::{Deserialize, Serialize};
use crate::npc_view::NpcView;

/// 方向
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 轉換為座標偏移量
    pub fn to_delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
    
    /// 從座標偏移量創建方向
    #[allow(dead_code)]
    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(Direction::Up),
            (0, 1) => Some(Direction::Down),
            (-1, 0) => Some(Direction::Left),
            (1, 0) => Some(Direction::Right),
            _ => None,
        }
    }
}

/// NPC 意圖（不可變）
/// NPC AI 只能返回這些意圖，由 GameWorld 決定是否執行
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NpcAction {
    /// 說話
    Say(String),
    
    /// 移動
    Move(Direction),
    
    /// 撿起物品
    PickupItem {
        item_name: String,
        quantity: u32,
    },
    
    /// 使用物品
    UseItem(String),
    
    /// 放下物品
    DropItem {
        item_name: String,
        quantity: u32,
    },
    
    /// 交易請求
    Trade {
        target_id: String,
    },
    
    /// 攻擊
    Attack {
        target_id: String,
    },
    
    /// 使用戰鬥技能
    UseCombatSkill {
        skill_name: String,
        target_id: String,
    },
    
    /// 閒置（什麼都不做）
    Idle,
}

impl NpcAction {
    /// 獲取行為的描述（用於日誌）
    pub fn describe(&self) -> String {
        match self {
            NpcAction::Say(text) => format!("說: {text}"),
            NpcAction::Move(dir) => format!("移動: {dir:?}"),
            NpcAction::PickupItem { item_name, quantity } => {
                format!("撿起: {item_name} x{quantity}")
            },
            NpcAction::UseItem(item) => format!("使用: {item}"),
            NpcAction::DropItem { item_name, quantity } => {
                format!("放下: {item_name} x{quantity}")
            },
            NpcAction::Trade { target_id } => format!("交易請求: {target_id}"),
            NpcAction::Attack { target_id } => format!("攻擊: {target_id}"),
            NpcAction::UseCombatSkill { skill_name, target_id } => {
                format!("使用技能: {skill_name} -> {target_id}")
            },
            NpcAction::Idle => "閒置".to_string(),
        }
    }
}

/// NPC AI 策略特徵
pub trait NpcAiStrategy: Send + Sync {
    /// 根據 NpcView 決定 NPC 的行為
    /// 返回 None 表示此策略不處理，讓下一個策略處理
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction>;
    
    /// 策略的優先級（數字越小優先級越高）
    fn priority(&self) -> i32 {
        100
    }
}

/// 互動中策略
pub struct InteractingStrategy;

impl NpcAiStrategy for InteractingStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        if npc_view.is_interacting {
            Some(NpcAction::Idle)
        } else {
            None
        }
    }
    
    fn priority(&self) -> i32 {
        10
    }
}

/// 戰鬥策略
pub struct CombatStrategy;

impl NpcAiStrategy for CombatStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        if !npc_view.in_combat {
            return None;
        }
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let skills = ["punch", "kick"];
        let skill_name = skills[rng.gen_range(0..skills.len())];
        
        Some(NpcAction::UseCombatSkill {
            skill_name: skill_name.to_string(),
            target_id: "me".to_string(),
        })
    }
    
    fn priority(&self) -> i32 {
        20
    }
}

/// 隊伍策略
pub struct PartyStrategy;

impl NpcAiStrategy for PartyStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        if npc_view.in_party {
            Some(NpcAction::Idle)
        } else {
            None
        }
    }
    
    fn priority(&self) -> i32 {
        30
    }
}

/// 治療策略
pub struct HealingStrategy;

impl NpcAiStrategy for HealingStrategy {
    fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        if npc_view.self_hp >= npc_view.self_max_hp / 2 {
            return None;
        }
        
        let food_items = ["蘋果", "乾肉", "麵包"];
        
        for food in &food_items {
            if npc_view.self_items.iter().any(|(name, count)| name == food && *count > 0) {
                return Some(NpcAction::UseItem(food.to_string()));
            }
        }
        
        None
    }
    
    fn priority(&self) -> i32 {
        40
    }
}

/// 隨機行為策略
pub struct RandomBehaviorStrategy;

impl NpcAiStrategy for RandomBehaviorStrategy {
    fn decide_action(&self, _npc_view: &NpcView) -> Option<NpcAction> {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..100);
        
        if roll < 20 && !_npc_view.visible_items.is_empty() {
            let item = &_npc_view.visible_items[0];
            Some(NpcAction::PickupItem {
                item_name: item.item_name.clone(),
                quantity: 1,
            })
        } else if roll < 30 {
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            let direction = directions[rng.gen_range(0..directions.len())].clone();
            Some(NpcAction::Move(direction))
        } else {
            Some(NpcAction::Idle)
        }
    }
    
    fn priority(&self) -> i32 {
        1000
    }
}

/// AI 策略組合器
pub struct NpcAiStrategyComposer {
    strategies: Vec<Box<dyn NpcAiStrategy>>,
}

impl NpcAiStrategyComposer {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }
    
    pub fn add_strategy(mut self, strategy: Box<dyn NpcAiStrategy>) -> Self {
        self.strategies.push(strategy);
        self.strategies.sort_by_key(|s| s.priority());
        self
    }
    
    pub fn decide_action(&self, npc_view: &NpcView) -> Option<NpcAction> {
        for strategy in &self.strategies {
            if let Some(action) = strategy.decide_action(npc_view) {
                return Some(action);
            }
        }
        None
    }
}

impl Default for NpcAiStrategyComposer {
    fn default() -> Self {
        Self::new()
            .add_strategy(Box::new(InteractingStrategy))
            .add_strategy(Box::new(CombatStrategy))
            .add_strategy(Box::new(PartyStrategy))
            .add_strategy(Box::new(HealingStrategy))
            .add_strategy(Box::new(RandomBehaviorStrategy))
    }
}
