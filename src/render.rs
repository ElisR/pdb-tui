// Create a the surface from a PDB file
use nalgebra::Perspective3;
use nalgebra::Unit;
use nalgebra::{Matrix4, Vector4};
use nalgebra::{Point2, Point3};
// use pdbtbx::PDB;

// Constants for playing around with rendering
const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_PIXELS_X: u16 = 800;
const SCREEN_PIXELS_Y: u16 = 450;
const FOV: f32 = std::f32::consts::PI / 4.0; // Radians

// Arguments order: aspect, fovy, znear, zfar.
pub fn create_line() -> Vector4<f32> {
    // Defining the projection from view space to clip space
    let projection = Perspective3::new(ASPECT_RATIO, FOV, 1.0, 10000.0);

    // Defining a random point on the screen
    let screen_point = Point2::new(10.0f32, 20.0f32);

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

// pub fn create_slice_pointer() {
//     let line = create_line();

//     let slice = line.as_slice();
//     let pointer = slice.as_ptr();

//     pointer
// }
