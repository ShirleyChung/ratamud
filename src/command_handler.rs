use std::collections::HashMap;

/// 命令解析結果
/// 這個枚舉包含所有可能的遊戲命令類型
#[derive(Clone, Debug)]
pub enum CommandResult {
    Output(String),                  // 在輸出區顯示的字串
    Error(String),                   // 命令錯誤顯示在狀態列
    Exit,
    Clear,                           // 清除文本區塊
    AddToSide(String),               // 添加到側邊面板
    ShowWorld,                       // 打開世界資訊面板
    ShowMinimap,                     // 打開小地圖面板
    HideMinimap,                     // 關閉小地圖面板
    ShowLog,                         // 打開日誌視窗
    HideLog,                         // 關閉日誌視窗
    ShowMap,                         // 打開大地圖顯示
    ShowHistory(usize),              // 顯示指令歷史記錄 (顯示數量)
    Look(Option<String>),            // 查看當前位置或查看 NPC (可選：NPC 名稱/ID)
    Move(i32, i32),                  // 移動 (dx, dy)，顯示方向
    Get(Option<String>, u32),        // 撿起物品 (可選：物品名稱, 數量)
    Drop(String, u32),               // 放下物品 (物品名稱, 數量)
    Eat(String),                     // 吃食物回復 HP (食物名稱)
    UseItem(String),                 // 使用物品 (物品名稱)
    UseItemOn(String, String),       // 對npc使用物品 (物品名稱)
    Sleep,
    Dream(Option<String>),           // 做夢 (可選：夢境內容)
    WakeUp,
    Summon(String),                  // 召喚 NPC (NPC 名稱/ID)
    Conquer(String),                 // 征服指定方向，使其可行走 (up/down/left/right/u/d/l/r)
    FlyTo(String),                   // 飛到指定位置/地圖/地點 (坐標/地圖名/地點名)
    NameHere(String),                // 命名當前地點
    Name(String, String),            // 命名 NPC 或地點 (目標, 新名稱)
    Destroy(String),                 // 刪除指定的 NPC 或物品 (NPC名稱/物品名稱)
    Create(String, String, Option<String>), // 創建物件 (類型, 物件類型, 可選名稱)
    Set(String, String, i32),        // 設置角色屬性 (目標人物, 屬性, 數值)
    SwitchControl(String),           // 切換操控的角色 (NPC名稱/ID)
    Trade(String),                   // 查看 NPC 商品 (NPC名稱/ID)
    Buy(String, String, u32),        // 購買物品 (NPC, 物品, 數量)
    Sell(String, String, u32),       // 出售物品 (NPC, 物品, 數量)
    Give(String, String, u32),       // 給予物品 (NPC, 物品, 數量)
    SetDialogue(String, String, String), // 設置 NPC 台詞 (NPC, 話題, 台詞)
    SetDialogueWithConditions(String, String, String, String), // 設置帶條件的 NPC 台詞 (NPC, 話題, 台詞, 條件字串)
    SetEagerness(String, u8),        // 設置 NPC 說話積極度 (NPC, 積極度0-100)
    SetRelationship(String, i32),    // 設置 NPC 好感度 (NPC, 好感度-100~100)
    ChangeRelationship(String, i32), // 改變 NPC 好感度 (NPC, 變化量)
    Talk(String, String),            // 與 NPC 對話 (NPC名稱/ID, 話題)
    Wait(String),                    // 叫住 NPC (NPC名稱/ID)
    Party(String),                   // 邀請 NPC 組隊 (NPC名稱/ID)
    Disband,                         // 解散隊伍
    Punch(Option<String>),           // 拳擊 (可選：目標)
    Kick(Option<String>),            // 踢擊 (可選：目標)
    Escape,                          // 逃離戰鬥
    ListNpcs,                        // 列出所有 NPC
    CheckNpc(String),                // 查看 NPC 詳細資訊 (NPC名稱/ID)
    ToggleTypewriter,                // 切換打字機效果
    // 任務系統
    QuestList,                       // 列出所有任務
    QuestActive,                     // 列出進行中的任務
    QuestAvailable,                  // 列出可接取的任務
    QuestCompleted,                  // 列出已完成的任務
    QuestInfo(String),               // 查看任務詳情 (任務ID)
    QuestStart(String),              // 開始任務 (任務ID)
    QuestComplete(String),           // 完成任務 (任務ID)
    QuestAbandon(String),            // 放棄任務 (任務ID)
    Help,                            // 顯示幫助訊息
}

