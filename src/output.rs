use ratatui::text::{Line, Text, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::layout::{Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use std::time::{Instant, Duration};
use crate::observable::{Observable, Empty};

// 管理輸出訊息和滾動位置的結構體
pub struct OutputManager {
    messages: Vec<String>,      // 儲存所有輸出訊息
    scroll: usize,              // 目前滾動位置
    status: String,             // 狀態列訊息
    status_time: Option<Instant>, // 狀態訊息的時間戳
    side_messages: Vec<String>, // 儲存側邊輸出訊息
    side_scroll: usize,         // 側邊輸出的滾動位置
    show_side_panel: bool,      // 是否顯示側邊面板
    side_observable: Box<dyn Observable>, // 側邊面板的可觀察對象
    current_time: String,       // 當前遊戲時間顯示
    show_minimap: bool,         // 是否顯示小地圖
    minimap_lines: Vec<String>, // 小地圖的行內容
    log_messages: Vec<String>,  // 系統日誌訊息
    log_scroll: usize,          // 日誌滾動位置
    show_log: bool,             // 是否顯示日誌視窗
    show_map: bool,             // 是否顯示大地圖
    map_offset_x: usize,        // 大地圖顯示的偏移量 X
    map_offset_y: usize,        // 大地圖顯示的偏移量 Y
}

impl OutputManager {
    // 建立新的輸出管理器
    pub fn new() -> Self {
        OutputManager {
            messages: Vec::new(),
            scroll: 0,
            status: String::new(),
            status_time: None,
            side_messages: Vec::new(),
            side_scroll: 0,
            show_side_panel: false,
            side_observable: Box::new(Empty),
            current_time: String::from("Day 1 09:00:00"),
            show_minimap: false,
            minimap_lines: Vec::new(),
            log_messages: Vec::new(),
            log_scroll: 0,
            show_log: true,  // 預設顯示日誌視窗
            show_map: false,
            map_offset_x: 0,
            map_offset_y: 0,
        }
    }

    // 添加訊息並將滾動位置移到最後（僅儲存純文本）
    pub fn print(&mut self, message: String) {
        self.messages.push(message);
        // 將 scroll 設為一個很大的值，render_output 會自動限制它
        self.scroll = usize::MAX;
    }

    // 設定狀態列訊息（5秒後自動清除）
    pub fn set_status(&mut self, status: String) {
        self.status = status;
        self.status_time = Some(Instant::now());
    }

    // 更新狀態列（檢查是否超過5秒）
    pub fn update_status(&mut self) {
        if let Some(time) = self.status_time {
            if time.elapsed() > Duration::from_secs(5) {
                self.status.clear();
                self.status_time = None;
            }
        }
    }

    // 獲取當前狀態（如果已過期則返回空字串）
    #[allow(dead_code)]
    pub fn get_status(&self) -> String {
        if let Some(time) = self.status_time {
            if time.elapsed() > Duration::from_secs(5) {
                String::new()
            } else {
                self.status.clone()
            }
        } else {
            self.status.clone()
        }
    }

    // 設置當前時間顯示
    pub fn set_current_time(&mut self, time: String) {
        self.current_time = time;
    }

    // 清除所有訊息
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll = 0;
    }

    // 向上滾動
    #[allow(dead_code)]
    pub fn scroll_up(&mut self) {
        // 先確保 scroll 不會超過合理範圍
        if self.scroll > self.messages.len() {
            self.scroll = self.messages.len().saturating_sub(1);
        }
        // 一次向上捲動 5 行，讓效果更明顯
        self.scroll = self.scroll.saturating_sub(5);
    }

    // 向下滾動（受可見高度限制）
    #[allow(dead_code)]
    pub fn scroll_down(&mut self, visible_height: usize) {
        // 先確保 scroll 不會超過合理範圍
        if self.scroll > self.messages.len() {
            self.scroll = self.messages.len().saturating_sub(1);
        }
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        // 一次向下捲動 5 行，讓效果更明顯
        self.scroll = (self.scroll + 5).min(max_scroll);
    }

    // 渲染輸出區域的小部件
    pub fn render_output(&self, area: Rect) -> Paragraph {
        let message_area_height = area.height.saturating_sub(2) as usize;
        let total_messages = self.messages.len();
        let max_scroll = total_messages.saturating_sub(message_area_height);
        let scroll = self.scroll.min(max_scroll);

        // 計算可見的訊息範圍
        let visible_messages = if total_messages > message_area_height {
            &self.messages[scroll..scroll + message_area_height.min(total_messages - scroll)]
        } else {
            &self.messages[..]
        };

        // 將訊息轉換為渲染線條
        let message_lines: Vec<Line> = visible_messages
            .iter()
            .map(|m| Line::from(m.as_str()))
            .collect();

        // 建立帶邊框的段落小部件
        Paragraph::new(Text::from(message_lines))
            .block(Block::default().title("*").borders(Borders::ALL))
    }

    // 渲染狀態列（只顯示臨時狀態訊息）
    pub fn render_status(&self) -> Paragraph {
        let status_text = if let Some(time) = self.status_time {
            if time.elapsed() > Duration::from_secs(5) {
                String::new()  // 狀態過期後顯示空白
            } else {
                self.status.clone()
            }
        } else {
            String::new()
        };

        let status_span = Span::styled(
            status_text,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM)
        );
        Paragraph::new(Line::from(status_span))
            .alignment(Alignment::Left)
    }

    // 添加側邊訊息
    pub fn add_side_message(&mut self, message: String) {
        self.side_messages.push(message);
        self.side_scroll = self.side_messages.len().saturating_sub(1);
    }

    // 清除側邊訊息
    #[allow(dead_code)]
    pub fn clear_side_messages(&mut self) {
        self.side_messages.clear();
        self.side_scroll = 0;
    }

    // 切換側邊面板顯示狀態
    pub fn toggle_side_panel(&mut self) {
        self.show_side_panel = !self.show_side_panel;
    }

    // 獲取側邊面板狀態
    pub fn is_side_panel_open(&self) -> bool {
        self.show_side_panel
    }

    // 關閉側邊面板
    pub fn close_side_panel(&mut self) {
        self.show_side_panel = false;
    }

    // 側邊面板向上滾動
    #[allow(dead_code)]
    pub fn scroll_side_up(&mut self) {
        if self.side_scroll > 0 {
            self.side_scroll -= 1;
        }
    }

    // 側邊面板向下滾動
    #[allow(dead_code)]
    pub fn scroll_side_down(&mut self, visible_height: usize) {
        let max_scroll = self.side_messages.len().saturating_sub(visible_height);
        if self.side_scroll < max_scroll {
            self.side_scroll += 1;
        }
    }

    // 取得minimap 的內容
    pub fn get_minimap(&self, _area: Rect) -> Paragraph {
        // 根據 show_minimap 狀態決定要渲染的內容
        // 渲染小地圖
        let lines: Vec<Line> = self.minimap_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title("")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Cyan)))
            .style(Style::default().bg(Color::DarkGray).fg(Color::Cyan))
    }
    // 取得stats內容
    pub fn get_side_panel(&self, _area: Rect) -> Paragraph {
            // 渲染 Status 面板
        let lines = crate::observable::observable_to_lines(self.side_observable.as_ref());
        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title("")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::White)))
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
    }
    
    // 獲取側邊面板內容的行數
    pub fn get_side_panel_content_height(&self) -> u16 {
        let lines = crate::observable::observable_to_lines(self.side_observable.as_ref());
        (lines.len() + 2) as u16  // 內容行數 + 上下邊框
    }

    // 設置側邊面板的 Observable 對象
    pub fn set_side_observable(&mut self, obs: Box<dyn Observable>) {
        self.side_observable = obs;
    }

    // 開啟小地圖
    pub fn show_minimap(&mut self) {
        self.show_minimap = true;
    }

    // 關閉小地圖
    pub fn hide_minimap(&mut self) {
        self.show_minimap = false;
    }

    // 切換小地圖顯示狀態
    #[allow(dead_code)]
    pub fn toggle_minimap(&mut self) {
        self.show_minimap = !self.show_minimap;
    }

    // 獲取小地圖狀態
    pub fn is_minimap_open(&self) -> bool {
        self.show_minimap
    }

    // 更新小地圖內容（八方向的描述）
    pub fn update_minimap(&mut self, minimap_data: Vec<String>) {
        self.minimap_lines = minimap_data;
    }

    // 渲染小地圖懸浮視窗
    #[allow(dead_code)]
    pub fn render_minimap(&self, _area: Rect) -> Paragraph {
        let lines: Vec<Line> = self.minimap_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title("")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::Cyan)))
            .style(Style::default().bg(Color::DarkGray).fg(Color::Cyan))
    }

    // 在Output Message印出物件
    #[allow(dead_code)]
    pub fn print_obserable(&mut self, obs: &dyn Observable) {
        self.print(obs.show_title());
        self.print(obs.show_description());
        let list = obs.show_list();
        if !list.is_empty() {
            self.print(format!("--"));
            for item in list {
                self.print(format!("• {}", item));
            }
        }

    }

    // === 日誌視窗相關方法 ===
    
    // 添加系統日誌訊息
    pub fn log(&mut self, message: String) {
        use chrono::Local;
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let log_entry = format!("[{}] {}", timestamp, message);
        self.log_messages.push(log_entry);
        self.log_scroll = self.log_messages.len().saturating_sub(1);
    }
    
    // 切換日誌視窗顯示/隱藏
    pub fn toggle_log(&mut self) {
        self.show_log = !self.show_log;
    }
    
    // 顯示日誌視窗
    pub fn show_log_window(&mut self) {
        self.show_log = true;
    }
    
    // 隱藏日誌視窗
    pub fn hide_log(&mut self) {
        self.show_log = false;
    }
    
    // 獲取日誌視窗狀態
    pub fn is_log_open(&self) -> bool {
        self.show_log
    }
    
    // 日誌視窗向上滾動
    pub fn scroll_log_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }
    
    // 日誌視窗向下滾動
    pub fn scroll_log_down(&mut self, visible_height: usize) {
        let max_scroll = self.log_messages.len().saturating_sub(visible_height);
        if self.log_scroll < max_scroll {
            self.log_scroll += 1;
        }
    }
    
    // 渲染日誌視窗
    pub fn render_log(&self, area: Rect) -> Paragraph {
        let visible_height = area.height.saturating_sub(2) as usize;
        
        // 自動滾動到底部，顯示最新的訊息
        let total_messages = self.log_messages.len();
        let start_idx = if total_messages > visible_height {
            total_messages - visible_height
        } else {
            0
        };
        let end_idx = total_messages;
        
        let visible_messages: Vec<Line> = self.log_messages[start_idx..end_idx]
            .iter()
            .map(|msg| Line::from(msg.as_str()))
            .collect();
        
        Paragraph::new(Text::from(visible_messages))
            .block(Block::default()
                .title("📋 System Log")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black).fg(Color::Green)))
            .style(Style::default().bg(Color::Black).fg(Color::Green))
    }

    // === 大地圖相關方法 ===
    
    // 顯示大地圖
    pub fn show_map(&mut self, player_x: usize, player_y: usize) {
        self.show_map = true;
        // 將地圖偏移量設為玩家位置附近
        self.map_offset_x = player_x.saturating_sub(20);
        self.map_offset_y = player_y.saturating_sub(10);
    }
    
    // 關閉大地圖
    pub fn close_map(&mut self) {
        self.show_map = false;
    }
    
    // 檢查是否顯示大地圖
    pub fn is_map_open(&self) -> bool {
        self.show_map
    }
    
    // 移動大地圖視圖
    pub fn move_map_view(&mut self, dx: i32, dy: i32, max_width: usize, max_height: usize) {
        if dx < 0 && self.map_offset_x > 0 {
            self.map_offset_x = self.map_offset_x.saturating_sub((-dx) as usize);
        } else if dx > 0 {
            self.map_offset_x = (self.map_offset_x + dx as usize).min(max_width.saturating_sub(1));
        }
        
        if dy < 0 && self.map_offset_y > 0 {
            self.map_offset_y = self.map_offset_y.saturating_sub((-dy) as usize);
        } else if dy > 0 {
            self.map_offset_y = (self.map_offset_y + dy as usize).min(max_height.saturating_sub(1));
        }
    }
    
    // 渲染大地圖
    pub fn render_big_map(&self, area: Rect, map: &crate::map::Map, player_x: usize, player_y: usize, npc_manager: &crate::npc_manager::NpcManager) -> Paragraph {
        let visible_width = area.width.saturating_sub(2) as usize;
        let visible_height = area.height.saturating_sub(2) as usize;
        
        let mut lines = Vec::new();
        
        // 標題行
        lines.push(Line::from(vec![
            Span::styled(
                format!("地圖: {} (玩家位置: {}, {})", map.name, player_x, player_y),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )
        ]));
        
        lines.push(Line::from(vec![
            Span::styled(
                "操作: ↑↓←→ 移動視圖 | q 退出 | P=玩家 M=商人 F=農夫 D=醫生 W=工人 T=旅者 I=物品",
                Style::default().fg(Color::Gray)
            )
        ]));
        
        lines.push(Line::from(""));
        
        // 繪製地圖
        for y in 0..visible_height.min(map.height.saturating_sub(self.map_offset_y)) {
            let mut line_spans = Vec::new();
            let map_y = y + self.map_offset_y;
            
            for x in 0..visible_width.min(map.width.saturating_sub(self.map_offset_x)) {
                let map_x = x + self.map_offset_x;
                
                // 判斷是否是玩家位置
                if map_x == player_x && map_y == player_y {
                    line_spans.push(Span::styled("P", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
                } else {
                    // 檢查是否有 NPC
                    let npcs_here = npc_manager.get_npcs_at(map_x, map_y);
                    if !npcs_here.is_empty() {
                        let npc = npcs_here[0];
                        let npc_char = crate::npc_manager::NpcManager::get_display_char(&npc.name);
                        line_spans.push(Span::styled(
                            npc_char.to_string(),
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                        ));
                    } else if let Some(point) = map.get_point(map_x, map_y) {
                        // 檢查是否有物品
                        if !point.objects.is_empty() {
                            line_spans.push(Span::styled("I", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                        } else if point.walkable {
                            line_spans.push(Span::styled(" ", Style::default().fg(Color::White)));
                        } else {
                            // 根據地圖類型顯示不同字符
                            let char_display = match map.map_type {
                                crate::map::MapType::Forest => "🌲",
                                crate::map::MapType::Cave => "▓",
                                crate::map::MapType::Desert => "≈",
                                crate::map::MapType::Mountain => "△",
                                _ => "x",
                            };
                            line_spans.push(Span::styled(char_display, Style::default().fg(Color::DarkGray)));
                        }
                    } else {
                        line_spans.push(Span::styled("?", Style::default().fg(Color::Red)));
                    }
                }
            }
            lines.push(Line::from(line_spans));
        }
        
        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title("🗺️  大地圖")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black).fg(Color::White)))
            .style(Style::default().bg(Color::Black).fg(Color::White))
    }
}
