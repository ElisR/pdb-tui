// #![allow(dead_code)]
use crate::{
    rasterizer::{ColoredChar, ColoredPixel, Rasterizer},
    scene::{create_ray, Scene},
    surface::ValidShape,
};
use image::{imageops::flip_vertical_in_place, GrayImage, ImageResult};
use parry3d::query::RayCast;
use ratatui::style::Color;
use std::path::Path;

const SCREEN_PIXELS_X: usize = 320;
const SCREEN_PIXELS_Y: usize = 180;

pub enum CanvasError {
    PixelOutOfRange { x: usize, y: usize },
}

#[derive(Debug)]
pub struct Canvas<R: Rasterizer> {
    pub frame_buffer: Vec<ColoredChar>,
    // TODO Consider changing pixel buffer to 2D array for more convenience
    pub pixel_buffer: Vec<ColoredPixel>,
    pub toi_buffer: Vec<f32>,
    width: usize,
    height: usize,
    pub rasterizer: R,
    /// Pixel intensity used for the background
    pub bg_pixel: ColoredPixel,
}
impl<R: Rasterizer> Canvas<R> {
    /// Constructor for canvas.
    /// Depending on the rasterizer, the canvas may contain more pixels than passed to the constructor.
    /// This is because the rasterizer may perform some downsampling to produce a string.
    pub fn new(render_width: usize, render_height: usize, rasterizer: R) -> Self {
        let bg_pixel = ColoredPixel {
            intensity: 1.1f32,
            color: Color::Reset,
        };

        // Recalculate height and width depending on rasterizer
        let width = rasterizer.grid_width() * render_width;
        let height = rasterizer.grid_height() * render_height;

        let size = width * height;
        let pixel_buffer = vec![bg_pixel; size];
        let toi_buffer = vec![f32::MAX; size];
        let frame_buffer = rasterizer.pixels_to_stdout(pixel_buffer.chunks(width).collect());
        Canvas {
            frame_buffer,
            pixel_buffer,
            toi_buffer,
            width,
            height,
            rasterizer,
            bg_pixel,
        }
    }
}
impl<R: Rasterizer> Canvas<R> {
    /// Get the grid width multiplying the render width to get the width of internal objects
    /// These are not the same because rasterizer may perform subsampling
    pub fn grid_width(&self) -> usize {
        self.rasterizer.grid_width()
    }
    /// Get the grid height multiplying the render height to get the height of internal objects
    /// These are not the same because rasterizer may perform subsampling
    pub fn grid_height(&self) -> usize {
        self.rasterizer.grid_height()
    }
    /// Resize the canvas self-consistently
    /// Unfortunately also wipes the canvas
    pub fn resize(&mut self, render_width: usize, render_height: usize) {
        self.width = self.grid_width() * render_width;
        self.height = self.grid_height() * render_height;
        let size = self.width * self.height;

        self.pixel_buffer = vec![self.bg_pixel; size];
        self.toi_buffer = vec![f32::MAX; size];
        self.frame_buffer = self.rasterizer.pixels_to_stdout(self.pixels_as_scanlines())
    }
    /// Return width
    /// Width made private by default to discourage resizing without resizing other quantities
    pub fn render_width(&self) -> usize {
        self.width / self.grid_width()
    }
    /// Return height
    /// Height made private by default to discourage resizing without resizing other quantities
    pub fn render_height(&self) -> usize {
        self.height / self.grid_height()
    }
    /// Update the frame buffer with whatever the pixel buffer is set to
    pub fn update_frame(&mut self) {
        self.frame_buffer = self.rasterizer.pixels_to_stdout(self.pixels_as_scanlines())
    }
    /// Reshape the vector of pixels to a 2D vector that can be accepted by `Rasterizer`
    fn pixels_as_scanlines(&self) -> Vec<&[ColoredPixel]> {
        self.pixel_buffer.chunks(self.width).collect()
    }
    /// Reshape the vector of pixels to
    fn pixels_as_grid_chunks(&self) {
        // TODO
        todo!()
    }
    /// Utility function for calculating index, given pixel location
    /// `x` here runs from `0..width` i.e. `0..grid_width()*render_width()`.
    ///
    /// Here, chunks are grouped like like
    /// ```
    /// 0011223344
    /// 0011223344
    /// 5566778899
    /// 5566778899
    /// ```
    fn pixel_to_index(&self, x: usize, y: usize) -> Result<usize, CanvasError> {
        // This makes the most sense because then horizontally adjacent characters adjacent in memory
        if x < self.width && y < self.height {
            let x_major = x / self.grid_width();
            let x_minor = x % self.grid_width();

            let y_major = y / self.grid_height();
            let y_minor = y % self.grid_height();

            let idx = y_major * self.width * self.grid_height()
                + y_minor * self.width
                + x_major * self.grid_width()
                + x_minor;

            // Original code
            Ok(idx)
        } else {
            Err(CanvasError::PixelOutOfRange { x, y })
        }
    }
    /// Set a pixel unconditionally
    /// Will do nothing if pixel out of range
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, colored_pixel: ColoredPixel) {
        if let Ok(idx) = self.pixel_to_index(x, y) {
            self.pixel_buffer[idx] = colored_pixel;
        }
    }
    /// Set a pixel conditional on time-of-impact being lower than current buffer value
    /// Also updates time-of-impact buffer
    /// Will do nothing if pixel out of range
    #[inline]
    pub fn set_pixel_toi(&mut self, x: usize, y: usize, colored_pixel: ColoredPixel, toi: f32) {
        if let Ok(idx) = self.pixel_to_index(x, y) {
            if toi < self.toi_buffer[idx] {
                self.pixel_buffer[idx] = colored_pixel;
                self.toi_buffer[idx] = toi;
            }
        }
    }
    /// Update time-of-impact buffer
    /// Will do nothing if pixel out of range
    #[inline]
    pub fn set_toi(&mut self, x: usize, y: usize, toi: f32) {
        if let Ok(idx) = self.pixel_to_index(x, y) {
            self.toi_buffer[idx] = toi;
        }
    }
    /// Set all the buffers to just display the background
    pub fn flush_buffers(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_pixel(x, y, self.bg_pixel);
                self.set_toi(x, y, f32::MAX);
            }
        }
    }
    /// Update the canvas with the current state of the scene
    pub fn draw_scene_to_canvas<S: RayCast + ValidShape>(&mut self, scene: &Scene<S>) {
        self.flush_buffers();
        for y in 0..self.height {
            for x in 0..self.width {
                let x_clip = pixel_to_clip(x, self.width);
                let y_clip = pixel_to_clip(y, self.height);
                let ray = create_ray(x_clip, y_clip, scene);
                // FIXME make sure this works when using something other than meshes
                for colored_shape in scene.shapes().iter() {
                    // FIXME Make sure max_toi is reasonable
                    let toi_result = colored_shape.shape.cast_ray_and_get_normal(
                        &colored_shape.world_transform,
                        &ray,
                        scene.scene_projection.perspective.zfar() + 100.0,
                        true,
                    );
                    // TODO Consider whether we should take `abs` of intensity
                    if let Some(ri) = toi_result {
                        let normal = ri.normal;
                        // Taking ReLU of intensity to give darkness if incident on normal pointing in wrong direction
                        // TODO Consider using `std::clamp` function for more readability
                        let intensity: f32 = scene
                            .lights
                            .iter()
                            .fold(0.0, |i, l| i + normal.dot(l).max(0.0));
                        self.set_pixel_toi(
                            x,
                            y,
                            ColoredPixel {
                                intensity,
                                color: colored_shape.color,
                            },
                            ri.toi,
                        );
                    }
                }
            }
        }
        self.update_frame()
    }
    /// Wrapper for saving image. Filetype will be inferred from path
    pub fn save_image<Q>(&self, path: Q) -> ImageResult<()>
    where
        Q: AsRef<Path>,
    {
        let pixels_transformed = self.pixel_buffer.iter().map(|p| p.to_grayscale()).collect();
        let mut image_buffer =
            GrayImage::from_raw(self.width as u32, self.height as u32, pixels_transformed).unwrap();
        // Flip because small coord means small index, but top of image should have large y
        flip_vertical_in_place(&mut image_buffer);
        image_buffer.save(path)
    }
}

