use nalgebra::{Isometry3, Point3};
use parry3d::mass_properties::MassProperties;
use parry3d::shape::{Compound, Shape, TriMesh};
use tobj::Mesh;

const DEFAULT_DENSITY: f32 = 1.0;

/// Using the `parry` implementation of meshes
pub trait ToTriMesh {
    fn to_tri_mesh(&self) -> TriMesh;
}
impl ToTriMesh for Mesh {
    fn to_tri_mesh(&self) -> TriMesh {
        let indices: Vec<[u32; 3]> = self.indices.chunks(3).map(|t| [t[0], t[1], t[2]]).collect();
        let positions: Vec<Point3<f32>> = self
            .positions
            .chunks(3)
            .map(|t| Point3::new(t[0], t[1], t[2]))
            .collect();
        TriMesh::new(positions, indices)
    }
}

/// Trait for something whose center can be calculated
pub trait ValidShape {
    fn mass_properties_default(&self) -> Option<MassProperties>;
    fn get_com(&self) -> Point3<f32> {
        match self.mass_properties_default() {
            Some(mp) => mp.local_com,
            None => Point3::origin(),
        }
    }
    /// Transform shapes according to transformation
    fn transform(&mut self, transform: &Isometry3<f32>);
}

impl ValidShape for TriMesh {
    fn mass_properties_default(&self) -> Option<MassProperties> {
        Some(self.mass_properties(DEFAULT_DENSITY))
    }
    fn transform(&mut self, transform: &Isometry3<f32>) {
        self.transform_vertices(transform);
    }
}

impl ValidShape for Compound {
    fn mass_properties_default(&self) -> Option<MassProperties> {
        Some(self.mass_properties(DEFAULT_DENSITY))
    }
    fn transform(&mut self, _transform: &Isometry3<f32>) {
        // TODO Write this function
        // Currently difficult without copying the entirety
        todo!()
    }
}

/// Calculate center of many shapes
/// Returns the origin if vector is empty
impl<S: ValidShape> ValidShape for Vec<S> {
    fn mass_properties_default(&self) -> Option<MassProperties> {
        self.iter()
            .filter_map(|m| m.mass_properties_default())
            .reduce(|sum_m, m| sum_m + m)
    }
    fn transform(&mut self, transform: &Isometry3<f32>) {
        for shape in self.iter_mut() {
            shape.transform(transform)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::get_meshes_from_obj;
    use std::path::Path;

    #[test]
    fn test_reading_and_conversion() {
        let test_obj = "./data/surface.obj";
        assert!(Path::new(test_obj).exists());

        let meshes = get_meshes_from_obj(test_obj);
        let tri_mesh = meshes[0].to_tri_mesh();

        assert!(!tri_mesh.indices().is_empty());
    }
}
