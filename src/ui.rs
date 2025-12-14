use ratatui::widgets::{Block, Borders, Paragraph};
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
