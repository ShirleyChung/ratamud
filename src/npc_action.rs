use serde::{Deserialize, Serialize};

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
            NpcAction::Idle => "閒置".to_string(),
        }
    }
}
