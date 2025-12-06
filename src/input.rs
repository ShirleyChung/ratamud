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

        match parts[0] {
            "exit" | "quit" => CommandResult::Exit,
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
                    CommandResult::Error("Usage: show status".to_string())
                } else if parts[1] == "status" {
                    CommandResult::ShowStatus
                } else if parts[1] == "world" {
                    CommandResult::ShowWorld
                } else {
                    CommandResult::Error(format!("Unknown show command: {}", parts[1]))
                }
            },
            "look" => {
                // look 命令，查看當前位置
                CommandResult::Look
            },
            "l" => {
                // l 可以是 look 或 left，根據上下文判斷
                // 但既然 look 優先，先檢查
                CommandResult::Look
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
            _ => CommandResult::Error(format!("Unknown command: {}", parts[0])),
        }
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
    Output(String),       // 在輸出區顯示的字串
    Ignored,             // 忽略（不顯示）
    Error(String),       // 命令錯誤顯示在狀態列
    Exit,                // 退出程式
    Clear,               // 清除文本區塊
    AddToSide(String),   // 添加到側邊面板
    ShowStatus,          // 打開狀態面板
    ShowWorld,           // 打開世界資訊面板
    CloseStatus,         // 關閉狀態面板
    Look,                // 查看當前位置
    Move(i32, i32),      // 移動 (dx, dy)，顯示方向
}
