use ratatui::style::Color;

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
        render_width: usize,
    ) -> Vec<ColoredChar>;
    /// Get the grid-size used for rasterizing
    fn grid_height(&self) -> usize;
    fn grid_width(&self) -> usize;
}
