//! Rendering fonts such that we can later learn the mappings

use core::f32;

use ab_glyph::{point, Font, FontRef, Glyph, InvalidFont, OutlinedGlyph};
use image::{ImageBuffer, Rgba};
use std::collections::HashMap;

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

pub enum GlyphError {
    PixelOutOfRange { x: usize, y: usize },
}

/// Holds all the intensity matrices for all the printable ASCII characters
pub struct GlyphMatrix {
    /// Character that this glyph represents
    symbol: char,
    /// Number of horizontal pixels assigned to one glyph
    width: usize,
    /// Number of vertical pixels assigned to one glyph
    height: usize,
    /// 2D array holding the intensity values across the grid
    // TODO Consider changing this to an array directly
    // NOTE That may require a constant annotation in the generic type
    matrix: Vec<f32>,
    /// Glyph outline that can be drawn
    glyph_outline: Option<OutlinedGlyph>,
    /// Vertical offset required to move glyph to center
    v_offset: Option<usize>,
    /// Horizontal offset required to move glyph to center
    h_offset: Option<usize>,
}

impl GlyphMatrix {
    pub fn new<F: Font>(font: &F, symbol: char, width: usize, height: usize) -> Self {
        let glyph = font.glyph_id(symbol).with_scale(height as f32);
        let glyph_outline = font.outline_glyph(glyph);
        let mut v_offset = None;
        let mut h_offset = None;
        // By default, fill in the matrix with characters that haven't been positioned properly
        let mut default_matrix = vec![0f32; width * height];
        if let Some(go) = &glyph_outline {
            v_offset = Some(0usize);
            h_offset = Some(0usize);
            go.draw(|x, y, c| {
                let idx = Self::pixel_to_index(x as usize, y as usize, width, height);
                if let Ok(idx) = idx {
                    default_matrix[idx] = c;
                }
            });
        }
        Self {
            symbol,
            width,
            height,
            glyph_outline,
            v_offset,
            h_offset,
            matrix: default_matrix,
        }
    }
    /// Converting from x, y locations to 1D index
    #[inline]
    fn pixel_to_index(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<usize, GlyphError> {
        if x < width && y < height {
            Ok(y * width + x)
        } else {
            Err(GlyphError::PixelOutOfRange { x, y })
        }
    }
    /// Get the value of a pixel
    fn get_pixel(&self, x: usize, y: usize) -> Option<f32> {
        let idx = Self::pixel_to_index(x, y, self.width, self.height);
        match idx {
            Ok(i) => Some(self.matrix[i]),
            Err(_) => None,
        }
    }
    fn update_matrix(&mut self) {
        self.matrix = vec![0f32; self.width * self.height];
        if let Some(go) = self.glyph_outline.as_ref() {
            go.draw(|x, y, c| {
                let idx = Self::pixel_to_index(
                    x as usize + self.h_offset.unwrap(),
                    y as usize + self.v_offset.unwrap(),
                    self.width,
                    self.height,
                );
                if let Ok(idx) = idx {
                    self.matrix[idx] = c;
                }
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
        let img = ImageBuffer::from_fn(self.width as u32, self.height as u32, |x, y| {
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
}

pub struct ASCIIMatrices {
    #[allow(dead_code)]
    width: usize,
    height: usize,
    glyph_matrices: HashMap<char, GlyphMatrix>,
}

impl ASCIIMatrices {
    /// Bare constructor for glyph matrices
    /// Note that after construction, glyph matrices will not be centered, so should call `v_center()` on struct
    pub fn new<F: Font>(font: &F, width: usize, height: usize) -> Self {
        let ascii_symbols: Vec<char> = (32..=126u8).map(|i| i as char).collect();
        let mut glyph_matrices = HashMap::new();
        for symbol in ascii_symbols.into_iter() {
            let glyph_matrix = GlyphMatrix::new(font, symbol, width, height);

            glyph_matrices.insert(symbol, glyph_matrix);
        }
        let mut out = Self {
            width,
            height,
            glyph_matrices,
        };
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
        let v_global_offset = (((self.height as f32) - new_bottom) / 2.0) as usize;

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
    pub fn save(&self) {
        for (_, gm) in self.glyph_matrices.iter() {
            gm.save()
        }
    }
}

/// Take a font and render its characters
pub fn draw_chars() -> Result<(), InvalidFont> {
    let font = get_font();
    let grid_size = 120usize;

    let ascii_matrices = ASCIIMatrices::new(&font, grid_size / 2, grid_size);
    ascii_matrices.save();

    Ok(())
}
