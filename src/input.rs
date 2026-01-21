use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::app::AppContext; // Add AppContext import
use crate::ui::Menu;
use std::collections::{HashMap, VecDeque};

// 處理用戶輸入的結構體
pub struct InputHandler {
    pub input: String,      // 當前輸入緩衝區
    pub buffer: Vec<String>, // 儲存所有已提交的文本
    pub last_command: Option<String>, // 儲存上一次的命令
    pub command_history: VecDeque<String>, // 命令歷史記錄隊列
    pub max_history: usize, // 最大歷史記錄數量
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
            last_command: None,
            command_history: VecDeque::new(),
            max_history: 100,
        }
    }

    // 將按鍵事件轉換為指令字串（主輸入處理）
    pub fn handle_input_events(&mut self, key: KeyEvent, context: &mut AppContext) -> Option<CommandResult> {
        // 優先處理互動選單（交易、對話等）
        if context.interaction_menu.is_some() {
            // 互動選單使用按鍵導航
            return self.handle_interaction_menu(key, context);
        }
        
        // If menu is open, handle menu input first
        if context.menu.is_some() {
            // 一般選單使用按鍵導航
            return self.handle_context_menu(key, context);
        }

        // 處理特殊按鍵（F1, PageUp/Down, Shift+方向鍵等）
        if let Some(result) = self.handle_normal_keyevent(key, context) {
            return Some(result);
        }

        // 正常狀態：將按鍵轉換為指令字串
        if let Some(command_str) = self.key_to_command_string(key, context) {
            return self.process_command_string(command_str);
        }
        
        None
    }
    
    // 將按鍵轉換為指令字串
    fn key_to_command_string(&mut self, key: KeyEvent, context: &mut AppContext) -> Option<String> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        
        match key.code {
            KeyCode::Up => Some("up".to_string()),
            KeyCode::Down => Some("down".to_string()),
            KeyCode::Left => Some("left".to_string()),
            KeyCode::Right => Some("right".to_string()),
            KeyCode::Esc => {
                // 處理 Esc 鍵開關選單
                if context.menu.is_none() {
                    let mut new_menu = Menu::new(
                        "遊戲選單".to_string(),
                        vec![
                            "繼續遊戲".to_string(),
                            "儲存遊戲".to_string(),
                            "載入遊戲".to_string(),
                            "設定".to_string(),
                            "離開遊戲".to_string(),
                        ],
                    );
                    new_menu.activate();
                    *context.menu = Some(new_menu);
                    context.output_manager.print("選單開啟".to_string());
                } else {
                    *context.menu = None;
                    context.output_manager.print("選單關閉".to_string());
                }
                None
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                None
            }
            KeyCode::Backspace => {
                self.input.pop();
                None
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    let cmd = self.input.clone();
                    self.input.clear();
                    Some(cmd)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn handle_interaction_menu(&mut self, key: KeyEvent, context: &mut AppContext) -> Option<CommandResult> {
        let interaction_menu = context.interaction_menu.as_mut()?;
        
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Up => interaction_menu.previous(),
                KeyCode::Down => interaction_menu.next(),
                KeyCode::Enter => {
                    if let Some(selected_item) = interaction_menu.get_selected_item().cloned() {
                        let state = context.game_world.interaction_state.clone();
                        
                        interaction_menu.deactivate();
                        *context.interaction_menu = None;
                        
                        return match state {
                            crate::world::InteractionState::Trading { npc_name } => {
                                self.handle_trading_state(context, &selected_item, &npc_name)
                            },
                            crate::world::InteractionState::Buying { npc_name } => {
                                self.handle_buying_state(context, &selected_item, &npc_name)
                            },
                            crate::world::InteractionState::Selling { npc_name } => {
                                self.handle_selling_state(context, &selected_item, &npc_name)
                            },
                            crate::world::InteractionState::None => None,
                        };
                    }
                },
                KeyCode::Esc => {
                    self.cancel_interaction(context);
                },
                _ => {}
            }
        }
        None
    }

    fn handle_trading_state(&mut self, context: &mut AppContext, selected_item: &str, npc_name: &str) -> Option<CommandResult> {
        if selected_item == "購買物品" {
            context.game_world.interaction_state = 
                crate::world::InteractionState::Buying { npc_name: npc_name.to_string() };
            Some(CommandResult::Trade(npc_name.to_string()))
        } else if selected_item == "出售物品" {
            context.game_world.interaction_state = 
                crate::world::InteractionState::Selling { npc_name: npc_name.to_string() };
            Some(CommandResult::Trade(npc_name.to_string()))
        } else if selected_item == "離開" {
            context.game_world.interaction_state = crate::world::InteractionState::None;
            if let Some(npc) = context.game_world.npc_manager.get_npc_mut(npc_name) {
                npc.is_interacting = false;
            }
            context.output_manager.print("結束交易".to_string());
            None
        } else {
            None
        }
    }

    fn handle_buying_state(&mut self, context: &mut AppContext, selected_item: &str, npc_name: &str) -> Option<CommandResult> {
        if selected_item == "返回" {
            context.game_world.interaction_state = 
                crate::world::InteractionState::Trading { npc_name: npc_name.to_string() };
            Some(CommandResult::Trade(npc_name.to_string()))
        } else if let Some((item_part, _)) = selected_item.split_once(" x") {
            let item_name = if let Some((chinese_name, _)) = item_part.split_once(" (") {
                chinese_name.trim()
            } else {
                item_part.trim()
            };
            
            let resolved_item = crate::item_registry::resolve_item_name(item_name);
            Some(CommandResult::Buy(npc_name.to_string(), resolved_item, 1))
        } else {
            None
        }
    }

    fn handle_selling_state(&mut self, context: &mut AppContext, selected_item: &str, npc_name: &str) -> Option<CommandResult> {
        if selected_item == "返回" {
            context.game_world.interaction_state = 
                crate::world::InteractionState::Trading { npc_name: npc_name.to_string() };
            Some(CommandResult::Trade(npc_name.to_string()))
        } else if let Some((item_part, _)) = selected_item.split_once(" x") {
            let item_name = if let Some((chinese_name, _)) = item_part.split_once(" (") {
                chinese_name.trim()
            } else {
                item_part.trim()
            };
            
            let resolved_item = crate::item_registry::resolve_item_name(item_name);
            Some(CommandResult::Sell(npc_name.to_string(), resolved_item, 1))
        } else {
            context.game_world.interaction_state = crate::world::InteractionState::None;
            if let Some(npc) = context.game_world.npc_manager.get_npc_mut(npc_name) {
                npc.is_interacting = false;
            }
            None
        }
    }

    fn cancel_interaction(&mut self, context: &mut AppContext) {
        let state = context.game_world.interaction_state.clone();
        
        match state {
            crate::world::InteractionState::Trading { npc_name } |
            crate::world::InteractionState::Buying { npc_name } |
            crate::world::InteractionState::Selling { npc_name } => {
                if let Some(npc) = context.game_world.npc_manager.get_npc_mut(&npc_name) {
                    npc.is_interacting = false;
                }
            },
            crate::world::InteractionState::None => {},
        }
        
        context.output_manager.print("取消交易".to_string());
        context.game_world.interaction_state = crate::world::InteractionState::None;
        
        if let Some(menu) = context.interaction_menu.as_mut() {
            menu.deactivate();
        }
        *context.interaction_menu = None;
    }

    fn handle_context_menu(&mut self, key: KeyEvent, context: &mut AppContext) -> Option<CommandResult> {
        let active_menu = context.menu.as_mut()?;
        
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Up => active_menu.previous(),
                KeyCode::Down => active_menu.next(),
                KeyCode::Enter => {
                    if let Some(selected_item) = active_menu.get_selected_item() {
                        context.output_manager.print(format!("選單確認: {selected_item}"));
                        if selected_item == "離開遊戲" {
                            *context.should_exit = true;
                        }
                    }
                    active_menu.deactivate();
                    *context.menu = None;
                },
                KeyCode::Esc | KeyCode::Char('q' | 'Q') => {
                    context.output_manager.print("選單取消".to_string());
                    active_menu.deactivate();
                    *context.menu = None;
                },
                _ => {}
            }
        }
        None
    }

    fn handle_normal_keyevent(&mut self, key: KeyEvent, context: &mut AppContext) -> Option<CommandResult> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        match key.code {
            KeyCode::F(1) => {
                context.output_manager.toggle_status_panel();
                None
            },
            KeyCode::Char('q' | 'Q') => {
                if context.output_manager.is_map_open() {
                    context.output_manager.close_map();
                    context.output_manager.set_status("大地圖已關閉".to_string());
                    None
                } else {
                    // 'q' 字符會被轉為指令字串
                    None
                }
            },
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    match key.code {
                        KeyCode::Up => {
                            context.output_manager.scroll_up();
                            context.output_manager.set_status("向上捲動訊息".to_string());
                        },
                        KeyCode::Down => {
                            context.output_manager.scroll_down(20);
                            context.output_manager.set_status("向下捲動訊息".to_string());
                        },
                        _ => {}
                    }
                    None
                } else if context.output_manager.is_map_open() {
                    if let Some(current_map_data) = context.game_world.get_current_map() {
                        let (dx, dy) = match key.code {
                            KeyCode::Up => (0, -5),
                            KeyCode::Down => (0, 5),
                            KeyCode::Left => (-5, 0),
                            KeyCode::Right => (5, 0),
                            _ => (0, 0),
                        };
                        context.output_manager.move_map_view(dx, dy, current_map_data.width, current_map_data.height);
                    }
                    None
                } else {
                    // 方向鍵會被轉為指令字串（在 key_to_command_string 處理）
                    None
                }
            },
            KeyCode::PageUp => {
                context.output_manager.scroll_up();
                context.output_manager.set_status("向上捲動訊息".to_string());
                None
            },
            KeyCode::PageDown => {
                context.output_manager.scroll_down(20);
                context.output_manager.set_status("向下捲動訊息".to_string());
                None
            },
            _ => None,
        }
    }


    // 處理指令字串（新核心方法）
    pub fn process_command_string(&mut self, command_str: String) -> Option<CommandResult> {
        // 處理特殊指令
        let result = match command_str.as_str() {
            "up" => CommandResult::Move(0, -1),
            "down" => CommandResult::Move(0, 1),
            "left" => CommandResult::Move(-1, 0),
            "right" => CommandResult::Move(1, 0),
            _ => {
                // 一般文字指令
                self.parse_input(command_str.clone())
            }
        };
        
        // 保存指令到歷史記錄
        if command_str != "re" && command_str != "repeat" {
            if !matches!(result, CommandResult::Error(_)) {
                self.last_command = Some(command_str.clone());
                self.add_to_history(command_str);
            }
        }
        
        Some(result)
    }
    
    // 添加指令到歷史記錄
    fn add_to_history(&mut self, command: String) {
        // 如果超過最大數量，移除最舊的
        if self.command_history.len() >= self.max_history {
            self.command_history.pop_front();
        }
        self.command_history.push_back(command);
    }
    
    // 獲取指令歷史記錄
    #[allow(dead_code)]
    pub fn get_history(&self) -> &VecDeque<String> {
        &self.command_history
    }
    
    // 獲取最近的 N 條指令
    #[allow(dead_code)]
    pub fn get_recent_commands(&self, count: usize) -> Vec<String> {
        self.command_history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    // 獲取歷史記錄數量
    pub fn history_count(&self) -> usize {
        self.command_history.len()
    }

    // 取得目前輸入的文本
    pub fn get_input(&self) -> &str {
        &self.input
    }

    // 清除目前輸入的文本
    #[allow(dead_code)]
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    // 解析輸入內容（使用字串輸入）
    fn parse_input(&mut self, input: String) -> CommandResult {
        self.handle_command(input)
    }

    // 處理命令（所有輸入都是命令，不需要 / 前綴）
    /// 
    /// 【主命令處理器】此函數是主程式（app.rs）實際使用的命令解析器
    /// 
    /// 執行流程：
    /// 1. 分割輸入為 parts（以空白分隔）
    /// 2. 根據第一個 part 判斷命令類型
    /// 3. 解析參數並返回對應的 CommandResult
    /// 
    /// 【與其他命令處理器的關係】
    /// ┌──────────────────────────────────────────────────────────────┐
    /// │ InputHandler::handle_command() [此函數] - 主程式使用          │
    /// │ - 支援所有最新命令（40+ 個）                                  │
    /// │ - 包含：give, re, talk, check, quest, 等                     │
    /// │ - 持續更新維護                                                │
    /// └──────────────────────────────────────────────────────────────┘
    /// 
    /// vs
    /// 
    /// ┌──────────────────────────────────────────────────────────────┐
    /// │ CommandProcessor::parse_command() - FFI 使用                  │
    /// │ - 支援基本命令（約 30 個）                                     │
    /// │ - 較簡化，可能缺少最新功能                                     │
    /// │ - 位於：src/command_processor.rs                             │
    /// └──────────────────────────────────────────────────────────────┘
    /// 
    /// 【特殊功能】
    /// - "re" / "repeat": 重複上一次成功的命令（遞迴呼叫 handle_command）
    /// - 自動保存成功的命令到 self.last_command
    /// 
    /// 【待重構】未來可考慮：
    /// 1. 將命令解析邏輯抽取到獨立模組
    /// 2. 使用 CommandProcessor 或統一到此函數
    /// 3. 減少程式碼重複
    fn handle_command(&mut self, input: String) -> CommandResult {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return CommandResult::Error("No command provided".to_string());
        }

        // 先檢查是否為 status 相關命令（這些命令不應關閉 status）
        let _is_status_command = matches!(parts[0], "status" | "i" | "show" | "s" if parts.len() == 1 && (parts[0] == "status" || parts[0] == "i") || (parts.len() > 1 && parts[1] == "status"));
        
        let result = match parts[0] {
            "re" | "repeat" => {
                // 重複上一次的命令
                if let Some(ref last_cmd) = self.last_command {
                    return self.handle_command(last_cmd.clone());
                }
                CommandResult::Error("沒有可重複的命令".to_string())
            },
            "history" | "hist" => {
                // 顯示指令歷史記錄
                // history [n] - 顯示最近 n 條指令（預設 10）
                let count = if parts.len() > 1 {
                    parts[1].parse::<usize>().unwrap_or(10).min(50)
                } else {
                    10
                };
                CommandResult::ShowHistory(count)
            },
            "exit" | "quit" => CommandResult::Exit,
            "help" => CommandResult::Help,
            "save" => {
                // save [filename] 命令，預設檔名為 save.txt
                let filename = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "save.txt".to_string());
                self.execute_save(&filename)
            },
            "clear" => CommandResult::Clear,
            "status" | "i" => {
                // status/i 命令，顯示玩家詳細資訊（重用 check me 功能）
                CommandResult::CheckNpc("me".to_string())
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
            "use" => {
                // use 命令，使用物品
                // use <物品名稱>
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
                    CommandResult::Error("Usage: dream [content]".to_string())
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
                // set 命令的多種用法：
                // 1. set item <物品名稱> <價格> - 設置物品價格
                // 2. set <目標人物> <屬性> <數值> - 設置角色屬性
                //    支持屬性: hp, mp, strength, knowledge, sociality, gold/金幣
                if parts.len() < 4 {
                    CommandResult::Error("Usage: set <目標人物> <屬性> <數值> 或 set item <物品名稱> <價格>".to_string())
                } else {
                    // 檢查是否為設置物品價格
                    if parts[1].to_lowercase() == "item" {
                        if parts.len() < 4 {
                            CommandResult::Error("Usage: set item <物品名稱> <價格>".to_string())
                        } else {
                            let item_name = parts[2].to_string();
                            let price = parts[3].parse::<i32>().unwrap_or(0);
                            // 使用特殊格式：target="item", attribute=物品名稱, value=價格
                            CommandResult::Set("item".to_string(), item_name, price)
                        }
                    } else {
                        // 設置角色屬性
                        let target = parts[1].to_string();
                        let attribute = parts[2].to_string();
                        let value = parts[3].parse::<i32>().unwrap_or(0);
                        CommandResult::Set(target, attribute, value)
                    }
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
            "give" => {
                // give <npc> <item> [quantity] 命令，給予 NPC 物品
                if parts.len() < 3 {
                    CommandResult::Error("Usage: give <npc> <item> [quantity]".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let item = parts[2].to_string();
                    let quantity = if parts.len() > 3 {
                        parts[3].parse::<u32>().unwrap_or(1)
                    } else {
                        1
                    };
                    CommandResult::Give(npc, item, quantity)
                }
            },
            "setdialogue" | "setdia" | "sdl" => {
                // setdialogue 命令的多種用法：
                // 1. sdl <npc> <話題> add <對話> when <條件> - 新增帶條件的對話
                // 2. sdl <npc> set <話題> when <條件> say <對話> - 設定條件式對話（更直觀）
                // 3. sdl <npc> <話題> <對話> - 簡單版（無條件）
                // 範例: 
                //   sdl sakura 閒聊 add 你長得好漂亮啊 when 顏值>80 and 性別=女
                //   sdl ammy set 閒聊 when 力量>100 and 顏值>80 say 你真是又帥又厲害
                //   sdl 商人 見面 哈囉！你好，來看看我的商品
                if parts.len() < 4 {
                    CommandResult::Error("Usage: sdl <npc> <話題> <對話> 或 sdl <npc> add/set <話題> when <條件> say <對話>".to_string())
                } else {
                    let npc = parts[1].to_string();
                    
                    // 檢查是否使用 "set" 語法
                    if parts[2] == "set" {
                        // sdl <npc> set <話題> when <條件> say <對話>
                        if parts.len() < 6 {
                            CommandResult::Error("Usage: sdl <npc> set <話題> when <條件> say <對話>".to_string())
                        } else {
                            let topic = parts[3].to_string();
                            
                            if let Some(when_pos) = parts.iter().position(|&p| p == "when") {
                                if let Some(say_pos) = parts.iter().position(|&p| p == "say") {
                                    let conditions_str = parts[when_pos+1..say_pos].join(" ");
                                    let dialogue = parts[say_pos+1..].join(" ");
                                    CommandResult::SetDialogueWithConditions(npc, topic, dialogue, conditions_str)
                                } else {
                                    CommandResult::Error("缺少 'say' 關鍵字".to_string())
                                }
                            } else {
                                CommandResult::Error("缺少 'when' 關鍵字".to_string())
                            }
                        }
                    } 
                    // 檢查是否使用 "add" 語法
                    else if parts[3] == "add" {
                        // sdl <npc> <話題> add <對話> when <條件>
                        let topic = parts[2].to_string();
                        if let Some(when_pos) = parts.iter().position(|&p| p == "when") {
                            let dialogue = parts[4..when_pos].join(" ");
                            let conditions_str = parts[when_pos+1..].join(" ");
                            CommandResult::SetDialogueWithConditions(npc, topic, dialogue, conditions_str)
                        } else {
                            // 只有 add，沒有 when
                            let dialogue = parts[4..].join(" ");
                            CommandResult::SetDialogue(npc, topic, dialogue)
                        }
                    } else {
                        // 簡單版本（無條件）
                        let topic = parts[2].to_string();
                        let dialogue = parts[3..].join(" ");
                        CommandResult::SetDialogue(npc, topic, dialogue)
                    }
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
            "setrelationship" | "setrel" => {
                // setrelationship <npc> <好感度> 命令，設置 NPC 好感度 (-100~100)
                // 範例: setrelationship 商人 50
                if parts.len() < 3 {
                    CommandResult::Error("Usage: setrelationship <npc> <好感度(-100~100)>".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let relationship = parts[2].parse::<i32>().unwrap_or(0).clamp(-100, 100);
                    CommandResult::SetRelationship(npc, relationship)
                }
            },
            "changerelationship" | "changerel" | "addrel" => {
                // changerelationship <npc> <變化量> 命令，改變 NPC 好感度
                // 範例: changerelationship 商人 10
                if parts.len() < 3 {
                    CommandResult::Error("Usage: changerelationship <npc> <變化量>".to_string())
                } else {
                    let npc = parts[1].to_string();
                    let delta = parts[2].parse::<i32>().unwrap_or(0);
                    CommandResult::ChangeRelationship(npc, delta)
                }
            },
            "talk" | "speak" => {
                // talk <npc> [話題] 命令，與 NPC 對話
                // 範例: talk 商人 閒聊
                if parts.len() < 2 {
                    CommandResult::Error("Usage: talk <npc> [話題]".to_string())
                } else {
                    let npc_name = parts[1].to_string();
                    let topic = if parts.len() >= 3 {
                        parts[2..].join(" ")
                    } else {
                        "閒聊".to_string()
                    };
                    CommandResult::Talk(npc_name, topic)
                }
            },
            "wait" => {
                // wait <npc> 命令，叫住 NPC（根據好感度判斷是否成功）
                // 範例: wait 商人
                if parts.len() < 2 {
                    CommandResult::Wait("".to_string())
                } else {
                    let npc_name = parts[1].to_string();
                    CommandResult::Wait(npc_name)
                }
            },
            "party" => {
                // party <npc> 命令，邀請 NPC 組隊
                if parts.len() < 2 {
                    CommandResult::Error("Usage: party <npc>".to_string())
                } else {
                    let npc_name = parts[1].to_string();
                    CommandResult::Party(npc_name)
                }
            },
            "disband" => {
                // disband 命令，解散隊伍
                CommandResult::Disband
            },
            "punch" | "ph" => {
                // punch/ph [目標] 命令，使用拳擊
                if parts.len() < 2 {
                    CommandResult::Punch(None)
                } else {
                    CommandResult::Punch(Some(parts[1].to_string()))
                }
            },
            "kick" | "kk" => {
                // kick/kk [目標] 命令，使用踢擊
                if parts.len() < 2 {
                    CommandResult::Kick(None)
                } else {
                    CommandResult::Kick(Some(parts[1].to_string()))
                }
            },
            "escape" | "esc" => {
                // escape/esc 命令，逃離戰鬥
                CommandResult::Escape
            },
            "check" | "inspect" | "examine" => {
                // check <npc> 命令，查看 NPC 的詳細資訊
                if parts.len() < 2 {
                    CommandResult::CheckNpc("me".to_string())
                } else {
                    let npc_name = parts[1..].join(" ");
                    CommandResult::CheckNpc(npc_name)
                }
            },
            "quest" => {
                // quest 命令系列
                if parts.len() < 2 {
                    CommandResult::QuestList
                } else {
                    match parts[1] {
                        "list" | "all" => CommandResult::QuestList,
                        "active" | "current" => CommandResult::QuestActive,
                        "available" | "avail" => CommandResult::QuestAvailable, // Changed from avail to available
                        "completed" | "done" => CommandResult::QuestCompleted,
                        "info" | "show" => {
                            if parts.len() < 3 {
                                CommandResult::Error("Usage: quest info <任務ID>".to_string())
                            } else {
                                CommandResult::QuestInfo(parts[2].to_string())
                            }
                        },
                        "start" | "accept" => {
                            if parts.len() < 3 {
                                CommandResult::Error("Usage: quest start <任務ID>".to_string())
                            } else {
                                CommandResult::QuestStart(parts[2].to_string())
                            }
                        },
                        "complete" | "finish" => {
                            if parts.len() < 3 {
                                CommandResult::Error("Usage: quest complete <任務ID>".to_string())
                            } else {
                                CommandResult::QuestComplete(parts[2].to_string())
                            }
                        },
                        "abandon" | "cancel" => {
                            if parts.len() < 3 {
                                CommandResult::Error("Usage: quest abandon <任務ID>".to_string())
                            } else {
                                CommandResult::QuestAbandon(parts[2].to_string())
                            }
                        },
                        _ => CommandResult::Error(format!("Unknown quest subcommand: {}", parts[1])),
                    }
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
                    Ok(_) => CommandResult::Output(format!("Saved {} lines to {}", self.buffer.len(), filename)),
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