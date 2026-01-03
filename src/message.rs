use serde::{Deserialize, Serialize};

/// 輸出訊息（GameWorld → OutputManager）
/// 這是 GameWorld 處理事件後產生的輸出
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    /// NPC 說話
    NpcSay {
        npc_id: String,
        npc_name: String,
        text: String,
    },
    
    /// 系統訊息
    System(String),
    
    /// 戰鬥訊息
    Combat {
        attacker: String,
        target: String,
        damage: i32,
    },
    
    /// 物品撿起
    ItemPickup {
        entity: String,
        item: String,
        count: u32,
    },
    
    /// 物品使用
    ItemUse {
        entity: String,
        item: String,
        effect: String,
    },
    
    /// 移動訊息
    Movement {
        entity: String,
        from: (usize, usize),
        to: (usize, usize),
    },
    
    /// 錯誤訊息
    Error(String),
    
    /// 日誌訊息（系統內部）
    Log(String),
}

impl Message {
    /// 轉換為顯示文字
    pub fn to_display_text(&self) -> String {
        match self {
            Message::NpcSay { npc_name, text, .. } => {
                format!("💬 {npc_name} 說：「{text}」")
            },
            Message::System(text) => text.clone(),
            Message::Combat { attacker, target, damage } => {
                format!("⚔️  {attacker} 攻擊 {target}，造成 {damage} 點傷害")
            },
            Message::ItemPickup { entity, item, count } => {
                format!("📦 {entity} 撿起了 {item} x{count}")
            },
            Message::ItemUse { entity, item, effect } => {
                format!("✨ {entity} 使用了 {item}，{effect}")
            },
            Message::Movement { entity, to, .. } => {
                format!("🚶 {} 移動到 ({}, {})", entity, to.0, to.1)
            },
            Message::Error(text) => format!("❌ {text}"),
            Message::Log(text) => text.clone(),
        }
    }
    
    /// 是否為日誌訊息（不顯示在主輸出）
    pub fn is_log(&self) -> bool {
        matches!(self, Message::Log(_) | Message::Movement { .. })
    }
}
