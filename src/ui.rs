use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};
use ratatui::text::{Line, Span};
use ratatui::layout::{Rect, Alignment};
use ratatui::style::{Color, Style, Modifier};

// 處理輸入區域顯示的結構體
pub struct InputDisplay;

impl InputDisplay {
    // 渲染輸入區域的小部件
    pub fn render_input(input_text: &str, _area: Rect) -> Paragraph<'_> {
        // 建立帶邊框和標題的輸入區塊
        let input_block = Block::default()
            .title("*")
            .borders(Borders::ALL);

        // 將輸入文本轉換為段落小部件
        Paragraph::new(Line::from(input_text))
            .block(input_block)
    }
}

// 處理標題列顯示的結構體
pub struct HeaderDisplay;

impl HeaderDisplay {
    // 渲染標題列
    pub fn render_header<'a>(world_name: &'a str, current_time: &'a str) -> Paragraph<'a> {
        let header_text = format!("⚔️  {world_name} | 🕐 {current_time}");
        
        let header_span = Span::styled(
            header_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );
        
        Paragraph::new(Line::from(header_span))
            .alignment(Alignment::Left)
    }
}

pub struct Menu {
    pub title: String,
    pub items: Vec<String>,
    pub selected_index: usize,
    pub active: bool,
}

impl Menu {
    pub fn new(title: String, items: Vec<String>) -> Self {
        Self {
            title,
            items,
            selected_index: 0,
            active: false, // Menu starts inactive
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_item(&self) -> Option<&String> {
        self.items.get(self.selected_index)
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.selected_index = 0; // Reset selection when deactivated
    }

    pub fn render_widget(&self) -> List<'_> {
        let items: Vec<ListItem> = self.items.iter()
            .enumerate()
            .map(|(i, item)| {
                let mut style = Style::default().fg(Color::White);
                if i == self.selected_index {
                    style = style.add_modifier(Modifier::BOLD).bg(Color::DarkGray);
                }
                ListItem::new(Span::styled(item.clone(), style))
            })
            .collect();

        List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    self.title.clone(),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )))
    }
}
