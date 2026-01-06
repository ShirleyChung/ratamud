use serde::{Deserialize, Serialize};

/// 位置信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// 遊戲時間信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub day: u32,
}

/// 實體類型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Player,
    Npc,
    Item,
}

/// 實體信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityInfo {
    pub entity_type: EntityType,
    pub id: String,
    pub pos: Position,
    pub name: String,
}

/// 物品信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemInfo {
    pub item_name: String,
    pub count: u32,
    pub pos: Position,
}

/// 地形信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainInfo {
    pub walkable: bool,
    pub description: String,
}

/// NPC 可見的世界快照（不可變）
/// 這是 NPC AI 決策的唯一輸入
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcView {
    /// NPC 自身 ID
    pub self_id: String,
    
    /// NPC 當前位置
    pub self_pos: Position,
    
    /// NPC 當前生命值
    pub self_hp: i32,
    
    /// NPC 最大生命值
    pub self_max_hp: i32,
    
    /// NPC 當前魔力值
    pub self_mp: i32,
    
    /// NPC 背包物品
    pub self_items: Vec<(String, u32)>,
    
    /// NPC 所在地圖
    pub current_map: String,
    
    /// 遊戲時間
    pub time: GameTime,
    
    /// 附近的實體（玩家、其他 NPC）
    pub nearby_entities: Vec<EntityInfo>,
    
    /// 可見的物品
    pub visible_items: Vec<ItemInfo>,
    
    /// 當前位置的地形信息
    pub terrain: TerrainInfo,
    
    /// NPC 是否正在互動中
    pub is_interacting: bool,
    
    /// NPC 是否在隊伍中
    pub in_party: bool,
}

impl NpcView {
    /// 創建一個空的 NpcView（用於測試）
    #[allow(dead_code)]
    pub fn empty(npc_id: String) -> Self {
        NpcView {
            self_id: npc_id,
            self_pos: Position { x: 0, y: 0 },
            self_hp: 100,
            self_max_hp: 100,
            self_mp: 100,
            self_items: Vec::new(),
            current_map: String::new(),
            time: GameTime {
                hour: 0,
                minute: 0,
                second: 0,
                day: 1,
            },
            nearby_entities: Vec::new(),
            visible_items: Vec::new(),
            terrain: TerrainInfo {
                walkable: true,
                description: "未知區域".to_string(),
            },
            is_interacting: false,
            in_party: false,
        }
    }
}
