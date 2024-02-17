/// Simple rasterizer that assigns one ASCII character per pixel intensity.
/// Doesn't care about shapes of the pixels.
#[derive(Clone)]
pub struct BasicAsciiRasterizer {
    gradient: Vec<char>,
    ranges: Vec<(f32, f32)>,
    background: char,
}

#[derive(Debug, PartialEq)]
pub enum RasterizerError {
    GradientNotMatchingThresholds,
    ThresholdsNotIncreasing,
}

impl BasicAsciiRasterizer {
    /// Validating the parameters and raising any errors that should propagate to the constructor
    fn validate_parameters(
        gradient: &Vec<char>,
        thresholds: &Vec<f32>,
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
    fn pixel_to_char(&self, val: f32) -> char {
        let mut out = self.background;
        for (i, (min, max)) in self.ranges.iter().enumerate() {
            if val > *min && val <= *max {
                out = self.gradient[i];
                return out;
            }
        }
        out
    }
}

impl Default for BasicAsciiRasterizer {
    fn default() -> Self {
        let gradient = vec!['@', '%', '#', '*', '+', '=', '-', ':', '.'];
        let background = '@';
        BasicAsciiRasterizer::new(
            gradient,
            vec![0.0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            background,
        )
        .unwrap()
    }
}

pub trait Rasterizer {
    // Convert a vector of slices of pixels to a vector of characters to be printed to the terminal
    fn pixels_to_stdout(&self, pixels: Vec<&[f32]>) -> Vec<char>;

    /// Get the character used for the background
    fn get_bg_char(&self) -> char;
}

impl Rasterizer for BasicAsciiRasterizer {
    fn pixels_to_stdout(&self, pixels: Vec<&[f32]>) -> Vec<char> {
        // Add one to account for newline character
        let total_chars: usize = pixels.iter().map(|row| row.len() + 1).sum();
        let mut out: Vec<char> = Vec::with_capacity(total_chars);
        // Reverse because small coord means small index, but the top of the screen should have large y
        for row in pixels.iter().rev() {
            for pixel in row.iter() {
                let ascii = self.pixel_to_char(*pixel);
                out.push(ascii);
            }
            out.push('\n');
        }
        out
    }

    fn get_bg_char(&self) -> char {
        self.background
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rasterizer() {
        let rasterizer = BasicAsciiRasterizer::default();
        assert_eq!(rasterizer.pixel_to_char(0.15), '@');
        assert_eq!(rasterizer.pixel_to_char(0.65), '=');
        assert_eq!(rasterizer.pixel_to_char(0.85), ':');
        assert_eq!(rasterizer.pixel_to_char(1.15), rasterizer.get_bg_char());
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
