// #![allow(dead_code)]
use crate::{
    read::{get_meshes_from_obj, get_shapes_from_pdb},
    surface::{ToTriMesh, ValidShape},
};
use nalgebra::{Isometry3, Perspective3, Point3, Vector3};
use parry3d::{
    query::{Ray, RayCast},
    shape::{Compound, TriMesh},
};
use std::path::Path;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
/// Default for FOV in radians
const FOVY: f32 = std::f32::consts::FRAC_PI_4;
const ZNEAR_DEFAULT: f32 = 1.0;
const ZFAR_DEFAULT: f32 = 100.0;

/// The ratio of height to width of terminal characters.
/// This depends on the font being used by the terminal emulator
const CHAR_ASPECT_RATIO: f32 = 2.0;

/// Take a point in 2D projection of clip space and convert to ray in world space
pub fn create_ray<S: RayCast + ValidShape>(x_clip: f32, y: f32, scene: &Scene<S>) -> Ray {
    // Compute two points in clip-space.
    let near_ndc_point = Point3::new(x_clip, y, -1.0);
    let far_ndc_point = Point3::new(x_clip, y, 1.0);

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
    let origin: Point3<f32> = scene.view.inverse() * near_view_point;
    // FIXME Turn this into unit normal to avoid TOI being incorrect
    // FIXME Check other places which assume maximum TOI
    let dir: Vector3<f32> = scene.view.inverse() * (far_view_point - near_view_point);
    // dir.normalize_mut();
    Ray::new(origin, dir)
}

/// Adjusts the aspect ratio for the projection according to non-square pixels
fn adjust_aspect(aspect_ratio: f32, char_aspect_ratio: f32) -> f32 {
    aspect_ratio / char_aspect_ratio
}

/// Wrapper struct holding the projection information defining the frustum shape
/// Needed to be able to implement default for quick testing
#[derive(Debug)]
pub struct SceneProjection {
    pub perspective: Perspective3<f32>,
}
impl SceneProjection {
    pub fn new(znear: f32, zfar: f32, aspect_ratio: f32, fovy: f32) -> Self {
        let adjusted_aspect_ratio = adjust_aspect(aspect_ratio, CHAR_ASPECT_RATIO);
        let perspective = Perspective3::new(adjusted_aspect_ratio, fovy, znear, zfar);
        SceneProjection { perspective }
    }
    /// Create new projection that fits meshes into `znear` and `zfar`
    /// Will resort to default `znear` and `zfar` if slice of meshes is empty
    // FIXME Update for new shapes defined in scene
    pub fn update_for_shapes(&mut self, shapes: &[(Isometry3<f32>, TriMesh)]) {
        // FIXME Have this not be tied to orientation, maybe by using sphere
        let znear = shapes
            .iter()
            .map(|(t, m)| m.aabb(t).mins.z)
            .reduce(f32::min)
            .unwrap_or(ZNEAR_DEFAULT);
        let zfar = shapes
            .iter()
            .map(|(t, m)| m.aabb(t).maxs.z)
            .reduce(f32::max)
            .unwrap_or(ZFAR_DEFAULT);
        self.perspective.set_znear_and_zfar(znear, zfar);
    }
}
impl Default for SceneProjection {
    fn default() -> Self {
        Self::new(ZNEAR_DEFAULT, ZFAR_DEFAULT, ASPECT_RATIO, FOVY)
    }
}

/// Holding geometric objects related to rendering
///
/// Holds camera position relative to world coordinates
/// Also holds list of all the light sources
// TODO Implement debug for this manually
// TODO Make generic according to different types of shape
pub struct Scene<S: RayCast = TriMesh> {
    pub view: Isometry3<f32>,
    /// Direction that the lights are pointing (as opposed to location of point source)
    pub lights: Vec<Vector3<f32>>,
    pub scene_projection: SceneProjection,
    shapes: Vec<(Isometry3<f32>, S)>,
}

