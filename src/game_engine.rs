use crate::world::GameWorld;
use crate::person::Person;
use crate::command_processor::CommandProcessor;
use crate::input::CommandResult;
use std::collections::VecDeque;

/// 無頭遊戲引擎（不依賴終端 UI）
#[allow(dead_code)]
pub struct GameEngine {
    pub world: GameWorld,
    pub player: Person,
    processor: CommandProcessor,
    output_buffer: VecDeque<String>,
    max_output_buffer: usize,
}

#[allow(dead_code)]
impl GameEngine {
    /// 創建新的遊戲引擎
    pub fn new(player_name: &str, player_desc: &str) -> Self {
        let player = Person::new(player_name.to_string(), player_desc.to_string());
        let world = GameWorld::new(player.clone());
        
        GameEngine {
            world,
            player,
            processor: CommandProcessor::new(),
            output_buffer: VecDeque::new(),
            max_output_buffer: 1000,
        }
    }
    
    /// 處理文本命令
    /// 返回：(是否繼續遊戲, 命令結果描述)
    pub fn process_command(&mut self, command: &str) -> (bool, String) {
        // 解析命令
        let cmd_result = self.processor.parse_command(command);
        
        // 檢查是否退出
        if matches!(cmd_result, CommandResult::Exit) {
            self.add_output("再見！感謝遊玩。".to_string());
            return (false, "退出遊戲".to_string());
        }
        
        // 執行命令
        let result_msg = self.execute_command(cmd_result);
        self.add_output(result_msg.clone());
        
        (true, result_msg)
    }
    
    /// 執行命令並返回結果訊息
    fn execute_command(&mut self, cmd: CommandResult) -> String {
        match cmd {
            CommandResult::Move(dx, dy) => {
                let new_x = (self.player.x as i32 + dx).max(0) as usize;
                let new_y = (self.player.y as i32 + dy).max(0) as usize;
                
                self.player.x = new_x;
                self.player.y = new_y;
                
                format!("移動到 ({new_x}, {new_y})")
            }
            
            CommandResult::Look(target) => {
                let map_name = self.world.current_map_name.clone();
                let pos = format!("({}, {})", self.player.x, self.player.y);
                
                if let Some(t) = target {
                    format!("你查看 {t}...")
                } else {
                    format!("你環顧四周...\n地圖: {}\n位置: {}\nHP: {}", 
                           map_name, pos, self.player.hp)
                }
            }
            
            CommandResult::ShowStatus => {
                format!("玩家: {}\nHP: {}/{}\nMP: {}/{}\n位置: ({}, {})\n地圖: {}",
                       self.player.name,
                       self.player.hp,
                       self.player.max_hp,
                       self.player.mp,
                       self.player.max_mp,
                       self.player.x,
                       self.player.y,
                       self.world.current_map_name.clone())
            }
            
            CommandResult::ShowMap => {
                format!("當前地圖: {}", self.world.current_map_name.clone())
            }
            
            CommandResult::ShowMinimap => {
                "小地圖已開啟".to_string()
            }
            
            CommandResult::HideMinimap => {
                "小地圖已關閉".to_string()
            }
            
            CommandResult::ShowLog => {
                "日誌已開啟".to_string()
            }
            
            CommandResult::HideLog => {
                "日誌已關閉".to_string()
            }
            
            CommandResult::ShowWorld => {
                format!("世界: {}", self.world.current_map_name.clone())
            }
            
            CommandResult::Clear => {
                self.output_buffer.clear();
                "畫面已清除".to_string()
            }
            
            CommandResult::Help => {
                self.get_help_text()
            }
            
            CommandResult::Get(item, qty) => {
                if let Some(item_name) = item {
                    format!("拾取 {item_name} x{qty}")
                } else {
                    "拾取物品".to_string()
                }
            }
            
            CommandResult::Drop(item, qty) => {
                format!("丟棄 {item} x{qty}")
            }
            
            CommandResult::Eat(food) => {
                format!("吃掉 {food}")
            }
            
            CommandResult::Sleep => {
                "開始睡覺...".to_string()
            }
            
            CommandResult::Dream(target) => {
                if let Some(t) = target {
                    format!("夢見 {t}...")
                } else {
                    "做了一個夢...".to_string()
                }
            }
            
            CommandResult::WakeUp => {
                "醒來了！".to_string()
            }
            
            CommandResult::Summon(npc_name) => {
                format!("召喚 {npc_name}")
            }
            
            CommandResult::Conquer(place) => {
                format!("征服 {place}")
            }
            
            CommandResult::FlyTo(place) => {
                format!("飛往 {place}")
            }
            
            CommandResult::NameHere(new_name) => {
                format!("將此地命名為 {new_name}")
            }
            
            CommandResult::Name(old_name, new_name) => {
                format!("將 {old_name} 重命名為 {new_name}")
            }
            
            CommandResult::Create(type_str, name, desc) => {
                if let Some(d) = desc {
                    format!("創建 {type_str} '{name}': {d}")
                } else {
                    format!("創建 {type_str} '{name}'")
                }
            }
            
            CommandResult::Destroy(target) => {
                format!("摧毀 {target}")
            }
            
            CommandResult::Set(npc, attr, value) => {
                format!("設置 {npc} 的 {attr} 為 {value}")
            }
            
            CommandResult::ListNpcs => {
                let npcs = self.world.npc_manager.get_npcs_at_in_map(
                    &self.world.current_map_name,
                    self.player.x,
                    self.player.y
                );
                
                if npcs.is_empty() {
                    "附近沒有 NPC".to_string()
                } else {
                    let mut result = "附近的 NPC:\n".to_string();
                    for npc in npcs {
                        result.push_str(&format!("  - {} ({})\n", npc.name, npc.description));
                    }
                    result
                }
            }
            
            CommandResult::SwitchControl(npc_name) => {
                format!("切換控制到 {npc_name}")
            }
            
            CommandResult::Trade(npc_name) => {
                format!("與 {npc_name} 交易")
            }
            
            CommandResult::Buy(npc, item, qty) => {
                format!("從 {npc} 購買 {item} x{qty}")
            }
            
            CommandResult::Sell(npc, item, qty) => {
                format!("向 {npc} 出售 {item} x{qty}")
            }
            
            CommandResult::SetDialogue(npc, scene, dialogue) => {
                format!("設置 {npc} 在 {scene} 場景的對話: {dialogue}")
            }
            
            CommandResult::SetEagerness(npc, eagerness) => {
                format!("設置 {npc} 的熱情度為 {eagerness}")
            }
            
            CommandResult::SetRelationship(npc, relationship) => {
                format!("設置 {npc} 的好感度為 {relationship}")
            }
            
            CommandResult::ChangeRelationship(npc, delta) => {
                format!("改變 {npc} 的好感度 {delta:+}")
            }
            
            CommandResult::Talk(npc_name, topic) => {
                // 與 NPC 對話
                if let Some(npc) = self.world.npc_manager.get_npc(&npc_name) {
                    format!("與 {} 聊「{}」...", npc.name, topic)
                } else {
                    format!("找不到 NPC: {npc_name}")
                }
            }
            
            CommandResult::CheckNpc(npc_name) => {
                // 查找 NPC 並顯示詳細資訊
                if let Some(npc) = self.world.npc_manager.get_npc(&npc_name) {
                    npc.show_detail()
                } else {
                    format!("找不到 NPC: {npc_name}")
                }
            }
            
            CommandResult::SetDialogueWithConditions(npc, topic, _dialogue, conditions) => {
                format!("設置 {npc} 的「{topic}」對話（條件: {conditions}）")
            }
            
            CommandResult::ToggleTypewriter => {
                "切換打字機效果".to_string()
            }
            
            // 任務系統
            CommandResult::QuestList => "顯示所有任務".to_string(),
            CommandResult::QuestActive => "顯示進行中的任務".to_string(),
            CommandResult::QuestAvailable => "顯示可接取的任務".to_string(),
            CommandResult::QuestCompleted => "顯示已完成的任務".to_string(),
            CommandResult::QuestInfo(quest_id) => format!("查看任務: {quest_id}"),
            CommandResult::QuestStart(quest_id) => format!("開始任務: {quest_id}"),
            CommandResult::QuestComplete(quest_id) => format!("完成任務: {quest_id}"),
            CommandResult::QuestAbandon(quest_id) => format!("放棄任務: {quest_id}"),
            
            CommandResult::Output(msg) => {
                msg
            }
            
            CommandResult::AddToSide(msg) => {
                msg
            }
            
            CommandResult::Error(msg) => {
                format!("錯誤: {msg}")
            }
            
            CommandResult::Exit => {
                unreachable!("Exit should be handled earlier")
            }
        }
    }
    
