#![allow(dead_code)]
use std::path::Path;

use crate::rasterizer::Rasterizer;
use crate::read::get_meshes_from_obj;
use crate::surface::ToTriMesh;
use image::imageops::flip_vertical_in_place;
use image::{GrayImage, ImageResult};
// use std::ops::Range;
// Create a the surface from a PDB file
// use crate::surface::SimpleMesh;
use nalgebra::{Isometry3, Perspective3, Point3};
// use nalgebra::Unit;
// use nalgebra::UnitVector3;
use nalgebra::{Matrix4, Vector3};
// use pdbtbx::PDB;
use parry3d::query::{Ray, RayCast};
use parry3d::shape::TriMesh;

// Constants for playing around with rendering
const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_PIXELS_X: usize = 320;
const SCREEN_PIXELS_Y: usize = 180;
const FOV: f32 = std::f32::consts::FRAC_PI_4; // Radians

pub fn create_ray(x: f32, y: f32, scene: &Scene) -> (Point3<f32>, Vector3<f32>) {
    // Compute two points in clip-space.
    let near_ndc_point = Point3::new(x, y, -1.0);
    let far_ndc_point = Point3::new(x, y, 1.0);

    // Unproject them to view-space.
    let near_view_point = scene
        .scene_projection
        .perspective
        .unproject_point(&near_ndc_point);
    let far_view_point = scene
        .scene_projection
        .perspective
        .unproject_point(&far_ndc_point);

    // Compute the view-space line parameters.
    let line_location = near_view_point;
    // FIXME Turn this into unit normal to avoid TOI being incorrect (currently difficult because of types)
    // let line_direction = Unit::new_normalize(far_view_point - near_view_point);
    let line_direction = far_view_point - near_view_point;
    (line_location, line_direction)
}

/// Wrapper struct holding the projection information defining the frustrum shape
/// Needed to be able to implement default for quick testing
pub struct SceneProjection {
    pub perspective: Perspective3<f32>,
}
impl SceneProjection {
    fn new(znear: f32, zfar: f32, aspect_ratio: f32, fovy: f32) -> Self {
        let perspective = Perspective3::new(aspect_ratio, fovy, znear, zfar);
        SceneProjection { perspective }
    }
}
impl Default for SceneProjection {
    fn default() -> Self {
        let znear = 1.0f32;
        let zfar = 100.0f32;
        let aspect_ratio = ASPECT_RATIO;
        let fovy = FOV;
        Self::new(znear, zfar, aspect_ratio, fovy)
    }
}

/// Holding geometric objects related to rendering
///
/// Holds camera position relative to world coordinates
/// Also holds list of all the light sources
pub struct Scene {
    pub view: Matrix4<f32>,
    pub lights: Vec<Vector3<f32>>,
    pub scene_projection: SceneProjection,
    meshes: Vec<TriMesh>,
}
impl Scene {
    fn new(
        eye: &Point3<f32>,
        target: &Point3<f32>,
        up: &Vector3<f32>,
        lights: &[Vector3<f32>],
        scene_projection: SceneProjection,
        meshes: Vec<TriMesh>,
    ) -> Self {
        let view = Matrix4::face_towards(eye, target, up);
        let lights = lights.to_owned();
        Scene {
            view,
            lights,
            scene_projection,
            meshes,
        }
    }
    pub fn load_meshes_from_path<Q: AsRef<Path>>(&mut self, path: Q) {
        let tobj_meshes = get_meshes_from_obj(path);
        self.meshes = tobj_meshes.iter().map(|m| m.to_tri_mesh()).collect();
    }

    /// Transform meshes according to tranformation
    pub fn transform_meshes(&mut self, transform: &Isometry3<f32>) {
        for mesh in self.meshes.iter_mut() {
            mesh.transform_vertices(transform);
        }
    }
}
impl Default for Scene {
    fn default() -> Self {
        let eye = Point3::new(0.0f32, 0.0f32, -10.0f32);
        let target = Point3::new(0.0f32, 0.0f32, 0.0f32);
        let up = Vector3::new(0.0f32, 1.0f32, 0.0f32);
        let lights = vec![
            0.7 * Vector3::new(0.0f32, 1.0f32, 1.0f32),
            // Vector3::new(0.0f32, -1.0f32, -1.0f32),
        ];
        let scene_projection = SceneProjection::default();

        // let test_obj = "./data/surface.obj";
        // let meshes = get_meshes_from_obj(test_obj);
        // let meshes = meshes.iter().map(|m| m.to_tri_mesh()).collect();
        let meshes = vec![];
        Self::new(&eye, &target, &up, &lights, scene_projection, meshes)
    }
}

pub enum CanvasError {
    PixelOutOfRange { x: usize, y: usize },
}

