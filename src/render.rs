use std::ops::Range;
// Create a the surface from a PDB file
use crate::surface::{SimpleMesh, Triangle, AABB};
use nalgebra::Perspective3;
use nalgebra::Unit;
use nalgebra::{Matrix4, Vector3, Vector4};
use nalgebra::{Point2, Point3};
// use pdbtbx::PDB;

// Constants for playing around with rendering
const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_PIXELS_X: u16 = 800;
const SCREEN_PIXELS_Y: u16 = 450;
const FOV: f32 = std::f32::consts::PI / 4.0; // Radians

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

// Calculate the dot product of a triangle's normal with a ray coming from the camera
// TODO Look into doing this where everything is turned into a slice instead
// TODO Change this so that camera always points along -z axis
pub fn orient(tri: &Triangle, ray: &Vector4<f32>) -> f32 {
    let normal = tri.normal();
    normal.dot(ray)
}

// Arguments order: aspect, fovy, znear, zfar.
// NOTE This function may be pointless because we may be doing collision detection in clip space
pub fn create_line(x: f32, y: f32) -> Vector4<f32> {
    // Defining a random point on the screen
    let screen_point = Point2::new(x, y);

    // Defining the projection from view space to clip space
    let projection = Perspective3::new(ASPECT_RATIO, FOV, 1.0, 10000.0);

    // Compute two points in clip-space.
    // "ndc" = normalized device coordinates.
    let near_ndc_point = Point3::new(
        screen_point.x / SCREEN_PIXELS_X as f32,
        screen_point.y / SCREEN_PIXELS_Y as f32,
        -1.0,
    );
    let far_ndc_point = Point3::new(
        screen_point.x / SCREEN_PIXELS_X as f32,
        screen_point.y / SCREEN_PIXELS_Y as f32,
        1.0,
    );

    // Unproject them to view-space.
    let near_view_point = projection.unproject_point(&near_ndc_point);
    let far_view_point = projection.unproject_point(&far_ndc_point);

    // Compute the view-space line parameters.
    // let line_location = near_view_point;
    let line_direction = Unit::new_normalize(far_view_point - near_view_point);

    // NOTE The view-space line parameters are relative to the camera
    line_direction.to_homogeneous()
}

// Where pixels will be printed
pub struct Canvas {
    pub frame_buffer: Vec<(char, (u8, u8, u8))>,
    pub z_buffer: Vec<f32>,
    pub width: u16,
    pub height: u16,
}

/// Holding geometric objects related to rendering
///
/// Holds camera position relative to world coordinates
/// Also holds list of all the light sources
pub struct Scene {
    pub view: Matrix4<f32>,
    pub lights: Vec<Vector4<f32>>,
    pub projection: Perspective3<f32>,
}
impl Scene {
    // TODO Work out the best way to pass lights: reference or directly
    fn new(
        eye: &Point3<f32>,
        target: &Point3<f32>,
        up: &Vector3<f32>,
        lights: &Vec<Vector3<f32>>,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = Matrix4::face_towards(eye, target, up);
        let lights = lights.iter().map(|v| v.to_homogeneous()).collect();
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
        let lights = vec![Vector3::new(0.0f32, -1.0f32, 1.0f32)];
        let znear = 1.0f32;
        let zfar = 100.0f32;
        Self::new(&eye, &target, &up, &lights, znear, zfar)
    }
}

/// Convert from clip space to pixel space
/// Will return values outside of range `0..pixels` if value is outside range `-1.0..1.0`
/// TODO Check for weird behaviour if output is below range of u16
/// NOTE Might want to use `i32` instead
fn clip_to_pixel(clip_coord: f32, pixels: u16) -> u16 {
    let pixel_width = 2.0 / pixels as f32;
    ((clip_coord + 1.0) / pixel_width).floor() as u16
}

/// Convert from the centre of a pixel to clip space
/// Will return value outside range if `pixel >= pixels` or `pixel < 0`
fn pixel_to_clip(pixel: u16, pixels: u16) -> f32 {
    let pixel_width = 2.0 / pixels as f32;
    (pixel as f32) * pixel_width + pixel_width / 2.0 - 1.0
}

/// Go from AABB, assumed to be in clip space, to x and y pixel ranges
fn get_pixel_ranges_from_aabb(aabb: AABB, width: u16, height: u16) -> (u16, u16, u16, u16) {
    // TODO Validate this max and min logic, don't want to give conservative range
    let x_min = clip_to_pixel(aabb.min[0], width).max(width).min(0);
    let y_min = clip_to_pixel(aabb.min[1], height).max(height).min(0);
    let x_max = clip_to_pixel(aabb.max[0], width).max(width).min(0);
    let y_max = clip_to_pixel(aabb.max[1], height).max(height).min(0);
    (x_min, x_max, y_min, y_max)
}

/// Finding the 1/z value where triangle and ray intersect
/// If 1/z == 0.0 then there is no intersection
// fn triange_pixel_collide_z(tri: &Triangle, x: f32, y: f32) -> f32 {

// }

fn draw_mesh_to_canvas(mesh: SimpleMesh, scene: Scene, canvas: Canvas) {
    let view_projection = scene.projection.as_matrix() * scene.view;
    for tri in mesh.triangles.iter() {
        let intensity: f32 = scene
            .lights
            .iter()
            .fold(0.0, |i, l| i + tri.normal().dot(l));
        let tri_clip = tri.new_mul(view_projection);
        let tri_clip_aabb = tri_clip.aabb();

        let (x_min, x_max, y_min, y_max) =
            get_pixel_ranges_from_aabb(tri_clip_aabb, canvas.width, canvas.height);

        for x in x_min..x_max {
            for y in y_min..y_max {
                let x_clip = pixel_to_clip(x, canvas.width);
                let y_clip = pixel_to_clip(y, canvas.height);
                // TODO Calculate ray for x and y then check for collisions with triangle
                // NOTE Probably have to use barycentric coordinates
                // NOTE Probably better to just use some predefined collision detection algorithms
            }
        }
    }

    // TODO For each triangle, loop through the pixels in its AABB and check for ray collision
    // If there is a collision, calculate z of collision and only update if smaller than existing buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_to_pixel() {
        let pixels: u16 = 10;
        assert_eq!(clip_to_pixel(-1.0 + 0.1, pixels), 0);
        assert_eq!(clip_to_pixel(-1.0 + 0.21, pixels), 1);
        assert_eq!(clip_to_pixel(-1.0 + 1.99, pixels), 9);
    }
    #[test]
    fn test_pixel_to_clip() {
        let pixels: u16 = 10;
        assert!((pixel_to_clip(0, pixels) - -0.9f32).abs() <= f32::EPSILON);
        assert!((pixel_to_clip(1, pixels) - -0.7f32).abs() <= f32::EPSILON);
        assert!((pixel_to_clip(9, pixels) - 0.9f32).abs() <= f32::EPSILON);
    }
    #[test]
    fn test_rasterizer() {
        let rasterizer = BasicAsciiRasterizer::default();
        assert_eq!(rasterizer.pixel_to_char(0.15), '.');
        assert_eq!(rasterizer.pixel_to_char(0.65), '*');
        assert_eq!(rasterizer.pixel_to_char(0.85), '%');
        assert_eq!(rasterizer.pixel_to_char(1.15), ' ');
    }
}
