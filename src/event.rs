use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 事件觸發器類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum TriggerType {
    #[serde(rename = "time")]
    Time {
        schedule: String,  // Crontab 格式: "分 時 日 月 星期"
        #[serde(skip_serializing_if = "Option::is_none")]
        random_chance: Option<f32>,  // 0.0-1.0
        #[serde(skip_serializing_if = "Option::is_none")]
        day_range: Option<[u32; 2]>,  // [開始天, 結束天]
        #[serde(skip_serializing_if = "Option::is_none")]
        time_range: Option<[String; 2]>,  // ["HH:MM:SS", "HH:MM:SS"]
    },
    #[serde(rename = "random")]
    Random {
        interval_seconds: u64,  // 檢查間隔
        chance: f32,  // 觸發機率 0.0-1.0
    },
    #[serde(rename = "location")]
    Location {
        positions: Vec<[usize; 2]>,  // [[x, y], ...]
    },
    #[serde(rename = "condition")]
    Condition {
        conditions: Vec<String>,  // 條件表達式
    },
    #[serde(rename = "manual")]
    Manual,
}

/// 人物條件（Who）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhoCondition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npcs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_level: Option<LevelRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
}

/// 位置類型（支援固定位置或隨機位置）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Position {
    Fixed([usize; 2]),  // 固定位置 [x, y]
    Random(String),     // "random" 表示隨機位置
}

impl Position {
    /// 解析位置，如果是隨機則從地圖中選擇一個可行走的位置
    pub fn resolve(&self, map: &crate::map::Map) -> Option<[usize; 2]> {
        match self {
            Position::Fixed(pos) => Some(*pos),
            Position::Random(_) => {
                let walkable_points = map.get_walkable_points();
                if walkable_points.is_empty() {
                    None
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let idx = rng.gen_range(0..walkable_points.len());
                    Some([walkable_points[idx].0, walkable_points[idx].1])
                }
            }
        }
    }
}

/// 地點條件（Where）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhereCondition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<[usize; 2]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<AreaRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaRange {
    pub x: [usize; 2],  // [min, max]
    pub y: [usize; 2],
}

/// 物品條件（What）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhatCondition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_items: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_objects: Option<Vec<String>>,
}

/// 事件動作類型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventAction {
    #[serde(rename = "spawn_npc")]
    SpawnNpc {
        npc_id: String,
        position: Position,
        #[serde(skip_serializing_if = "Option::is_none")]
        dialogue: Option<String>,
    },
    #[serde(rename = "remove_npc")]
    RemoveNpc {
        npc_id: String,
    },
    #[serde(rename = "message")]
    Message {
        text: String,
    },
    #[serde(rename = "dialogue")]
    Dialogue {
        npc_id: String,
        text: String,
    },
    #[serde(rename = "add_item")]
    AddItem {
        item: String,
        position: Position,
    },
    #[serde(rename = "remove_item")]
    RemoveItem {
        item: String,
        position: Position,
    },
    #[serde(rename = "teleport")]
    Teleport {
        map: String,
        position: Position,
    },
}

/// 事件狀態設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventState {
    #[serde(default = "default_true")]
    pub repeatable: bool,
    #[serde(default)]
    pub cooldown: u64,  // 秒
    #[serde(default = "default_neg_one")]
    pub max_triggers: i32,  // -1 = 無限
    #[serde(default)]
    pub prerequisites: Vec<String>,
}

fn default_true() -> bool { true }
fn default_neg_one() -> i32 { -1 }

impl Default for EventState {
    fn default() -> Self {
        EventState {
            repeatable: true,
            cooldown: 0,
            max_triggers: -1,
            prerequisites: Vec::new(),
        }
    }
}

/// 事件腳本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TriggerType,
    
    #[serde(default)]
    pub who: WhoCondition,
    #[serde(default)]
    pub r#where: WhereCondition,  // where 是 Rust 關鍵字，使用 r#where
    #[serde(default)]
    pub what: WhatCondition,
    
    pub actions: Vec<EventAction>,
    
    #[serde(default)]
    pub state: EventState,
}

impl GameEvent {
    /// 檢查事件是否可以觸發
    pub fn can_trigger(&self, runtime_state: &EventRuntimeState) -> bool {
        // 檢查冷卻時間
        if let Some(last_trigger) = runtime_state.last_trigger_time {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - last_trigger;
            
            if elapsed < self.state.cooldown {
                return false;
            }
        }
        
        // 檢查觸發次數
        if self.state.max_triggers >= 0 && runtime_state.trigger_count >= self.state.max_triggers as u32 {
            return false;
        }
        
        // 檢查是否可重複
        if !self.state.repeatable && runtime_state.trigger_count > 0 {
            return false;
        }
        
        true
    }
}

/// 事件運行時狀態
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventRuntimeState {
    pub trigger_count: u32,
    pub last_trigger_time: Option<u64>,  // Unix timestamp
    pub completed_prerequisites: Vec<String>,
}

impl EventRuntimeState {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_trigger(&mut self) {
        self.trigger_count += 1;
        self.last_trigger_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }
}

/// 事件管理器
pub struct EventManager {
    events: HashMap<String, GameEvent>,
    runtime_states: HashMap<String, EventRuntimeState>,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            events: HashMap::new(),
            runtime_states: HashMap::new(),
        }
    }
    
    pub fn add_event(&mut self, event: GameEvent) {
        let id = event.id.clone();
        self.events.insert(id.clone(), event);
        self.runtime_states.entry(id).or_default();
    }
    
    pub fn get_event(&self, id: &str) -> Option<&GameEvent> {
        self.events.get(id)
    }
    
    pub fn get_runtime_state(&self, id: &str) -> Option<&EventRuntimeState> {
        self.runtime_states.get(id)
    }
    
    #[allow(dead_code)]
    pub fn get_runtime_state_mut(&mut self, id: &str) -> Option<&mut EventRuntimeState> {
        self.runtime_states.get_mut(id)
    }
    
    pub fn list_events(&self) -> Vec<&GameEvent> {
        self.events.values().collect()
    }
    
    /// 觸發事件
    pub fn trigger_event(&mut self, event_id: &str) -> bool {
        if let Some(state) = self.runtime_states.get_mut(event_id) {
            state.record_trigger();
            true
        } else {
            false
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}
