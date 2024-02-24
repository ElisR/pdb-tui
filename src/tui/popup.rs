#![allow(dead_code)]
use ratatui::{
    prelude::{Buffer, Line, Rect, Style},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use derive_setters::Setters;

#[derive(Debug, Default, Setters)]
pub struct HelpPopup<'a> {
    #[setters(into)]
    title: Line<'a>,
    #[setters(into)]
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl Widget for HelpPopup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf);
    }
}