impl CommandResult {
    /// 獲取指令說明
    pub fn description(&self) -> Option<(&'static str, &'static str, &'static str)> {
        // 返回 (指令語法, 說明, 分類)
        match self {
            CommandResult::Exit => Some(("exit / quit", "退出遊戲", "🎮 遊戲控制")),
            CommandResult::Help => Some(("help", "顯示此幫助訊息", "🎮 遊戲控制")),
            CommandResult::Clear => Some(("clear", "清除訊息輸出", "🛠️  其他")),
            CommandResult::ShowHistory(..) => Some(("history / hist [<數量>]", "顯示指令歷史記錄", "ℹ️  資訊查詢")),
            CommandResult::Look(..) => Some(("look / l [<npc>]", "查看位置或NPC", "🎮 遊戲控制")),
            CommandResult::Move(..) => Some(("↑↓←→ / up/down/left/right (u/d/r)", "移動角色", "🎮 遊戲控制")),
            CommandResult::Conquer(..) => Some(("conq / conquer <方向>", "征服方向使其可行走", "🎮 遊戲控制")),
            CommandResult::FlyTo(..) => Some(("flyto / ft <目標>", "傳送到位置/地圖/地點", "🎮 遊戲控制")),
            CommandResult::NameHere(..) => Some(("namehere <名稱>", "命名當前地點", "🎮 遊戲控制")),
            CommandResult::Name(..) => Some(("name <目標> <名稱>", "命名NPC或地點", "🎮 遊戲控制")),
            CommandResult::Get(..) => Some(("get [<物品>] [<數量>]", "撿起物品", "🎒 物品管理")),
            CommandResult::Drop(..) => Some(("drop <物品> <數量>", "放下物品", "🎒 物品管理")),
            CommandResult::Eat(..) => Some(("eat <食物>", "吃食物回復HP", "🎒 物品管理")),
            CommandResult::UseItem(..) => Some(("use <物品>", "使用物品（藥水/食物等）", "🎒 物品管理")),
            CommandResult::UseItemOn(..) => Some(("use <物品> on <npc>", "對NPC使用物品（藥水/食物等）", "🎒 物品管理")),
            CommandResult::Sleep => Some(("sleep", "進入睡眠狀態", "💤 睡眠")),
            CommandResult::Dream(..) => Some(("dream [<內容>]", "做夢（睡眠時）", "💤 睡眠")),
            CommandResult::WakeUp => Some(("wakeup / wake", "從睡眠中醒來", "💤 睡眠")),
            CommandResult::Summon(..) => Some(("summon / sn <npc>", "召喚NPC到此", "👥 NPC互動")),
            CommandResult::ShowWorld => Some(("show world", "顯示世界資訊", "ℹ️  資訊查詢")),
            CommandResult::ShowMinimap => Some(("show minimap", "顯示小地圖", "🗺️  介面控制")),
            CommandResult::HideMinimap => Some(("hide minimap", "隱藏小地圖", "🗺️  介面控制")),
            CommandResult::ShowLog => Some(("show log", "顯示系統日誌", "🗺️  介面控制")),
            CommandResult::HideLog => Some(("hide log", "隱藏系統日誌", "🗺️  介面控制")),
            CommandResult::ShowMap => Some(("show map / sm", "顯示大地圖 (↑↓←→移動, q退出", "🗺️  介面控制")),
            CommandResult::Destroy(..) => Some(("destroy / ds <目標>", "刪除NPC或物品", "🛠️  其他")),
            CommandResult::Create(..) => Some(("create / cr <類型> <物件類型> [名稱]", "創建物件 (item/npc)", "🛠️  其他")),
            CommandResult::Set(..) => Some(("set <人物> <屬性> <數值> 或 set item <物品> <價格>", "設置角色屬性 (hp/mp/strength/knowledge/sociality/gold) 或物品價格", "🛠️  其他")),
            CommandResult::SwitchControl(..) => Some(("ctrl / control <npc>", "切換操控的角色", "👥 NPC互動")),
            CommandResult::Trade(..) => Some(("trade <npc>", "查看NPC商品", "💰 交易")),
            CommandResult::Buy(..) => Some(("buy <npc> <item> [數量]", "購買物品", "💰 交易")),
            CommandResult::Sell(..) => Some(("sell <npc> <item> [數量]", "出售物品", "💰 交易")),
            CommandResult::Give(..) => Some(("give <npc> <item> [數量]", "給予NPC物品", "👥 NPC互動")),
            CommandResult::Wait(..) => Some(("wait <npc>", "叫住NPC（基於好感度）", "👥 NPC互動")),
            CommandResult::Party(..) => Some(("party <npc>", "邀請NPC組隊", "👥 NPC互動")),
            CommandResult::Disband => Some(("disband", "解散隊伍", "👥 NPC互動")),
            CommandResult::Punch(..) => Some(("punch / ph [目標]", "拳擊（無目標=練習）", "⚔️  戰鬥")),
            CommandResult::Kick(..) => Some(("kick / kk [目標]", "踢擊（無目標=練習）", "⚔️  戰鬥")),
            CommandResult::Escape => Some(("escape / esc", "逃離戰鬥", "⚔️  戰鬥")),
            CommandResult::ListNpcs => Some(("npcs", "列出所有NPC", "👥 NPC互動")),
            _ => None,
        }
    }

