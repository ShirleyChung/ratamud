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
            current_time: String::from("Day 1 09:00"),
            show_minimap: false,
            minimap_lines: Vec::new(),
        }
    }

    // 添加訊息並將滾動位置移到最後（僅儲存純文本）
    pub fn print(&mut self, message: String) {
        self.messages.push(message);
        self.scroll = self.messages.len().saturating_sub(1);
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
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    // 向下滾動（受可見高度限制）
    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_scroll = self.messages.len().saturating_sub(visible_height);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
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

    // 渲染狀態列（不顯示邊框，只顯示文字）
    pub fn render_status(&self) -> Paragraph {
        let status_text = if let Some(time) = self.status_time {
            if time.elapsed() > Duration::from_secs(5) {
                self.current_time.clone()
            } else {
                format!("{} | {}", self.status, self.current_time)
            }
        } else {
            self.current_time.clone()
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
    pub fn scroll_side_up(&mut self) {
        if self.side_scroll > 0 {
            self.side_scroll -= 1;
        }
    }

    // 側邊面板向下滾動
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
    pub fn get_side_panel(&self, area: Rect) -> Paragraph {
            // 渲染 Status 面板
        let lines = crate::observable::observable_to_lines(self.side_observable.as_ref());
        Paragraph::new(Text::from(lines))
            .block(Block::default()
                .title("")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray).fg(Color::White)))
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
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
}
