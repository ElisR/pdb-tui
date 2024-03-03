//! Rendering fonts such that we can later learn the mappings

use core::f32;

use ab_glyph::{point, Font, FontRef, Glyph, InvalidFont};
use image::{ImageBuffer, Rgba};

pub fn draw_char() -> Result<(), InvalidFont> {
    let font = FontRef::try_from_slice(include_bytes!("../../data/FiraCode-Regular.ttf"))?;
    let grid_size = 120u32;

    // Print all ASCII characters to debug, 32..=126u8
    // !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~
    let glyphs: Vec<Glyph> = (32..=126u8)
        .map(|i| i as char)
        .map(|c| font.glyph_id(c))
        .map(|g| g.with_scale_and_position(grid_size as f32, point(0.0, 0.0)))
        .collect();

    // Fix the vertical alignment, since each glyph is drawn with it's highest point up against the top of the cell
    let top = glyphs
        .clone()
        .into_iter()
        .filter_map(|g| font.outline_glyph(g))
        .map(|go| go.px_bounds().min.y)
        .reduce(f32::min)
        .unwrap();
    let bottom_new = glyphs
        .clone()
        .into_iter()
        .filter_map(|g| font.outline_glyph(g))
        .map(|go| go.px_bounds().min.y - top + go.px_bounds().height())
        .reduce(f32::max)
        .unwrap();
    let v_global_offset = (((grid_size as f32) - bottom_new) / 2.0) as u32;

    let left = glyphs
        .clone()
        .into_iter()
        .filter_map(|g| font.outline_glyph(g))
        .map(|go| go.px_bounds().min.x)
        .reduce(f32::min)
        .unwrap();

    let mut long_img = ImageBuffer::new((glyphs.len() as u32) * grid_size, grid_size);
    for (i, g) in glyphs.into_iter().enumerate() {
        if let Some(go) = font.outline_glyph(g) {
            let v_offset = (go.px_bounds().min.y - top).round() as u32;
            let h_offset = (go.px_bounds().min.x - left).round() as u32;

            println!(
                "{:?} = {:?}, offset = {}",
                go.glyph().id,
                go.px_bounds(),
                v_offset
            );
            go.draw(|x, y, c| {
                let pixel = long_img.get_pixel_mut(
                    i as u32 * grid_size / 2 + x + h_offset,
                    y + v_offset + v_global_offset,
                );
                *pixel = Rgba([0, 0, 0, (c * 255.0) as u8]);
            });
        }
    }
    long_img.save("characters.png").unwrap();

    Ok(())
}
