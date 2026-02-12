use crate::npc_action::NpcAction;
use serde::{Deserialize, Serialize};

/// 輸入事件
#[derive(Clone, Debug)]
pub enum InputEvent {
    /// 命令字串
    Command(String),
    
    /// 終端 UI 鍵盤按鍵（僅在啟用 terminal-ui 功能時可用）
    #[cfg(feature = "terminal-ui")]
    KeyPress(crossterm::event::KeyEvent),
}

/// 遊戲事件（跨執行緒通訊）
/// 所有執行緒與 GameWorld 的通訊都通過這個事件系統
#[derive(Clone, Debug)]
pub enum GameEvent {
    /// NPC 行為事件
    NpcActions {
        npc_id: String,
        actions: Vec<NpcAction>,
    },
    
    /// 時間更新事件
    TimerTick {
        elapsed_secs: u64,
    },
    
    /// 輸入事件
    Input(InputEvent),
}

/// 可序列化的遊戲事件（用於保存/回放）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerializableGameEvent {
    NpcActions {
        npc_id: String,
        actions: Vec<NpcAction>,
    },
    
    TimerTick {
        elapsed_secs: u64,
    },
    
    Command(String),
}

impl From<&GameEvent> for Option<SerializableGameEvent> {
    fn from(event: &GameEvent) -> Self {
        match event {
            GameEvent::NpcActions { npc_id, actions } => {
                Some(SerializableGameEvent::NpcActions {
                    npc_id: npc_id.clone(),
                    actions: actions.clone(),
                })
            },
            GameEvent::TimerTick { elapsed_secs } => {
                Some(SerializableGameEvent::TimerTick {
                    elapsed_secs: *elapsed_secs,
                })
            },
            GameEvent::Input(InputEvent::Command(cmd)) => {
                Some(SerializableGameEvent::Command(cmd.clone()))
            },
            #[cfg(feature = "terminal-ui")]
            GameEvent::Input(InputEvent::KeyPress(_)) => None,
        }
    }
}
