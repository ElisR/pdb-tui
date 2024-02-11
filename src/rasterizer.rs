/// Simple rasterizer that assigns one ASCII character per pixel intensity.
/// Doesn't care about shapes of the pixels.
#[derive(Clone)]
pub struct BasicAsciiRasterizer {
    gradient: Vec<char>,
    ranges: Vec<(f32, f32)>,
    background: char,
}

#[derive(Debug)]
pub enum RasterizerError {
    GradientNotMatchingThresholds,
    ThresholdsNotIncreasing,
}

impl BasicAsciiRasterizer {
    fn new(
        gradient: Vec<char>,
        thresholds: Vec<f32>,
        background: char,
    ) -> Result<BasicAsciiRasterizer, RasterizerError> {
        if gradient.len() + 1 == thresholds.len() {
            // Collect the thresholds into ranges from contiguous pairs
            // TODO Add a check that w[1] > w[0] always
            let ranges: Vec<(f32, f32)> = thresholds.windows(2).map(|w| (w[0], w[1])).collect();
            Ok(BasicAsciiRasterizer {
                gradient,
                ranges,
                background,
            })
        } else {
            Err(RasterizerError::GradientNotMatchingThresholds)
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
        let mut gradient = vec!['.', ':', '-', '=', '+', '*', '#', '%', '@'];
        gradient.reverse();
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
        let mut out: Vec<char> = Vec::with_capacity(pixels.len());
        // FIXME Find a better way to avoid stretching image
        // Crude hack to account for height of font character being bigger than width
        for row in pixels.iter().step_by(2) {
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
        assert_eq!(rasterizer.pixel_to_char(0.15), '.');
        assert_eq!(rasterizer.pixel_to_char(0.65), '*');
        assert_eq!(rasterizer.pixel_to_char(0.85), '%');
        assert_eq!(rasterizer.pixel_to_char(1.15), ' ');
    }
}
