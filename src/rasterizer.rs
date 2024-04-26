use ratatui::{
    prelude::Style,
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ColoredPixel {
    pub intensity: f32,
    pub color: Color,
}

impl ColoredPixel {
    pub fn to_grayscale(&self) -> u8 {
        (self.intensity * 255.0).round() as u8
    }

    // TODO Add function for converting `ColoredPixel` to RGB and RGBA value
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        todo!()
    }
}

impl From<ColoredPixel> for f32 {
    fn from(canvas_pixel: ColoredPixel) -> Self {
        canvas_pixel.intensity
    }
}
impl From<ColoredPixel> for Color {
    fn from(canvas_pixel: ColoredPixel) -> Self {
        canvas_pixel.color
    }
}

impl From<f32> for ColoredPixel {
    fn from(intensity: f32) -> Self {
        Self {
            intensity,
            color: Color::Black,
        }
    }
}

impl From<u8> for ColoredPixel {
    fn from(value: u8) -> Self {
        Self {
            intensity: value as f32 / std::u8::MAX as f32,
            color: Color::Red,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ColoredChar {
    pub symbol: char,
    pub color: Color,
}

impl From<ColoredChar> for char {
    fn from(colored_char: ColoredChar) -> Self {
        colored_char.symbol
    }
}

/// Convert from ASCII characters encoded as u8 in compute shader
impl From<u8> for ColoredChar {
    fn from(value: u8) -> Self {
        Self {
            symbol: value as char,
            color: Color::Red,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RasterizerError {
    GradientNotMatchingThresholds,
    ThresholdsNotIncreasing,
}

pub trait Rasterizer {
    // Convert a vector of slices of pixels to a vector of characters to be printed to the terminal
    fn pixels_to_stdout(
        &self,
        pixels: Vec<&[ColoredPixel]>,
        output_width: usize,
    ) -> Vec<ColoredChar>;
    /// Get the grid-size used for rasterizing
    fn grid_height(&self) -> usize;
    fn grid_width(&self) -> usize;
}

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
