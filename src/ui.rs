use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Line;
use ratatui::layout::Rect;

// 處理輸入區域顯示的結構體
pub struct InputDisplay;

impl InputDisplay {
    // 渲染輸入區域的小部件
    pub fn render_input(input_text: &str, _area: Rect) -> Paragraph {
        // 建立帶邊框和標題的輸入區塊
        let input_block = Block::default()
            .title("*")
            .borders(Borders::ALL);

        // 將輸入文本轉換為段落小部件
        Paragraph::new(Line::from(input_text))
            .block(input_block)
    }
}
