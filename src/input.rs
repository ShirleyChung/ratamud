use crossterm::event::{Event, KeyCode};

// 處理用戶輸入的結構體
pub struct InputHandler {
    pub input: String,      // 當前輸入緩衝區
    pub buffer: Vec<String>, // 儲存所有已提交的文本
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
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => self.input.push(c),          // 添加字符
                KeyCode::Backspace => {
                    self.input.pop();                            // 刪除最後一個字符
                },
                KeyCode::Enter => {
                    // Enter 鍵提交輸入
                    if !self.input.is_empty() {
                        let result = self.parse_input(self.input.clone());
                        self.input.clear();
                        return Some(result);
                    }
                },
                // 方向鍵處理
                KeyCode::Up => {
                    return Some(CommandResult::Move(0, -1));     // 向上
                },
                KeyCode::Down => {
                    return Some(CommandResult::Move(0, 1));      // 向下
                },
                KeyCode::Left => {
                    return Some(CommandResult::Move(-1, 0));     // 向左
                },
                KeyCode::Right => {
                    return Some(CommandResult::Move(1, 0));      // 向右
                },
                _ => {}
            }
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
        let _is_status_command = matches!(parts[0], "status" | "show" if parts.len() > 1 && parts[1] == "status");
        
        let result = match parts[0] {
            "exit" | "quit" => CommandResult::Exit,
            "help" => CommandResult::Help,
            "save" => {
                // save [filename] 命令，預設檔名為 save.txt
                let filename = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "save.txt".to_string());
                self.execute_save(&filename)
            },
            "clear" => CommandResult::Clear,
            "status" => {
                // status 命令，顯示玩家狀態到側邊面板
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
            "show" => {
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
                } else if parts[1] == "map" {
                    CommandResult::ShowMap
                } else {
                    CommandResult::Error(format!("Unknown show command: {}", parts[1]))
                }
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
            "summon" => {
                // summon <npc名稱/id> 命令，召喚 NPC 到玩家位置
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
            "flyto" => {
                // flyto <坐標/地圖名/地點名> 命令
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
            "destroy" => {
                // destroy <npc/物品> 命令，刪除當前位置的 NPC 或物品
                if parts.len() < 2 {
                    CommandResult::Error("Usage: destroy <npc名稱|物品名稱>".to_string())
                } else {
                    CommandResult::Destroy(parts[1].to_string())
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
                    Err(e) => CommandResult::Error(format!("Write error: {}", e)),
                }
            },
            Err(e) => CommandResult::Error(format!("File error: {}", e)),
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
    Summon(String),                  // 召喚 NPC (NPC 名稱/ID)
    Conquer(String),                 // 征服指定方向，使其可行走 (up/down/left/right/u/d/l/r)
    FlyTo(String),                   // 飛到指定位置/地圖/地點 (坐標/地圖名/地點名)
    NameHere(String),                // 命名當前地點
    Name(String, String),            // 命名 NPC 或地點 (目標, 新名稱)
    Destroy(String),                 // 刪除指定的 NPC 或物品 (NPC名稱/物品名稱)
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
            CommandResult::Look(..) => Some(("look [<npc>]", "查看位置或NPC", "🎮 遊戲控制")),
            CommandResult::Move(..) => Some(("↑↓←→", "移動角色", "🎮 遊戲控制")),
            CommandResult::Conquer(..) => Some(("conq <方向>", "征服方向使其可行走", "🎮 遊戲控制")),
            CommandResult::FlyTo(..) => Some(("flyto <目標>", "傳送到位置/地圖/地點", "🎮 遊戲控制")),
            CommandResult::NameHere(..) => Some(("namehere <名稱>", "命名當前地點", "🎮 遊戲控制")),
            CommandResult::Name(..) => Some(("name <目標> <名稱>", "命名NPC或地點", "🎮 遊戲控制")),
            CommandResult::Get(..) => Some(("get [<物品>] [<數量>]", "撿起物品", "🎒 物品管理")),
            CommandResult::Drop(..) => Some(("drop <物品> <數量>", "放下物品", "🎒 物品管理")),
            CommandResult::Summon(..) => Some(("summon <npc>", "召喚NPC到此", "👥 NPC互動")),
            CommandResult::ShowStatus => Some(("show status", "顯示角色狀態", "ℹ️  資訊查詢")),
            CommandResult::ShowWorld => Some(("show world", "顯示世界資訊", "ℹ️  資訊查詢")),
            CommandResult::ShowMinimap => Some(("show minimap", "顯示小地圖", "🗺️  介面控制")),
            CommandResult::HideMinimap => Some(("hide minimap", "隱藏小地圖", "🗺️  介面控制")),
            CommandResult::ShowLog => Some(("show log", "顯示系統日誌", "🗺️  介面控制")),
            CommandResult::HideLog => Some(("hide log", "隱藏系統日誌", "🗺️  介面控制")),
            CommandResult::ShowMap => Some(("show map", "顯示大地圖 (↑↓←→移動, q退出)", "🗺️  介面控制")),
            CommandResult::Destroy(..) => Some(("destroy <目標>", "刪除NPC或物品", "🛠️  其他")),
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
        ];
        
        let mut categories: HashMap<&'static str, Vec<(&'static str, &'static str)>> = HashMap::new();
        
        for cmd in commands {
            if let Some((usage, desc, category)) = cmd.description() {
                categories.entry(category).or_insert_with(Vec::new).push((usage, desc));
            }
        }
        
        // 按指定順序返回
        let order = vec![
            "🎮 遊戲控制",
            "🎒 物品管理",
            "👥 NPC互動",
            "🗺️  介面控制",
            "ℹ️  資訊查詢",
            "🛠️  其他",
        ];
        
        order.into_iter()
            .filter_map(|cat| categories.remove(cat).map(|cmds| (cat, cmds)))
            .collect()
    }
}

