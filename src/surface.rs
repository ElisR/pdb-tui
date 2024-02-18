use nalgebra::Point3;
use parry3d::mass_properties::MassProperties;
use parry3d::shape::TriMesh;
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
pub trait HasCenter {
    fn get_com(&self) -> Point3<f32>;
}

impl HasCenter for TriMesh {
    fn get_com(&self) -> Point3<f32> {
        MassProperties::from_trimesh(DEFAULT_DENSITY, self.vertices(), self.indices()).local_com
    }
}
/// Calculate center of many meshes
/// Returns the origin if vector is empty
impl HasCenter for Vec<TriMesh> {
    fn get_com(&self) -> Point3<f32> {
        let mass = self
            .iter()
            .map(|m| MassProperties::from_trimesh(DEFAULT_DENSITY, m.vertices(), m.indices()))
            .reduce(|sum_m, m| sum_m + m);
        match mass {
            Some(mp) => mp.local_com,
            None => Point3::origin(),
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