    /// 獲取所有可用指令的說明（按分類分組）
    pub fn get_help_info() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
        
        // 所有指令的代表實例
        let commands = vec![
            CommandResult::Move(0, 0),
            CommandResult::Look(None),
            CommandResult::Conquer(String::new()),
            CommandResult::FlyTo(String::new()),
            CommandResult::NameHere(String::new()),
            CommandResult::Name(String::new(), String::new()),
            CommandResult::Help,
            CommandResult::Exit,
            CommandResult::Get(None, 1),
            CommandResult::Drop(String::new(), 1),
            CommandResult::Eat(String::new()),
            CommandResult::Sleep,
            CommandResult::Dream(None),
            CommandResult::WakeUp,
            CommandResult::Summon(String::new()),
            CommandResult::ShowMinimap,
            CommandResult::HideMinimap,
            CommandResult::ShowLog,
            CommandResult::HideLog,
            CommandResult::ShowMap,
            CommandResult::ShowWorld,
            CommandResult::ShowHistory(10),
            CommandResult::Clear,
            CommandResult::Destroy(String::new()),
            CommandResult::Create(String::new(), String::new(), None),
            CommandResult::Set(String::new(), String::new(), 0),
            CommandResult::SwitchControl(String::new()),
            CommandResult::Trade(String::new()),
            CommandResult::Buy(String::new(), String::new(), 1),
            CommandResult::Sell(String::new(), String::new(), 1),
            CommandResult::Give(String::new(), String::new(), 1),
            CommandResult::ListNpcs,
            CommandResult::SetDialogue(String::new(), String::new(), String::new()),
            CommandResult::SetDialogueWithConditions(String::new(), String::new(), String::new(), String::new()),
            CommandResult::SetEagerness(String::new(), 0),
            CommandResult::SetRelationship(String::new(), 0),
            CommandResult::ChangeRelationship(String::new(), 0),
            CommandResult::Talk(String::new(), String::new()),
            CommandResult::Wait(String::new()),
            CommandResult::CheckNpc(String::new()),
            CommandResult::ToggleTypewriter,
            CommandResult::QuestList,
            CommandResult::QuestActive,
            CommandResult::QuestAvailable,
            CommandResult::QuestCompleted,
            CommandResult::QuestInfo(String::new()),
            CommandResult::QuestStart(String::new()),
            CommandResult::QuestComplete(String::new()),
            CommandResult::QuestAbandon(String::new()),
        ];
        
        let mut categories: HashMap<&'static str, Vec<(&'static str, &'static str)>> = HashMap::new();
        
        for cmd in commands {
            if let Some((usage, desc, category)) = cmd.description() {
                categories.entry(category).or_default().push((usage, desc));
            }
        }
        
        // 按指定順序返回
        let order = vec![
            "🎮 遊戲控制",
            "🎒 物品管理",
            "👥 NPC互動",
            "💰 交易",
            "⚔️  戰鬥",
            "🗺️  介面控制",
            "ℹ️  資訊查詢",
            "💤 睡眠",
            "🛠️  其他",
        ];
        
        let mut result_vec = Vec::new();
        for cat in order {
            if let Some(mut cmds) = categories.remove(cat) {
                // 字母排序
                cmds.sort_by(|a, b| a.0.cmp(b.0));
                
                // 在遊戲控制分類中手動添加 re 命令
                if cat == "🎮 遊戲控制" {
                    cmds.push(("re / repeat", "重複上一次的命令"));
                    cmds.sort_by(|a, b| a.0.cmp(b.0));
                }
                
                result_vec.push((cat, cmds));
            }
        }
        
        result_vec
    }
}

