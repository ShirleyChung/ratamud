use crossterm::event::{Event, KeyCode, KeyEventKind};

// 處理用戶輸入的結構體
pub struct InputHandler {
    pub input: String,      // 當前輸入緩衝區
    pub buffer: Vec<String>, // 儲存所有已提交的文本
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    // 建立新的輸入處理器
    pub fn new() -> Self {
        InputHandler {
            input: String::new(),
            buffer: Vec::new(),
        }
    }

    // 處理鍵盤事件
    pub fn handle_event(&mut self, event: Event) -> Option<CommandResult> {
        match event {
            Event::Paste(s) => {
                self.input.push_str(&s);
            }

            Event::Key(key) => {
                // ✅ Windows 相容：只處理 Press 事件，忽略 Repeat 和 Release
                // 這樣既支援中文輸入，又避免 Windows 的重複字符問題
                match key.kind {
                    KeyEventKind::Press => {
                        // 只處理按下事件
                    }
                    KeyEventKind::Repeat => {
                        // Windows 上會觸發 Repeat，我們忽略它
                        return None;
                    }
                    _ => {
                        // Release 等其他事件也忽略
                        return None;
                    }
                }

                match key.code {
                    KeyCode::Char(c) => {
                        self.input.push(c);
                    }

                    KeyCode::Backspace => {
                        self.input.pop();
                    }

                    KeyCode::Enter => {
                        if !self.input.is_empty() {
                            let result = self.parse_input(self.input.clone());
                            self.input.clear();
                            return Some(result);
                        }
                    }

                    KeyCode::Up => return Some(CommandResult::Move(0, -1)),
                    KeyCode::Down => return Some(CommandResult::Move(0, 1)),
                    KeyCode::Left => return Some(CommandResult::Move(-1, 0)),
                    KeyCode::Right => return Some(CommandResult::Move(1, 0)),

                    _ => {}
                }
            }

            _ => {}
        }

        None
    }

    // 取得目前輸入的文本
    pub fn get_input(&self) -> &str {
        &self.input
    }

    // 清除目前輸入的文本
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    // 解析輸入內容，所有輸入都視為命令
    fn parse_input(&mut self, input: String) -> CommandResult {
        // 所有輸入都當作命令處理
        self.handle_command(input)
    }