    /// 添加輸出到緩衝區
    fn add_output(&mut self, message: String) {
        // 觸發回調
        crate::callback::trigger_output_callback(&message);
        
        // 添加到緩衝區
        self.output_buffer.push_back(message);
        
        // 限制緩衝區大小
        while self.output_buffer.len() > self.max_output_buffer {
            self.output_buffer.pop_front();
        }
    }
    
    /// 獲取所有輸出（並清空緩衝區）
    pub fn get_output(&mut self) -> Vec<String> {
        let output: Vec<String> = self.output_buffer.drain(..).collect();
        output
    }
    
    /// 查看輸出（不清空緩衝區）
    pub fn peek_output(&self) -> Vec<String> {
        self.output_buffer.iter().cloned().collect()
    }
    
    /// 獲取最後 n 條輸出
    pub fn get_last_output(&self, n: usize) -> Vec<String> {
        self.output_buffer.iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .collect()
    }
    
    /// 獲取幫助文本
    fn get_help_text(&self) -> String {
        "可用命令:\n\
         移動: up/down/left/right, move <x> <y>\n\
         查看: look, status, map\n\
         物品: get, drop, eat\n\
         休息: sleep, dream, wakeup\n\
         NPC: summon, ctrl, trade, buy, sell, npcs\n\
         地圖: map, minimap, hidemap\n\
         系統: help, clear, quit\n".to_string()
    }
    
    /// 獲取遊戲狀態（JSON 格式）
    pub fn get_state_json(&self) -> String {
        serde_json::json!({
            "player": {
                "name": self.player.name,
                "hp": self.player.hp,
                "max_hp": self.player.max_hp,
                "mp": self.player.mp,
                "max_mp": self.player.max_mp,
                "position": [self.player.x, self.player.y],
                "status": self.player.status,
            },
            "world": {
                "current_map": self.world.current_map_name,
            }
        }).to_string()
    }
    
    /// 更新遊戲邏輯（每幀調用）
    pub fn update(&mut self, _delta_ms: u32) {
        // TODO: 更新 NPC AI, 事件系統等
    }
}