impl<S: RayCast + ValidShape> Scene<S> {
    fn new(
        eye: &Point3<f32>,
        target: &Point3<f32>,
        up: &Vector3<f32>,
        lights: &[Vector3<f32>],
        scene_projection: SceneProjection,
        shapes: Vec<(Isometry3<f32>, S)>,
    ) -> Self {
        let view = Isometry3::face_towards(eye, target, up);
        let lights = lights.to_owned();
        Scene {
            view,
            lights,
            scene_projection,
            shapes,
        }
    }
    pub fn shapes(&self) -> &[(Isometry3<f32>, S)] {
        &self.shapes[..]
    }
    /// Change the scene projection according to new width and height of canvas
    pub fn update_aspect(&mut self, width: usize, height: usize) {
        let aspect_ratio = width as f32 / height as f32;
        let adjusted_aspect_ratio = adjust_aspect(aspect_ratio, CHAR_ASPECT_RATIO);
        self.scene_projection
            .perspective
            .set_aspect(adjusted_aspect_ratio);
    }
    /// Change the view according to transformation
    pub fn transform_view(&mut self, transform: &Isometry3<f32>) {
        self.view = transform * self.view;
    }
    /// Transform shapes by a transformation
    /// Internally, prepends trasnformation to existing internal transformation
    pub fn transform_shapes(&mut self, transform: &Isometry3<f32>) {
        for (og_transform, _) in self.shapes.iter_mut() {
            *og_transform = transform * *og_transform;
        }
    }
    /// Make the mesh be at the center of the view
    pub fn shapes_to_center(&mut self) {
        let com = self.shapes.get_com();
        let transform = Isometry3::translation(-com.x, -com.y, -com.z);
        self.transform_shapes(&transform);
    }
    /// Resetting the view to point at the center-of-mass of the meshes
    // TODO Write this function
    pub fn reset_eye_to_com(&mut self) {
        todo!();
    }
}

impl Scene<TriMesh> {
    /// Adds meshes found at path to existing meshes vector
    pub fn load_meshes_from_path<Q: AsRef<Path>>(&mut self, path: Q) {
        let tobj_meshes = get_meshes_from_obj(path);
        let mut new_meshes = tobj_meshes
            .iter()
            .map(|m| m.to_tri_mesh())
            .map(|m| (Isometry3::identity(), m))
            .collect();
        self.shapes.append(&mut new_meshes);
        self.scene_projection.update_for_shapes(&self.shapes);
    }
}

impl Scene<Compound> {
    // TODO Add proper signature
    pub fn load_shapes_from_pdb<Q: AsRef<str>>(&mut self, path: Q) {
        let compounds = get_shapes_from_pdb(path);
        let mut shapes: Vec<(Isometry3<f32>, Compound)> = compounds
            .into_iter()
            .map(|c| (Isometry3::<f32>::identity(), c))
            .collect();
        self.shapes.append(&mut shapes);
        // FIXME Make this work
        // self.scene_projection.update_for_shapes(&self.shapes);
    }
}

impl<S: RayCast + ValidShape> Default for Scene<S> {
    fn default() -> Self {
        let eye = Point3::new(0.0f32, 0.0f32, -50.0f32);
        let target = Point3::new(0.0f32, 0.0f32, 0.0f32);
        let up = Vector3::new(0.0f32, 1.0f32, 0.0f32);
        let lights = vec![
            0.7 * Vector3::new(0.0f32, 1.0f32, 1.0f32),
            // Vector3::new(0.0f32, -1.0f32, -1.0f32),
        ];
        let scene_projection = SceneProjection::default();
        let shapes = vec![];
        Self::new(&eye, &target, &up, &lights, scene_projection, shapes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parry3d::shape::{Ball, SharedShape};
    use std::sync::Arc;

    #[test]
    fn create_compound_shape() {
        let mut scene = Scene::<Compound>::default();

        let sphere_1 = SharedShape(Arc::new(Ball::new(10.0)));
        let sphere_2 = SharedShape(Arc::new(Ball::new(15.0)));

        let t = Isometry3::<f32>::translation(15.0, 0.0, 0.0);
        let shapes = vec![(Isometry3::<f32>::identity(), sphere_1), (t, sphere_2)];

        let combo = Compound::new(shapes);
        scene.shapes.push((Isometry3::<f32>::identity(), combo));

        assert_eq!(scene.shapes.len(), 1)
    }
}