    // 處理命令（所有輸入都是命令，不需要 / 前綴）
    fn handle_command(&mut self, input: String) -> CommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return CommandResult::Error("No command provided".to_string());
        }

        // 先檢查是否為 status 相關命令（這些命令不應關閉 status）
        let _is_status_command = matches!(parts[0], "status" | "i" | "show" | "s" if parts.len() == 1 && (parts[0] == "status" || parts[0] == "i") || (parts.len() > 1 && parts[1] == "status"));
        
        let result = match parts[0] {
            "exit" | "quit" => CommandResult::Exit,
            "help" => CommandResult::Help,
            "save" => {
                // save [filename] 命令，預設檔名為 save.txt
                let filename = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "save.txt".to_string());
                self.execute_save(&filename)
            },
            "clear" => CommandResult::Clear,
            "status" | "i" => {
                // status/i 命令，顯示玩家狀態到側邊面板
                CommandResult::ShowStatus
            },
            "hello" => {
                // hello <message> 命令，在輸出區顯示 hello 之後的字串
                if parts.len() < 2 {
                    CommandResult::Error("Usage: hello <message>".to_string())
                } else {
                    let message = parts[1..].join(" ");
                    self.buffer.push(message.clone());
                    CommandResult::Output(message)  // 只顯示後面的字串
                }
            },
            "sideadd" => {
                // sideadd <message> 命令，添加訊息到側邊面板
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
                    CommandResult::ShowStatus
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
            "sm" => {
                // sm 是 show map 的別名
                CommandResult::ShowMap
            },
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
            "typewriter" | "tw" => {
                // 切換打字機效果
                CommandResult::ToggleTypewriter
            },
            "look" | "l" => {
                // look/l 命令，查看當前位置或 NPC
                // look - 查看當前位置
                // look <npc名稱/id> - 查看 NPC 狀態
                if parts.len() < 2 {
                    CommandResult::Look(None)
                } else {
                    CommandResult::Look(Some(parts[1].to_string()))
                }
            },
            "get" => {
                // get 命令，撿起物品
                // get - 撿起所有物品
                // get <物品名稱> - 撿起指定物品（數量1）
                // get <物品名稱> <數量> - 撿起指定數量
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
                // drop 命令，放下物品
                // drop <物品名稱> - 放下1個
                // drop <物品名稱> <數量> - 放下指定數量
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
                // eat 命令，吃食物回復 HP
                // eat <食物名稱>
                if parts.len() < 2 {
                    CommandResult::Error("Usage: eat <food name>".to_string())
                } else {
                    let food_name = parts[1].to_string();
                    CommandResult::Eat(food_name)
                }
            },
            "npcs" | "listnpcs" => {
                // npcs 命令，列出所有 NPC
                CommandResult::ListNpcs
            },
            "sleep" => {
                // sleep 命令，進入睡眠狀態
                CommandResult::Sleep
            },
            "dream" => {
                // dream 命令，在睡眠時做夢
                if parts.len() < 2 {
                    CommandResult::Dream(None)
                } else {
                    CommandResult::Dream(Some(parts[1..].join(" ")))
                }
            },
            "wakeup" | "wake" => {
                // wakeup/wake 命令，從睡眠中醒來
                CommandResult::WakeUp
            },
            "right" | "r" => {
                // 向右移動
                CommandResult::Move(1, 0)
            },
            "left" => {
                // 向左移動
                CommandResult::Move(-1, 0)
            },
            "up" | "u" => {
                // 向上移動
                CommandResult::Move(0, -1)
            },
            "down" | "d" => {
                // 向下移動
                CommandResult::Move(0, 1)
            },
            "summon" | "sn" => {
                // summon/sn <npc名稱/id> 命令，召喚 NPC 到玩家位置
                if parts.len() < 2 {
                    CommandResult::Error("Usage: summon <npc名稱/id>".to_string())
                } else {
                    CommandResult::Summon(parts[1].to_string())
                }
            },
            "conq" | "conquer" => {
                // conq <方向> 命令，征服指定方向使其可行走
                // 支持: up/down/left/right 或 u/d/l/r
                if parts.len() < 2 {
                    CommandResult::Error("Usage: conq <up|down|left|right>".to_string())
                } else {
                    CommandResult::Conquer(parts[1].to_string())
                }
            },
            "flyto" | "ft" => {
                // flyto/ft <坐標/地圖名/地點名> 命令
                if parts.len() < 2 {
                    CommandResult::Error("Usage: flyto <x,y|地圖名|地點名>".to_string())
                } else {
                    CommandResult::FlyTo(parts[1].to_string())
                }
            },
            "namehere" => {
                // namehere <名稱> 命令，命名當前地點
                if parts.len() < 2 {
                    CommandResult::Error("Usage: namehere <名稱>".to_string())
                } else {
                    CommandResult::NameHere(parts[1..].join(" "))
                }
            },
            "name" => {
                // name <目標> <名稱> 命令
                // name <npc> <新名稱> 或 name <x,y> <地點名稱>
                if parts.len() < 3 {
                    CommandResult::Error("Usage: name <npc|x,y> <新名稱>".to_string())
                } else {
                    CommandResult::Name(parts[1].to_string(), parts[2..].join(" "))
                }
            },
            "destroy" | "ds" => {
                // destroy/ds <npc/物品> 命令，刪除當前位置的 NPC 或物品
                if parts.len() < 2 {
                    CommandResult::Error("Usage: destroy <npc名稱|物品名稱>".to_string())
                } else {
                    CommandResult::Destroy(parts[1].to_string())
                }
            },
            "create" | "cr" => {
                // create/cr <類型> <物件類型> [名稱] 命令，創建物件
                // 類型: item 或 npc
                // 物件類型: 如 "工人", "蘋果" 等
                // 名稱: 可選，自訂義名稱
                if parts.len() < 3 {
                    CommandResult::Error("Usage: create <item|npc> <物件類型> [名稱]".to_string())
                } else {
                    let obj_type = parts[1].to_string();
                    let item_type = parts[2].to_string();
                    let name = if parts.len() > 3 {
                        Some(parts[3..].join(" "))
                    } else {
                        None
                    };
                    CommandResult::Create(obj_type, item_type, name)
                }
            },
            "set" => {
                // set <目標人物> <屬性> <數值> 命令，設置角色屬性
                // 支持: hp, mp, strength, knowledge, sociality
                if parts.len() < 4 {
                    CommandResult::Error("Usage: set <目標人物> <屬性> <數值>".to_string())
                } else {
                    let target = parts[1].to_string();
                    let attribute = parts[2].to_string();
                    let value = parts[3].parse::<i32>().unwrap_or(0);
                    CommandResult::Set(target, attribute, value)
                }
            },
            "ctrl" | "control" => {
                // ctrl/control <npc名稱/id> 命令，切換當前操控的角色
                if parts.len() < 2 {
                    CommandResult::Error("Usage: ctrl <npc名稱/id>".to_string())
                } else {
                    CommandResult::SwitchControl(parts[1].to_string())
                }
            },
            "trade" => {
                // trade <npc> 命令，查看 NPC 的商品列表
                if parts.len() < 2 {
                    CommandResult::Error("Usage: trade <npc>".to_string())
                } else {
                    CommandResult::Trade(parts[1].to_string())
                }
            },
            "buy" => {
                // buy <npc> <item> [quantity] 命令，從 NPC 購買物品
                if parts.len() < 3 {
                    CommandResult::Error("Usage: buy <npc> <item> [quantity]".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let item = parts[2].to_string();
                    let quantity = if parts.len() > 3 {
                        parts[3].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Buy(npc, item, quantity)
                }
            },
            "sell" => {
                // sell <npc> <item> [quantity] 命令，向 NPC 出售物品
                if parts.len() < 3 {
                    CommandResult::Error("Usage: sell <npc> <item> [quantity]".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let item = parts[2].to_string();
                    let quantity = if parts.len() > 3 {
                        parts[3].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Sell(npc, item, quantity)
                }
            },
            "setdialogue" | "setdia" => {
                // setdialogue <npc> <場景> <台詞> 命令，設置 NPC 台詞
                // 範例: setdialogue 商人 見面 哈囉！你好，來看看我的商品
                if parts.len() < 4 {
                    CommandResult::Error("Usage: setdialogue <npc> <場景> <台詞>".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let scene = parts[2].to_string();
                    let dialogue = parts[3..].join(" ");
                    CommandResult::SetDialogue(npc, scene, dialogue)
                }
            },
            "seteagerness" | "setea" => {
                // seteagerness <npc> <積極度> 命令，設置 NPC 說話積極度 (0-100)
                // 範例: seteagerness 商人 100
                if parts.len() < 3 {
                    CommandResult::Error("Usage: seteagerness <npc> <積極度(0-100)>".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let eagerness = parts[2].parse::<u8>().unwrap_or(100).min(100);
                    CommandResult::SetEagerness(npc, eagerness)
                }
            },
            _ => CommandResult::Error(format!("Unknown command: {}", parts[0])),
        };
        result
    }

    // 執行保存命令，將所有文本寫入檔案
    fn execute_save(&self, filename: &str) -> CommandResult {
        use std::fs::File;
        use std::io::Write;

        let content = self.buffer.join("\n");
        
        match File::create(filename) {
            Ok(mut file) => {
                match file.write_all(content.as_bytes()) {
                    Ok(_) => CommandResult::Error(format!("Saved {} lines to {}", self.buffer.len(), filename)),
                    Err(e) => CommandResult::Error(format!("Write error: {e}")),
                }
            },
            Err(e) => CommandResult::Error(format!("File error: {e}")),
        }
    }
}

// 命令執行結果的列舉
pub enum CommandResult {
    Output(String),                  // 在輸出區顯示的字串
    Error(String),                   // 命令錯誤顯示在狀態列
    Exit,                            // 退出程式
    Clear,                           // 清除文本區塊
    AddToSide(String),               // 添加到側邊面板
    ShowStatus,                      // 打開狀態面板
    ShowWorld,                       // 打開世界資訊面板
    ShowMinimap,                     // 打開小地圖面板
    HideMinimap,                     // 關閉小地圖面板
    ShowLog,                         // 打開日誌視窗
    HideLog,                         // 關閉日誌視窗
    ShowMap,                         // 打開大地圖顯示
    Look(Option<String>),            // 查看當前位置或查看 NPC (可選：NPC 名稱/ID)
    Move(i32, i32),                  // 移動 (dx, dy)，顯示方向
    Get(Option<String>, u32),        // 撿起物品 (可選：物品名稱, 數量)
    Drop(String, u32),               // 放下物品 (物品名稱, 數量)
    Eat(String),                     // 吃食物回復 HP (食物名稱)
    Sleep,                           // 進入睡眠狀態
    Dream(Option<String>),           // 做夢 (可選：夢境內容)
    WakeUp,                          // 從睡眠中醒來
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
    SetDialogue(String, String, String), // 設置 NPC 台詞 (NPC, 場景, 台詞)
    SetEagerness(String, u8),        // 設置 NPC 說話積極度 (NPC, 積極度0-100)
    ListNpcs,                        // 列出所有 NPC
    ToggleTypewriter,                // 切換打字機效果
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
            CommandResult::Look(..) => Some(("look / l [<npc>]", "查看位置或NPC", "🎮 遊戲控制")),
            CommandResult::Move(..) => Some(("↑↓←→ / up/down/left/right (u/d/r)", "移動角色", "🎮 遊戲控制")),
            CommandResult::Conquer(..) => Some(("conq / conquer <方向>", "征服方向使其可行走", "🎮 遊戲控制")),
            CommandResult::FlyTo(..) => Some(("flyto / ft <目標>", "傳送到位置/地圖/地點", "🎮 遊戲控制")),
            CommandResult::NameHere(..) => Some(("namehere <名稱>", "命名當前地點", "🎮 遊戲控制")),
            CommandResult::Name(..) => Some(("name <目標> <名稱>", "命名NPC或地點", "🎮 遊戲控制")),
            CommandResult::Get(..) => Some(("get [<物品>] [<數量>]", "撿起物品", "🎒 物品管理")),
            CommandResult::Drop(..) => Some(("drop <物品> <數量>", "放下物品", "🎒 物品管理")),
            CommandResult::Eat(..) => Some(("eat <食物>", "吃食物回復HP", "🎒 物品管理")),
            CommandResult::Sleep => Some(("sleep", "進入睡眠狀態", "💤 睡眠")),
            CommandResult::Dream(..) => Some(("dream [<內容>]", "做夢（睡眠時）", "💤 睡眠")),
            CommandResult::WakeUp => Some(("wakeup / wake", "從睡眠中醒來", "💤 睡眠")),
            CommandResult::Summon(..) => Some(("summon / sn <npc>", "召喚NPC到此", "👥 NPC互動")),
            CommandResult::ShowStatus => Some(("status / i", "顯示角色狀態", "ℹ️  資訊查詢")),
            CommandResult::ShowWorld => Some(("show world", "顯示世界資訊", "ℹ️  資訊查詢")),
            CommandResult::ShowMinimap => Some(("show minimap", "顯示小地圖", "🗺️  介面控制")),
            CommandResult::HideMinimap => Some(("hide minimap", "隱藏小地圖", "🗺️  介面控制")),
            CommandResult::ShowLog => Some(("show log", "顯示系統日誌", "🗺️  介面控制")),
            CommandResult::HideLog => Some(("hide log", "隱藏系統日誌", "🗺️  介面控制")),
            CommandResult::ShowMap => Some(("show map / sm", "顯示大地圖 (↑↓←→移動, q退出)", "🗺️  介面控制")),
            CommandResult::Destroy(..) => Some(("destroy / ds <目標>", "刪除NPC或物品", "🛠️  其他")),
            CommandResult::Create(..) => Some(("create / cr <類型> <物件類型> [名稱]", "創建物件 (item/npc)", "🛠️  其他")),
            CommandResult::Set(..) => Some(("set <人物> <屬性> <數值>", "設置角色屬性 (hp/mp/strength/knowledge/sociality)", "🛠️  其他")),
            CommandResult::SwitchControl(..) => Some(("ctrl / control <npc>", "切換操控的角色", "👥 NPC互動")),
            CommandResult::Trade(..) => Some(("trade <npc>", "查看NPC商品", "💰 交易")),
            CommandResult::Buy(..) => Some(("buy <npc> <item> [數量]", "購買物品", "💰 交易")),
            CommandResult::Sell(..) => Some(("sell <npc> <item> [數量]", "出售物品", "💰 交易")),
            CommandResult::ListNpcs => Some(("npcs", "列出所有NPC", "👥 NPC互動")),
            _ => None,
        }
    }

    /// 獲取所有可用指令的說明（按分類分組）
    pub fn get_help_info() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
        use std::collections::HashMap;
        
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
            CommandResult::ShowStatus,
            CommandResult::ShowWorld,
            CommandResult::Clear,
            CommandResult::Destroy(String::new()),
            CommandResult::Create(String::new(), String::new(), None),
            CommandResult::Set(String::new(), String::new(), 0),
            CommandResult::SwitchControl(String::new()),
            CommandResult::Trade(String::new()),
            CommandResult::Buy(String::new(), String::new(), 1),
            CommandResult::Sell(String::new(), String::new(), 1),
            CommandResult::ListNpcs,
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
            "🗺️  介面控制",
            "ℹ️  資訊查詢",
            "💤 睡眠",
            "🛠️  其他",
        ];
        
        order.into_iter()
            .filter_map(|cat| categories.remove(cat).map(|cmds| (cat, cmds)))
            .collect()
    }
}

