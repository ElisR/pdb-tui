use crate::rasterizer::{ColoredChar, ColoredPixel, Rasterizer, RasterizerError};
use ratatui::style::Color;

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
    fn pixel_to_char(&self, colored_pixel: ColoredPixel) -> ColoredChar {
        let mut symbol = self.background;
        for (i, (min, max)) in self.ranges.iter().enumerate() {
            if colored_pixel.intensity > *min && colored_pixel.intensity <= *max {
                symbol = self.gradient[i];
                return ColoredChar {
                    symbol,
                    color: colored_pixel.color,
                };
            }
        }
        ColoredChar {
            symbol,
            color: colored_pixel.color,
        }
    }
}

impl Default for BasicAsciiRasterizer {
    fn default() -> Self {
        let gradient = vec!['@', '%', '#', '*', '+', '=', '-', ':', '.'];
        let background = ' ';
        BasicAsciiRasterizer::new(
            gradient,
            vec![-0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            background,
        )
        .unwrap()
    }
}

impl Rasterizer for BasicAsciiRasterizer {
    fn pixels_to_stdout(
        &self,
        pixels: Vec<&[ColoredPixel]>,
        render_width: usize,
    ) -> Vec<ColoredChar> {
        // Add one to account for newline character
        // let total_chars: usize = pixels.iter().map(|row| row.len() + 1).sum();
        let total_chars = pixels.len() + (pixels.len() / render_width);
        let mut out: Vec<ColoredChar> = Vec::with_capacity(total_chars);
        // Reverse because small coord means small index, but the top of the screen should have large y
        for row in pixels.chunks(render_width).rev() {
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
    fn bg_char(&self) -> char {
        self.background
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
            rasterizer.bg_char()
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
