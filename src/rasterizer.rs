/// Simple rasterizer that assigns one ASCII character per pixel intensity.
/// Doesn't care about shapes of the pixels.
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
        BasicAsciiRasterizer::new(
            vec!['.', ':', '-', '=', '+', '*', '#', '%', '@'],
            vec![0.0, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            ' ',
        )
        .unwrap()
    }
}

pub trait Rasterizer {
    fn pixels_to_stdout(&self, pixels: Vec<Vec<f32>>) -> Vec<char>;
}

impl Rasterizer for BasicAsciiRasterizer {
    fn pixels_to_stdout(&self, pixels: Vec<Vec<f32>>) -> Vec<char> {
        let mut out: Vec<char> = vec![];
        for row in pixels.iter() {
            for pixel in row.iter() {
                let ascii = self.pixel_to_char(*pixel);
                out.push(ascii);
            }
            out.push('\n');
        }
        out
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