pub struct Canvas<R: Rasterizer> {
    pub frame_buffer: Vec<char>,
    pub pixel_buffer: Vec<f32>,
    pub toi_buffer: Vec<f32>,
    // TODO Don't make these public, so that they can't be trivially updated and break buffer sizes
    pub width: usize,
    pub height: usize,
    pub rasterizer: R,
    /// Pixel intensity used for the background
    pub bg_pixel: f32,
}
impl<R: Rasterizer + Default> Canvas<R> {
    /// Constructor for canvas
    pub fn new(width: usize, height: usize) -> Self {
        let bg_pixel = 1.1f32;

        // TODO Allow custom rasterizer to be passed into constructor
        let rasterizer = R::default();

        let size = width * height;
        let frame_buffer = vec![rasterizer.get_bg_char(); size];
        let pixel_buffer = vec![bg_pixel; size];
        let toi_buffer = vec![f32::MAX; size];
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
    /// Update the frame buffer with whatever the pixel buffer is set to
    pub fn update_frame(&mut self) {
        self.frame_buffer = self.rasterizer.pixels_to_stdout(self.reshaped_pixels())
    }

    /// Reshape the vector of pixels to a 2D vector that can be accepted by `Rasterizer`
    fn reshaped_pixels(&self) -> Vec<&[f32]> {
        self.pixel_buffer.chunks(self.width).collect()
    }

    /// Utility function for calculating index, given pixel location
    fn pixel_to_index(&self, x: usize, y: usize) -> Result<usize, CanvasError> {
        // This makes the most sense because then horizontally adjacent characters adjacent in memory
        if x < self.width && y < self.width {
            Ok(y * self.width + x)
        } else {
            Err(CanvasError::PixelOutOfRange { x, y })
        }
    }
    /// Set a pixel unconditionally
    /// Will do nothing if pixel out of range
    pub fn set_pixel(&mut self, x: usize, y: usize, val: f32) {
        match self.pixel_to_index(x, y) {
            Ok(idx) => {
                self.pixel_buffer[idx] = val;
            }
            Err(_e) => {}
        }
    }
    /// Set a pixel conditional on time-of-impact being lower than current buffer value
    /// Also updates time-of-impact buffer
    /// Will do nothing if pixel out of range
    pub fn set_pixel_toi(&mut self, x: usize, y: usize, val: f32, toi: f32) {
        match self.pixel_to_index(x, y) {
            Ok(idx) => {
                if toi < self.toi_buffer[idx] {
                    self.pixel_buffer[idx] = val;
                    self.toi_buffer[idx] = toi;
                }
            }
            Err(_e) => {}
        }
    }
    pub fn set_toi(&mut self, x: usize, y: usize, toi: f32) {
        match self.pixel_to_index(x, y) {
            Ok(idx) => {
                self.toi_buffer[idx] = toi;
            }
            Err(_e) => {}
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
    pub fn draw_scene_to_canvas(&mut self, scene: &Scene) {
        self.flush_buffers();

        for x in 0..self.width {
            for y in 0..self.height {
                let x_clip = pixel_to_clip(x, self.width);
                let y_clip = pixel_to_clip(y, self.height);

                let (ray_loc, ray_dir) = create_ray(x_clip, y_clip, scene);
                let ray: Ray = Ray::new(ray_loc, ray_dir);

                for mesh in scene.meshes.iter() {
                    // FIXME Make sure max_toi is reasonable
                    let toi_result = mesh.cast_local_ray_and_get_normal(
                        &ray,
                        scene.scene_projection.perspective.zfar() + 100.0,
                        true,
                    );
                    // TODO Consider whether we should take `abs` of intensity
                    // FIXME Make sure background is returned if no collision
                    if let Some(ri) = toi_result {
                        let normal = ri.normal;
                        let intensity: f32 =
                            scene.lights.iter().fold(0.0, |i, l| i + normal.dot(l));
                        self.set_pixel_toi(x, y, intensity, ri.toi);
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
        let pixels_transformed = self
            .pixel_buffer
            .iter()
            .map(|i| (i * 255.0).round() as u8)
            .collect();
        let mut image_buffer =
            GrayImage::from_raw(self.width as u32, self.height as u32, pixels_transformed).unwrap();
        // Flip because small coord means small index, but top of image should have large y
        flip_vertical_in_place(&mut image_buffer);
        image_buffer.save(path)
    }
}

impl<R: Rasterizer + Default> Default for Canvas<R> {
    fn default() -> Self {
        Canvas::new(SCREEN_PIXELS_X, SCREEN_PIXELS_Y)
    }
}

/// Convert from clip space to pixel space
/// Will return values outside of range `0..pixels` if value is outside range `-1.0..1.0`
/// TODO Check for weird behaviour if output is below range of u16
/// NOTE Might want to use `i32` instead
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
    use crate::rasterizer::BasicAsciiRasterizer;

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
