//! Fancier rasterizer using grids bigger than 1x1
use crate::ascii::glyph_render::{get_font, AsciiMatrices};
use crate::rasterizer::{ColoredChar, ColoredPixel, Rasterizer};
use ratatui::style::Color;
use std::collections::HashMap;

pub struct FancyAsciiRasterizer {
    ascii_matrices: AsciiMatrices,
}

impl FancyAsciiRasterizer {
    pub fn new(grid_width: usize, grid_height: usize) -> Self {
        let font = get_font();
        let ascii_matrices = AsciiMatrices::new(&font, grid_width, grid_height);
        Self { ascii_matrices }
    }
    pub fn mean_chunk_color(&self, chunk: &[ColoredPixel]) -> Color {
        let mut counts = HashMap::new();
        for color in chunk.iter().map(|cp| cp.color) {
            *counts.entry(color).or_insert(0usize) += 1usize;
        }
        let (col, _) = counts.into_iter().max_by_key(|&(_, count)| count).unwrap();
        col
    }
    pub fn chunk_to_symbol(&self, chunk: &[ColoredPixel]) -> ColoredChar {
        let color = self.mean_chunk_color(chunk);
        let intensities: Vec<f32> = chunk.iter().map(|c| c.intensity).collect();
        let symbol = self.ascii_matrices.pick_best_symbol(&intensities);
        ColoredChar { symbol, color }
    }
}

impl Default for FancyAsciiRasterizer {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl Rasterizer for FancyAsciiRasterizer {
    fn pixels_to_stdout(
        &self,
        pixels: Vec<&[ColoredPixel]>,
        output_width: usize,
    ) -> Vec<ColoredChar> {
        // NOTE As input, we essentially want each cell to be like its own "pixel" from before
        // Add one per row to account for newline character
        let total_chars = pixels.len() + (pixels.len() / output_width);
        let mut out: Vec<ColoredChar> = Vec::with_capacity(total_chars);
        // Reverse because small coord means small index, but the top of the screen should have large y
        for row in pixels.chunks(output_width).rev() {
            for chunk in row.iter() {
                let ascii = self.chunk_to_symbol(chunk);
                out.push(ascii);
            }
            // This works because the grid height is 1
            out.push(ColoredChar {
                symbol: '\n',
                color: Color::Reset,
            });
        }
        out
    }
    fn grid_height(&self) -> usize {
        self.ascii_matrices.height
    }
    fn grid_width(&self) -> usize {
        self.ascii_matrices.width
    }
}
