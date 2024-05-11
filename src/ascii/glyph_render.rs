//! Rendering fonts such that we can later learn the mappings
use ab_glyph::{point, Font, FontRef, Glyph, OutlinedGlyph};
use core::f32;
use image::{ImageBuffer, Rgba};
use std::collections::BTreeMap;

// TODO See if this can be made a structure constant even when AsciiMatrices is generic
// TODO Check if it is actually this number
pub const NUM_ASCII_MATRICES: usize = 95;

// TODO Make this into a sensible function
// TODO Also put this in the resources of the package
pub fn get_font() -> impl Font {
    FontRef::try_from_slice(include_bytes!("../../data/FiraCode-Regular.ttf")).unwrap()
}

/// Get ASCII characters to debug
/// !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~
pub fn get_ascii_from_font<F: Font>(font: &F, grid_size: u32) -> Vec<Glyph> {
    (32..=126u8)
        .map(|i| i as char)
        .map(|c| font.glyph_id(c))
        .map(|g| g.with_scale_and_position(grid_size as f32, point(0.0, 0.0)))
        .collect()
}

/// Struct holding mean and standard deviation statistics
/// Needed because of alignment requirements when passing uniform to GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AsciiStats {
    /// Mean intensity
    pub mu: f32,
    /// Standard deviation of intensity
    pub sigma: f32,
    _padding1: f32,
    _padding2: f32,
}

