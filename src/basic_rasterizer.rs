use crate::rasterizer::{ColoredChar, ColoredPixel, Rasterizer, RasterizerError};
use ratatui::style::Color;

use ratatui::{
    prelude::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// Simple rasterizer that assigns one ASCII character per pixel intensity.
/// Doesn't care about shapes of the pixels.
#[derive(Clone)]
pub struct BasicAsciiRasterizer {
    gradient: Vec<char>,
    ranges: Vec<(f32, f32)>,
    background: char,
}

impl BasicAsciiRasterizer {
    /// Validating the parameters and raising any errors that should propagate to the constructor
    fn validate_parameters(
        gradient: &[char],
        thresholds: &[f32],
    ) -> Result<Vec<(f32, f32)>, RasterizerError> {
        if gradient.len() + 1 != thresholds.len() {
            return Err(RasterizerError::GradientNotMatchingThresholds);
        }
        let ranges: Vec<(f32, f32)> = thresholds.windows(2).map(|w| (w[0], w[1])).collect();
        if !ranges.iter().all(|(l, u)| *l < *u) {
            return Err(RasterizerError::ThresholdsNotIncreasing);
        }
        Ok(ranges)
    }
    pub fn new(
        gradient: Vec<char>,
        thresholds: Vec<f32>,
        background: char,
    ) -> Result<BasicAsciiRasterizer, RasterizerError> {
        match BasicAsciiRasterizer::validate_parameters(&gradient, &thresholds) {
            Ok(ranges) => Ok(BasicAsciiRasterizer {
                gradient,
                ranges,
                background,
            }),
            Err(e) => Err(e),
        }
    }
    fn pixel_to_char(&self, pixel: ColoredPixel) -> ColoredChar {
        let mut symbol = self.background;
        for (i, (min, max)) in self.ranges.iter().enumerate() {
            if pixel.intensity > *min && pixel.intensity <= *max {
                symbol = self.gradient[i];
                return ColoredChar {
                    symbol,
                    color: pixel.color,
                };
            }
        }
        ColoredChar {
            symbol,
            color: pixel.color,
        }
    }

    // NOTE The performance benefit of this function may now be worth the hassle
    // TODO Delete it and go back to the old version unless benchmark show it's worth it
    pub fn pixels_to_widget(
        &self,
        pixels: Vec<&[ColoredPixel]>,
        output_width: usize,
    ) -> impl Widget {
        let lines: Vec<Line> = pixels
            .chunks(output_width)
            .rev()
            .map(|row| {
                let spans: Vec<Span> = row
                    .iter()
                    .map(|chunk| {
                        let pixel = chunk[0];
                        let ascii = self.pixel_to_char(pixel);
                        Span::styled(ascii.symbol.to_string(), Style::default().fg(ascii.color))
                    })
                    .collect();
                Line::default().spans(spans)
            })
            .collect();
        Paragraph::new(lines)
    }
}

impl Default for BasicAsciiRasterizer {
    fn default() -> Self {
        let gradient = vec!['@', '%', '#', '*', '+', '=', '-', ':', '.'];
        let thresholds = vec![-0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.999];
        let background = ' ';
        Self::new(gradient, thresholds, background).unwrap()
    }
}

impl Rasterizer for BasicAsciiRasterizer {
    fn pixels_to_stdout(
        &self,
        pixels: Vec<&[ColoredPixel]>,
        output_width: usize,
    ) -> Vec<ColoredChar> {
        // Add one per row to account for newline character
        let total_chars = pixels.len() + (pixels.len() / output_width);
        let mut out: Vec<ColoredChar> = Vec::with_capacity(total_chars);
        // Reverse because small coord means small index, but the top of the screen should have large y
        for row in pixels.chunks(output_width).rev() {
            for chunk in row.iter() {
                let pixel = chunk[0];
                let ascii = self.pixel_to_char(pixel);
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
        1
    }
    fn grid_width(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_rasterizer() {
        let rasterizer = BasicAsciiRasterizer::default();
        assert_eq!(
            rasterizer.pixel_to_char(ColoredPixel::from(0.15)).symbol,
            '@'
        );
        assert_eq!(
            rasterizer.pixel_to_char(ColoredPixel::from(0.65)).symbol,
            '='
        );
        assert_eq!(
            rasterizer.pixel_to_char(ColoredPixel::from(0.85)).symbol,
            ':'
        );
        assert_eq!(
            rasterizer.pixel_to_char(ColoredPixel::from(1.15)).symbol,
            rasterizer.background,
        );
    }

    #[test]
    fn test_nonincreasing_error() {
        let thresholds = vec![0.0, 0.4, 0.9, 0.6];
        let gradient = vec!['.', '.', '.'];
        let rasterizer = BasicAsciiRasterizer::new(gradient, thresholds, ' ');
        assert!(rasterizer.is_err_and(|x| x == RasterizerError::ThresholdsNotIncreasing));
    }

    #[test]
    fn test_notmatching_error() {
        let thresholds = vec![0.0, 0.4, 0.6];
        let gradient = vec!['.', '.', '.'];
        let rasterizer = BasicAsciiRasterizer::new(gradient, thresholds, ' ');
        assert!(rasterizer.is_err_and(|x| x == RasterizerError::GradientNotMatchingThresholds));
    }
}