/// 命令解析器 - 將文字命令轉換為 CommandResult
pub fn parse_command(input: &str) -> CommandResult {
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    if parts.is_empty() {
        return CommandResult::Error("No command provided".to_string());
    }

    match parts[0] {
        "exit" | "quit" => CommandResult::Exit,
        "help" => CommandResult::Help,
        "save" => {
            // save [filename] 命令，預設檔名為 save.txt
            let filename = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "save.txt".to_string());
            CommandResult::Output(format!("Save command: {}", filename))
        },
        "clear" => CommandResult::Clear,
        "status" | "i" => CommandResult::CheckNpc("me".to_string()),
        "hello" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: hello <message>".to_string())
            } else {
                let message = parts[1..].join(" ");
                CommandResult::Output(message)
            }
        },
        "sideadd" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: sideadd <message>".to_string())
            } else {
                let message = parts[1..].join(" ");
                CommandResult::AddToSide(message)
            }
        },
        "show" | "s" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: show <command>".to_string())
            } else if parts[1] == "status" {
                CommandResult::CheckNpc("me".to_string())
            } else if parts[1] == "world" {
                CommandResult::ShowWorld
            } else if parts[1] == "minimap" {
                CommandResult::ShowMinimap
            } else if parts[1] == "log" {
                CommandResult::ShowLog
            } else if parts[1] == "map" || parts[1] == "m" {
                CommandResult::ShowMap
            } else {
                CommandResult::Error(format!("Unknown show command: {}", parts[1]))
            }
        },
        "sm" => CommandResult::ShowMap,
        "hide" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: hide <command>".to_string())
            } else if parts[1] == "minimap" {
                CommandResult::HideMinimap
            } else if parts[1] == "log" {
                CommandResult::HideLog
            } else {
                CommandResult::Error(format!("Unknown hide command: {}", parts[1]))
            }
        },
        "typewriter" | "tw" => CommandResult::ToggleTypewriter,
        "look" | "l" => {
            if parts.len() < 2 {
                CommandResult::Look(None)
            } else {
                CommandResult::Look(Some(parts[1].to_string()))
            }
        },
        "get" => {
            if parts.len() < 2 {
                CommandResult::Get(None, 1)
            } else if parts.len() == 2 {
                let item_name = parts[1].to_string();
                CommandResult::Get(Some(item_name), 1)
            } else {
                let item_name = parts[1].to_string();
                let quantity = parts[2].parse::<u32>().unwrap_or(1);
                CommandResult::Get(Some(item_name), quantity)
            }
        },
        "drop" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: drop <item name> [quantity]".to_string())
            } else if parts.len() == 2 {
                let item_name = parts[1].to_string();
                CommandResult::Drop(item_name, 1)
            } else {
                let item_name = parts[1].to_string();
                let quantity = parts[2].parse::<u32>().unwrap_or(1);
                CommandResult::Drop(item_name, quantity)
            }
        },
        "eat" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: eat <food name>".to_string())
            } else {
                let food_name = parts[1].to_string();
                CommandResult::Eat(food_name)
            }
        },
        "use" => {
            if parts.len() < 2 {
                CommandResult::Error("用法: use <物品名稱> [on <目標>]".to_string())
            } else if parts.len() >= 4 && parts[2] == "on" {
                let item_name = parts[1].to_string();
                let target_name = parts[3].to_string();
                CommandResult::UseItemOn(item_name, target_name)
            } else {
                let item_name = parts[1].to_string();
                CommandResult::UseItem(item_name)
            }
        },
        "npcs" | "listnpcs" => CommandResult::ListNpcs,
        "sleep" => CommandResult::Sleep,
        "dream" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: dream [content]".to_string())
            } else {
                CommandResult::Dream(Some(parts[1..].join(" ")))
            }
        },
        "wakeup" | "wake" => CommandResult::WakeUp,
        "right" | "r" => CommandResult::Move(1, 0),
        "left" => CommandResult::Move(-1, 0),
        "up" | "u" => CommandResult::Move(0, -1),
        "down" | "d" => CommandResult::Move(0, 1),
        "north" | "n" => CommandResult::Move(0, -1),
        "south" => CommandResult::Move(0, 1),
        "east" | "e" => CommandResult::Move(1, 0),
        "west" | "w" => CommandResult::Move(-1, 0),
        "summon" | "sn" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: summon <npc名稱/id>".to_string())
            } else {
                CommandResult::Summon(parts[1].to_string())
            }
        },
        "conq" | "conquer" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: conq <up|down|left|right>".to_string())
            } else {
                CommandResult::Conquer(parts[1].to_string())
            }
        },
        "flyto" | "ft" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: flyto <目標>".to_string())
            } else {
                CommandResult::FlyTo(parts[1..].join(" "))
            }
        },
        "namehere" | "nh" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: namehere <名稱>".to_string())
            } else {
                CommandResult::NameHere(parts[1..].join(" "))
            }
        },
        "name" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: name <目標> <名稱>".to_string())
            } else {
                CommandResult::Name(parts[1].to_string(), parts[2..].join(" "))
            }
        },
        "destroy" | "ds" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: destroy <目標>".to_string())
            } else {
                CommandResult::Destroy(parts[1].to_string())
            }
        },
        "create" | "cr" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: create <類型> <物件類型> [名稱]".to_string())
            } else {
                let obj_type = parts[1].to_string();
                let subtype = parts[2].to_string();
                let name = if parts.len() > 3 {
                    Some(parts[3..].join(" "))
                } else {
                    None
                };
                CommandResult::Create(obj_type, subtype, name)
            }
        },
        "set" => {
            if parts.len() < 4 {
                CommandResult::Error("Usage: set <人物> <屬性> <數值>".to_string())
            } else {
                let target = parts[1].to_string();
                let attr = parts[2].to_string();
                let value = parts[3].parse::<i32>().unwrap_or(0);
                CommandResult::Set(target, attr, value)
            }
        },
        "ctrl" | "control" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: ctrl <npc>".to_string())
            } else {
                CommandResult::SwitchControl(parts[1].to_string())
            }
        },
        "trade" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: trade <npc>".to_string())
            } else {
                CommandResult::Trade(parts[1].to_string())
            }
        },
        "buy" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: buy <npc> <item> [數量]".to_string())
            } else {
                let npc = parts[1].to_string();
                let item = parts[2].to_string();
                let qty = parts.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                CommandResult::Buy(npc, item, qty)
            }
        },
        "sell" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: sell <npc> <item> [數量]".to_string())
            } else {
                let npc = parts[1].to_string();
                let item = parts[2].to_string();
                let qty = parts.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                CommandResult::Sell(npc, item, qty)
            }
        },
        "give" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: give <npc> <item> [數量]".to_string())
            } else {
                let npc = parts[1].to_string();
                let item = parts[2].to_string();
                let qty = parts.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                CommandResult::Give(npc, item, qty)
            }
        },
        "talk" => {
            if parts.len() < 3 {
                CommandResult::Error("Usage: talk <npc> <話題>".to_string())
            } else {
                let npc = parts[1].to_string();
                let topic = parts[2..].join(" ");
                CommandResult::Talk(npc, topic)
            }
        },
        "wait" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: wait <npc>".to_string())
            } else {
                CommandResult::Wait(parts[1].to_string())
            }
        },
        "party" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: party <npc>".to_string())
            } else {
                CommandResult::Party(parts[1].to_string())
            }
        },
        "disband" => CommandResult::Disband,
        "punch" | "ph" => {
            if parts.len() < 2 {
                CommandResult::Punch(None)
            } else {
                CommandResult::Punch(Some(parts[1].to_string()))
            }
        },
        "kick" | "kk" => {
            if parts.len() < 2 {
                CommandResult::Kick(None)
            } else {
                CommandResult::Kick(Some(parts[1].to_string()))
            }
        },
        "escape" | "esc" => CommandResult::Escape,
        "check" => {
            if parts.len() < 2 {
                CommandResult::Error("Usage: check <npc>".to_string())
            } else {
                CommandResult::CheckNpc(parts[1].to_string())
            }
        },
        "quest" | "q" => {
            if parts.len() < 2 {
                CommandResult::QuestList
            } else {
                match parts[1] {
                    "list" => CommandResult::QuestList,
                    "active" => CommandResult::QuestActive,
                    "available" => CommandResult::QuestAvailable,
                    "completed" => CommandResult::QuestCompleted,
                    "info" => {
                        if parts.len() < 3 {
                            CommandResult::Error("Usage: quest info <quest_id>".to_string())
                        } else {
                            CommandResult::QuestInfo(parts[2].to_string())
                        }
                    },
                    "start" => {
                        if parts.len() < 3 {
                            CommandResult::Error("Usage: quest start <quest_id>".to_string())
                        } else {
                            CommandResult::QuestStart(parts[2].to_string())
                        }
                    },
                    "complete" => {
                        if parts.len() < 3 {
                            CommandResult::Error("Usage: quest complete <quest_id>".to_string())
                        } else {
                            CommandResult::QuestComplete(parts[2].to_string())
                        }
                    },
                    "abandon" => {
                        if parts.len() < 3 {
                            CommandResult::Error("Usage: quest abandon <quest_id>".to_string())
                        } else {
                            CommandResult::QuestAbandon(parts[2].to_string())
                        }
                    },
                    _ => CommandResult::Error(format!("Unknown quest command: {}", parts[1])),
                }
            }
        },
        _ => CommandResult::Error(format!("Unknown command: {}", parts[0])),
    }
}
