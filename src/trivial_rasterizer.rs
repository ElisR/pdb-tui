//! Rasterizer for converting compute shader characters

use crate::rasterizer::ColoredChar;

use ratatui::{
    prelude::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub fn chars_to_widget(chars: Vec<ColoredChar>, output_width: usize) -> impl Widget {
    let lines: Vec<Line> = chars
        .chunks(output_width)
        .rev()
        .map(|row| {
            let spans: Vec<Span> = row
                .iter()
                .map(|colored_char| {
                    Span::styled(
                        colored_char.symbol.to_string(),
                        Style::default().fg(colored_char.color),
                    )
                })
                .collect();
            Line::default().spans(spans)
        })
        .collect();
    Paragraph::new(lines)
}
