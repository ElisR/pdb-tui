#![allow(dead_code)]
use crate::rasterizer::Rasterizer;
// use std::ops::Range;
// Create a the surface from a PDB file
// use crate::surface::SimpleMesh;
use nalgebra::Perspective3;
use nalgebra::Point3;
// use nalgebra::Unit;
// use nalgebra::UnitVector3;
use nalgebra::{Matrix4, Vector3};
// use pdbtbx::PDB;
use parry3d::query::Ray;
use parry3d::query::RayCast;
use parry3d::shape::TriMesh;

// Constants for playing around with rendering
const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_PIXELS_X: usize = 320;
const SCREEN_PIXELS_Y: usize = 180;
const FOV: f32 = std::f32::consts::PI / 4.0; // Radians

pub fn create_ray(x: f32, y: f32, scene: &Scene) -> (Point3<f32>, Vector3<f32>) {
    // Compute two points in clip-space.
    let near_ndc_point = Point3::new(x, y, -1.0);
    let far_ndc_point = Point3::new(x, y, 1.0);

    // Unproject them to view-space.
    let near_view_point = scene.projection.unproject_point(&near_ndc_point);
    let far_view_point = scene.projection.unproject_point(&far_ndc_point);

    // Compute the view-space line parameters.
    let line_location = near_view_point;
    // FIXME Turn this into unit normal to avoid TOI being incorrect (currently difficult because of types)
    // let line_direction = Unit::new_normalize(far_view_point - near_view_point);
    let line_direction = far_view_point - near_view_point;
    (line_location, line_direction)
}

pub enum CanvasError {
    PixelOutOfRange { x: usize, y: usize },
}

// Where pixels will be printed
pub struct Canvas<R: Rasterizer> {
    pub frame_buffer: Vec<char>,
    pub pixel_buffer: Vec<f32>,
    pub toi_buffer: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub rasterizer: R,
}
impl<R: Rasterizer + Default> Canvas<R> {
    /// Constructor for canvas
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let frame_buffer = vec![' '; size];
        let pixel_buffer = vec![0f32; size];
        // FIXME Replace with proper TOI default
        let toi_buffer = vec![0f32; size];
        // TODO Allow custom rasterizer to be passed into constructor
        let rasterizer = R::default();
        Canvas {
            frame_buffer,
            pixel_buffer,
            toi_buffer,
            width,
            height,
            rasterizer,
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
    pub fn set_pixel(&mut self, x: usize, y: usize, val: f32) {
        match self.pixel_to_index(x, y) {
            Ok(idx) => {
                self.pixel_buffer[idx] = val;
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
}

impl<R: Rasterizer + Default> Default for Canvas<R> {
    fn default() -> Self {
        Canvas::new(SCREEN_PIXELS_X, SCREEN_PIXELS_Y)
    }
}

/// Holding geometric objects related to rendering
///
/// Holds camera position relative to world coordinates
/// Also holds list of all the light sources
pub struct Scene {
    pub view: Matrix4<f32>,
    pub lights: Vec<Vector3<f32>>,
    pub projection: Perspective3<f32>,
}
impl Scene {
    // TODO Work out the best way to pass lights: reference or directly
    fn new(
        eye: &Point3<f32>,
        target: &Point3<f32>,
        up: &Vector3<f32>,
        lights: &[Vector3<f32>],
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = Matrix4::face_towards(eye, target, up);
        let lights = lights.to_owned();
        // TODO Swap out global aspect ratio and fov for something else
        let projection = Perspective3::new(ASPECT_RATIO, FOV, znear, zfar);
        Scene {
            view,
            lights,
            projection,
        }
    }
}
impl Default for Scene {
    fn default() -> Self {
        let eye = Point3::new(0.0f32, 0.0f32, -10.0f32);
        let target = Point3::new(0.0f32, 0.0f32, 0.0f32);
        let up = Vector3::new(0.0f32, 1.0f32, 0.0f32);
        let lights = vec![
            0.7 * Vector3::new(0.0f32, -1.0f32, 1.0f32),
            // Vector3::new(0.0f32, -1.0f32, -1.0f32),
        ];
        let znear = 1.0f32;
        let zfar = 100.0f32;
        Self::new(&eye, &target, &up, &lights, znear, zfar)
    }
}

/// Convert from clip space to pixel space
/// Will return values outside of range `0..pixels` if value is outside range `-1.0..1.0`
/// TODO Check for weird behaviour if output is below range of u16
/// NOTE Might want to use `i32` instead
fn clip_to_pixel(clip_coord: f32, pixels: usize) -> usize {
    let pixel_width = 2.0 / pixels as f32;
    ((clip_coord + 1.0) / pixel_width).floor() as usize
}

/// Convert from the centre of a pixel to clip space
/// Will return value outside range if `pixel >= pixels` or `pixel < 0`
fn pixel_to_clip(pixel: usize, pixels: usize) -> f32 {
    let pixel_width = 2.0 / pixels as f32;
    (pixel as f32) * pixel_width + pixel_width / 2.0 - 1.0
}

pub fn draw_trimesh_to_canvas<R: Rasterizer + Default>(
    mesh: &TriMesh,
    scene: &Scene,
    canvas: &mut Canvas<R>,
) {
    // TODO Define the model transformation somewhere
    for x in 0..canvas.width {
        for y in 0..canvas.width {
            let x_clip = pixel_to_clip(x, canvas.width);
            let y_clip = pixel_to_clip(y, canvas.height);

            let (ray_loc, ray_dir) = create_ray(x_clip, y_clip, scene);
            let ray: Ray = Ray::new(ray_loc, ray_dir);

            // FIXME Make sure max_toi is reasonable
            let toi_result =
                mesh.cast_local_ray_and_get_normal(&ray, scene.projection.zfar() + 100.0, true);
            // TODO Consider whether we should take `abs` of intensity
            if let Some(ri) = toi_result {
                let normal = ri.normal;
                let intensity: f32 = scene.lights.iter().fold(0.0, |i, l| i + normal.dot(l));
                canvas.set_pixel(x, y, intensity);
                canvas.set_toi(x, y, ri.toi);
            }
        }
    }
    canvas.update_frame()
}

#[cfg(test)]
mod tests {
    use crate::rasterizer::BasicAsciiRasterizer;
    use crate::read::get_meshes_from_obj;
    use crate::surface::ToTriMesh;

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

        // let (models, _materials) = tobj::load_obj(test_obj, &tobj::LoadOptions::default())
        //     .expect("Failed to OBJ load file");
        let meshes = get_meshes_from_obj(test_obj);
        let mesh = &meshes[0];
        let mesh = mesh.to_tri_mesh();

        let scene = Scene::default();

        // TODO Create canvas
        let mut canvas = Canvas::<BasicAsciiRasterizer>::default();

        draw_trimesh_to_canvas(&mesh, &scene, &mut canvas);
    }
}