/// Struct holding padded float
/// Choosing to pad rather than pack data and have complicated indices
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AsciiPixelPadded {
    pub intensity: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

impl From<f32> for AsciiPixelPadded {
    fn from(value: f32) -> Self {
        Self {
            intensity: value,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        }
    }
}

/// Holds all the intensity matrices for all the printable ASCII characters
/// `W` and `H` are the number of horizontal and vertical pixels assigned to one glyph
#[derive(Debug)]
pub struct GlyphMatrix<const W: usize, const H: usize> {
    /// Character that this glyph represents
    #[allow(dead_code)]
    symbol: char,
    /// 2D array holding the intensity values across the grid
    matrix: [[f32; W]; H],
    /// Glyph outline that can be drawn
    glyph_outline: Option<OutlinedGlyph>,
    /// Vertical offset required to move glyph to center
    v_offset: Option<usize>,
    /// Horizontal offset required to move glyph to center
    h_offset: Option<usize>,
}

impl<const W: usize, const H: usize> GlyphMatrix<W, H> {
    pub fn new<F: Font>(font: &F, symbol: char) -> Self {
        let glyph = font.glyph_id(symbol).with_scale(H as f32);
        let glyph_outline = font.outline_glyph(glyph);
        let mut v_offset = None;
        let mut h_offset = None;
        // By default, fill in the matrix with characters that haven't been positioned properly
        let mut default_matrix = [[0f32; W]; H];
        if let Some(go) = &glyph_outline {
            v_offset = Some(0usize);
            h_offset = Some(0usize);
            go.draw(|x, y, c| {
                // FIXME Be careful of out of bounds errors here
                default_matrix[y as usize][x as usize] = c;
            });
        }
        Self {
            symbol,
            glyph_outline,
            v_offset,
            h_offset,
            matrix: default_matrix,
        }
    }
    pub fn width(&self) -> usize {
        W
    }
    pub fn height(&self) -> usize {
        H
    }
    /// Get the value of a pixel
    fn get_pixel(&self, x: usize, y: usize) -> Option<f32> {
        if x < W && y < H {
            Some(self.matrix[y][x])
        } else {
            None
        }
    }
    fn update_matrix(&mut self) {
        self.matrix = [[0f32; W]; H];
        if let Some(go) = self.glyph_outline.as_ref() {
            go.draw(|x, y, c| {
                // FIXME Be careful about out of bounds errors here
                self.matrix[y as usize][x as usize] = 1.0 - c;
            });
        }
    }
    /// Testing whether glyph just contains whitespace
    pub fn is_blank(&self) -> bool {
        self.glyph_outline.is_none()
    }
    pub fn add_v_offset(&mut self, v_offset: usize) {
        if self.glyph_outline.is_some() {
            self.v_offset = Some(self.v_offset.unwrap() + v_offset);
        }
    }
    pub fn add_h_offset(&mut self, h_offset: usize) {
        if self.glyph_outline.is_some() {
            self.h_offset = Some(self.h_offset.unwrap() + h_offset);
        }
    }
    pub fn internal_min_y(&self) -> Option<f32> {
        self.glyph_outline.as_ref().map(|go| go.px_bounds().min.y)
    }
    pub fn internal_max_y(&self) -> Option<f32> {
        self.glyph_outline.as_ref().map(|go| go.px_bounds().max.y)
    }
    pub fn internal_min_x(&self) -> Option<f32> {
        self.glyph_outline.as_ref().map(|go| go.px_bounds().min.x)
    }
    pub fn internal_max_x(&self) -> Option<f32> {
        self.glyph_outline.as_ref().map(|go| go.px_bounds().max.x)
    }
    pub fn internal_height(&self) -> Option<f32> {
        self.glyph_outline
            .as_ref()
            .map(|go| go.px_bounds().height())
    }
    pub fn internal_width(&self) -> Option<f32> {
        self.glyph_outline.as_ref().map(|go| go.px_bounds().width())
    }
    pub fn save(&self) {
        let img = ImageBuffer::from_fn(W as u32, H as u32, |x, y| {
            Rgba([
                0,
                0,
                0,
                (self.get_pixel(x as usize, y as usize).unwrap_or(0.0f32) * 255.0) as u8,
            ])
        });
        let glyph_id = self
            .glyph_outline
            .as_ref()
            .map(|go| go.glyph().id)
            .unwrap_or_default();
        // TODO add a check that directory exists
        let filename = format!("characters/{:?}_character.png", glyph_id);
        img.save(filename).unwrap();
    }
    /// Calculate the mean of the ASCII matrix
    fn mean(&self) -> f32 {
        let sum: f32 = self.matrix.iter().flatten().sum();
        sum / (W as f32 * H as f32)
    }
    /// Calculate the standard deviation of the ASCII matrix
    fn std(&self) -> f32 {
        // TODO Double check this formula, you idiot
        let sum_squares: f32 = self.matrix.iter().flatten().map(|f| (*f) * (*f)).sum();
        (sum_squares / (W as f32 * H as f32)).sqrt()
    }
    /// Calculate both mean and standard deviation of the ASCII matrix
    pub fn stats(&self) -> AsciiStats {
        AsciiStats {
            mu: self.mean(),
            sigma: self.std(),
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
    /// Calculate a padded version of the matrix for WebGPU
    /// Even though it creates new data, the ASCII matrices still won't be very large: <1MiB for 95 x 16x32 grids
    pub fn padded_matrix(&self) -> [[AsciiPixelPadded; W]; H] {
        self.matrix.map(|row| row.map(|v| v.into()))
    }
}

#[derive(Debug)]
pub struct AsciiMatrices<const W: usize, const H: usize> {
    // There may be performance hit from not using `HashMap`, but choose convenience of being sorted
    glyph_matrices: BTreeMap<char, GlyphMatrix<W, H>>,
}

impl<const W: usize, const H: usize> AsciiMatrices<W, H> {
    /// Bare constructor for glyph matrices
    /// Will do horizontal and vertical centering
    pub fn new<F: Font>(font: &F) -> Self {
        // TODO Define this range near the constants
        let ascii_symbols: Vec<char> = (32..=126u8).map(|i| i as char).collect();
        let mut glyph_matrices = BTreeMap::new();
        for symbol in ascii_symbols.into_iter() {
            let glyph_matrix = GlyphMatrix::<W, H>::new(font, symbol);

            glyph_matrices.insert(symbol, glyph_matrix);
        }
        let mut out = Self { glyph_matrices };
        out.v_center();
        out.h_center();
        out
    }
    /// Center the glyphs vertically, so that they are consistent and lie in the middle
    /// Needed because by default, each glyph is drawn with its highest point up against the top of the cell
    pub fn v_center(&mut self) {
        let top = self
            .glyph_matrices
            .iter()
            .filter_map(|(_, gm)| gm.internal_min_y())
            .reduce(f32::min)
            .unwrap();
        for (_, gm) in self.glyph_matrices.iter_mut() {
            if !gm.is_blank() {
                let offset = (gm.internal_min_y().unwrap() - top).round() as usize;
                gm.add_v_offset(offset);
            }
        }
        let new_bottom = self
            .glyph_matrices
            .iter()
            .filter_map(|(_, gm)| gm.v_offset.zip(gm.internal_height()))
            .map(|(o, h)| h + o as f32)
            .reduce(f32::max)
            .unwrap();
        let v_global_offset = (((H as f32) - new_bottom) / 2.0) as usize;

        for (_, gm) in self.glyph_matrices.iter_mut() {
            if !gm.is_blank() {
                gm.add_v_offset(v_global_offset);
                gm.update_matrix();
            }
        }
    }
    /// Center the glyphs horizontally, so that kerning is respected and lie in the middle of a cell
    pub fn h_center(&mut self) {
        let left = self
            .glyph_matrices
            .iter()
            .filter_map(|(_, gm)| gm.internal_min_x())
            .reduce(f32::min)
            .unwrap();
        for (_, gm) in self.glyph_matrices.iter_mut() {
            if !gm.is_blank() {
                let offset = (gm.internal_min_x().unwrap() - left).round() as usize;
                gm.add_h_offset(offset);
                gm.update_matrix();
            }
        }
    }
    /// Save the rendered glyphs, useful for debugging
    pub fn save(&self) {
        for (_, gm) in self.glyph_matrices.iter() {
            gm.save()
        }
    }
    /// Export the ASCII matrices as a 3D texture
    pub fn padded_matrix_list(&self) -> [[[AsciiPixelPadded; W]; H]; NUM_ASCII_MATRICES] {
        self.glyph_matrices
            .values()
            .map(|gm| gm.padded_matrix())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
    /// Calculate the mean and standard deviation of every ASCII matrix
    /// Useful for the rasterizer which uses SSIM
    pub fn matrix_stats(&self) -> [AsciiStats; NUM_ASCII_MATRICES] {
        self.glyph_matrices
            .values()
            .map(|gm| gm.stats())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_chars() {
        let font = get_font();
        const GRID_WIDTH: usize = 16;
        const GRID_HEIGHT: usize = 32;

        let ascii_matrices = AsciiMatrices::<GRID_WIDTH, GRID_HEIGHT>::new(&font);
        assert!(!ascii_matrices.glyph_matrices.is_empty());

        assert_eq!(ascii_matrices.glyph_matrices.len(), NUM_ASCII_MATRICES);

        // let rand = ascii_matrices.glyph_matrices.get(&'a');
        // TODO Write some check using this
    }
}
