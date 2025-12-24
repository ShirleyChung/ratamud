use crate::input::CommandResult;

/// 純文本命令處理器（不依賴 Crossterm）
#[allow(dead_code)]
pub struct CommandProcessor;

#[allow(dead_code)]
impl CommandProcessor {
    pub fn new() -> Self {
        CommandProcessor
    }
    
    /// 解析文本命令字串，返回 CommandResult
    pub fn parse_command(&self, input: &str) -> CommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return CommandResult::Error("沒有輸入命令".to_string());
        }
        
        let cmd = parts[0].to_lowercase();
        
        match cmd.as_str() {
            // 退出命令
            "exit" | "quit" | "q" => CommandResult::Exit,
            
            // 幫助命令
            "help" | "h" | "?" => CommandResult::Help,
            
            // 方向移動命令
            "up" | "u" | "north" | "n" => CommandResult::Move(0, -1),
            "down" | "d" | "south" | "s" => CommandResult::Move(0, 1),
            "left" | "west" | "w" => CommandResult::Move(-1, 0),
            "right" | "r" | "east" | "e" => CommandResult::Move(1, 0),
            
            // 移動到指定位置
            "move" | "goto" | "go" => {
                if parts.len() >= 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        return CommandResult::Move(x, y);
                    }
                }
                CommandResult::Error("用法: move <x> <y>".to_string())
            }
            
            // 查看命令
            "look" | "l" => {
                let target = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                CommandResult::Look(target)
            }
            
            // 狀態命令
            "status" | "stat" | "i" | "info" => CommandResult::ShowStatus,
            
            // 地圖命令
            "map" | "showmap" | "m" => CommandResult::ShowMap,
            "minimap" | "mm" => CommandResult::ShowMinimap,
            "hidemap" => CommandResult::HideMinimap,
            
            // 日誌命令
            "log" | "showlog" => CommandResult::ShowLog,
            "hidelog" => CommandResult::HideLog,
            
            // 世界命令
            "world" | "showworld" => CommandResult::ShowWorld,
            
            // 清屏命令
            "clear" | "cls" => CommandResult::Clear,
            
            // 拾取物品
            "get" | "take" | "pickup" => {
                if parts.len() > 1 {
                    let item_name = parts[1..].join(" ");
                    let quantity = 1; // 預設數量
                    CommandResult::Get(Some(item_name), quantity)
                } else {
                    CommandResult::Get(None, 1)
                }
            }
            
            // 丟棄物品
            "drop" => {
                if parts.len() > 1 {
                    let item_name = parts[1].to_string();
                    let quantity = if parts.len() > 2 {
                        parts[2].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Drop(item_name, quantity)
                } else {
                    CommandResult::Error("用法: drop <物品> [數量]".to_string())
                }
            }
            
            // 吃東西
            "eat" => {
                if parts.len() > 1 {
                    CommandResult::Eat(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: eat <食物>".to_string())
                }
            }
            
            // 睡覺
            "sleep" => CommandResult::Sleep,
            "dream" => {
                let target = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                CommandResult::Dream(target)
            }
            "wakeup" | "wake" => CommandResult::WakeUp,
            
            // 召喚
            "summon" | "spawn" => {
                if parts.len() > 1 {
                    CommandResult::Summon(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: summon <NPC名稱>".to_string())
                }
            }
            
            // 征服
            "conquer" | "capture" => {
                if parts.len() > 1 {
                    CommandResult::Conquer(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: conquer <地點名稱>".to_string())
                }
            }
            
            // 飛往
            "flyto" | "fly" | "teleport" | "tp" => {
                if parts.len() > 1 {
                    CommandResult::FlyTo(parts[1..].join(" "))
                } else {
                    CommandResult::Error("用法: flyto <地點>".to_string())
                }
            }
            
            // 命名
            "name" => {
                if parts.len() >= 3 {
                    let old_name = parts[1].to_string();
                    let new_name = parts[2..].join(" ");
                    CommandResult::Name(old_name, new_name)
                } else if parts.len() == 2 {
                    CommandResult::NameHere(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: name <新名稱> 或 name <舊名> <新名>".to_string())
                }
            }
            
            // 創建
            "create" => {
                if parts.len() >= 3 {
                    let type_str = parts[1].to_string();
                    let name = parts[2].to_string();
                    let desc = if parts.len() > 3 {
                        Some(parts[3..].join(" "))
                    } else {
                        None
                    };
                    CommandResult::Create(type_str, name, desc)
                } else {
                    CommandResult::Error("用法: create <類型> <名稱> [描述]".to_string())
                }
            }
            
            // 摧毀
            "destroy" | "remove" | "delete" => {
                if parts.len() > 1 {
                    CommandResult::Destroy(parts[1..].join(" "))
                } else {
                    CommandResult::Error("用法: destroy <目標>".to_string())
                }
            }
            
            // 設置屬性
            "set" => {
                if parts.len() >= 4 {
                    let npc_name = parts[1].to_string();
                    let attr = parts[2].to_string();
                    let value = parts[3].parse::<i32>().unwrap_or(0);
                    CommandResult::Set(npc_name, attr, value)
                } else {
                    CommandResult::Error("用法: set <NPC> <屬性> <值>".to_string())
                }
            }
            
            // NPC 相關
            "npcs" | "listnpcs" => CommandResult::ListNpcs,
            
            "ctrl" | "control" => {
                if parts.len() > 1 {
                    CommandResult::SwitchControl(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: ctrl <NPC名稱/id>".to_string())
                }
            }
            
            "trade" => {
                if parts.len() > 1 {
                    CommandResult::Trade(parts[1].to_string())
                } else {
                    CommandResult::Error("用法: trade <NPC>".to_string())
                }
            }
            
            "buy" => {
                if parts.len() >= 3 {
                    let npc = parts[1].to_string();
                    let item = parts[2].to_string();
                    let quantity = if parts.len() > 3 {
                        parts[3].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Buy(npc, item, quantity)
                } else {
                    CommandResult::Error("用法: buy <NPC> <物品> [數量]".to_string())
                }
            }
            
            "sell" => {
                if parts.len() >= 3 {
                    let npc = parts[1].to_string();
                    let item = parts[2].to_string();
                    let quantity = if parts.len() > 3 {
                        parts[3].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Sell(npc, item, quantity)
                } else {
                    CommandResult::Error("用法: sell <NPC> <物品> [數量]".to_string())
                }
            }
            
            // 對話設置
            "setdialogue" => {
                if parts.len() >= 4 {
                    let npc_name = parts[1].to_string();
                    let scene = parts[2].to_string();
                    let dialogue = parts[3..].join(" ");
                    CommandResult::SetDialogue(npc_name, scene, dialogue)
                } else {
                    CommandResult::Error("用法: setdialogue <NPC> <場景> <對話>".to_string())
                }
            }
            
            "seteagerness" => {
                if parts.len() >= 3 {
                    let npc_name = parts[1].to_string();
                    let eagerness_f32 = parts[2].parse::<f32>().unwrap_or(0.5);
                    let eagerness = (eagerness_f32.clamp(0.0, 1.0) * 100.0) as u8;
                    CommandResult::SetEagerness(npc_name, eagerness)
                } else {
                    CommandResult::Error("用法: seteagerness <NPC> <熱情度0-1>".to_string())
                }
            }
            
            // 打字機效果
            "typewriter" => CommandResult::ToggleTypewriter,
            
            // 未知命令
            _ => CommandResult::Error(format!("未知命令: {cmd}")),
        }
    }
}

impl Default for CommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}