impl<R: Rasterizer + Default> Default for Canvas<R> {
    fn default() -> Self {
        let rasterizer = R::default();
        Canvas::new(SCREEN_PIXELS_X, SCREEN_PIXELS_Y, rasterizer)
    }
}

/// Convert from clip space to pixel space
/// Will return values outside of range `0..pixels` if value is outside range `-1.0..1.0`
/// TODO Check for weird behaviour if output is below range of `usize`
#[allow(dead_code)]
fn clip_to_pixel(clip_coord: f32, num_pixels: usize) -> usize {
    let pixel_width = 2.0 / num_pixels as f32;
    ((clip_coord + 1.0) / pixel_width).floor() as usize
}

/// Convert from the centre of a pixel to clip space
/// Will return value outside range if `pixel >= pixels` or `pixel < 0`
fn pixel_to_clip(pixel: usize, num_pixels: usize) -> f32 {
    let pixel_width = 2.0 / num_pixels as f32;
    (pixel as f32) * pixel_width + pixel_width / 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use crate::basic_rasterizer::BasicAsciiRasterizer;

    use super::*;
    use std::path::Path;

    #[test]
    /// Test that checks conversion from clip coordinates to pixels
    /// Test cases were scrutinised in separate text file.
    fn test_clip_to_pixel() {
        let pixels = 10;
        assert_eq!(clip_to_pixel(-1.0 + 0.1, pixels), 0);
        assert_eq!(clip_to_pixel(-1.0 + 0.21, pixels), 1);
        assert_eq!(clip_to_pixel(-1.0 + 1.99, pixels), 9);
    }
    #[test]
    /// Test that checks conversion from pixel to clip coordinates
    /// Test cases were scrutinised in separate text file.
    fn test_pixel_to_clip() {
        let pixels = 10;
        assert!((pixel_to_clip(0, pixels) - -0.9f32).abs() <= f32::EPSILON);
        assert!((pixel_to_clip(1, pixels) - -0.7f32).abs() <= f32::EPSILON);
        assert!((pixel_to_clip(9, pixels) - 0.9f32).abs() <= f32::EPSILON);
    }

    #[test]
    /// Test that loads in a mesh then renders it in a single position.
    fn test_drawing() {
        let test_obj = "./data/surface.obj";
        assert!(Path::new(test_obj).exists());

        let mut scene = Scene::default();
        scene.load_meshes_from_path(test_obj);
        let mut canvas = Canvas::<BasicAsciiRasterizer>::default();
        canvas.draw_scene_to_canvas(&scene);
    }
}
