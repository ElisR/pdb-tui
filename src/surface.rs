use nalgebra::Unit;
use nalgebra::{Matrix4, Point3, Vector4};
use parry3d::shape::TriMesh;
use std::clone::Clone;
use tobj::Mesh;

#[derive(PartialEq, Debug)]
pub struct AABB {
    pub min: Vector4<f32>,
    pub max: Vector4<f32>,
}

/// Axis-aligned bounding box
impl AABB {
    pub fn new(min: Vector4<f32>, max: Vector4<f32>) -> AABB {
        AABB { min, max }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Triangle {
    pub color: (u8, u8, u8),
    pub v1: Vector4<f32>,
    pub v2: Vector4<f32>,
    pub v3: Vector4<f32>,
}

impl Triangle {
    pub fn aabb(&self) -> AABB {
        AABB::new(
            Vector4::from_fn(|i, _size| self.v1[i].min(self.v2[i].min(self.v3[i]))),
            Vector4::from_fn(|i, _size| self.v1[i].max(self.v2[i].max(self.v3[i]))),
        )
    }
    /// Transform and mutate the triangle
    pub fn mul(&mut self, transform: Matrix4<f32>) -> &mut Self {
        self.v1 = transform * self.v1;
        self.v2 = transform * self.v2;
        self.v3 = transform * self.v3;
        self
    }
    /// Transform a triangle and return a new copy
    pub fn new_mul(self, transform: Matrix4<f32>) -> Self {
        Self {
            color: self.color,
            v1: transform * self.v1,
            v2: transform * self.v2,
            v3: transform * self.v3,
        }
    }
    pub fn normal(&self) -> Unit<Vector4<f32>> {
        // Two triangle edges
        let u1 = self.v2 - self.v1;
        let u2 = self.v3 - self.v1;
        // Cross product
        let x = (u1[1] * u2[2]) - (u1[2] * u2[1]);
        let y = (u1[2] * u2[0]) - (u1[0] * u2[2]);
        let z = (u1[0] * u2[1]) - (u1[1] * u2[0]);
        Unit::new_normalize(Vector4::new(x, y, z, 0.0))
    }
}

#[allow(dead_code)]
pub struct SimpleMesh {
    pub bounding_box: AABB,
    pub triangles: Vec<Triangle>,
}

pub trait ToSimpleMesh {
    fn to_simple_mesh(&self) -> SimpleMesh;
}
impl ToSimpleMesh for Mesh {
    fn to_simple_mesh(&self) -> SimpleMesh {
        let mut bounding_box = AABB {
            min: Vector4::new(0.0, 0.0, 0.0, 1.0),
            max: Vector4::new(0.0, 0.0, 0.0, 1.0),
        };
        let mut triangles = vec![
            Triangle {
                color: (1, 1, 1),
                v1: Vector4::new(0.0, 0.0, 0.0, 1.0),
                v2: Vector4::new(0.0, 0.0, 0.0, 1.0),
                v3: Vector4::new(0.0, 0.0, 0.0, 1.0)
            };
            self.indices.len() / 3
        ];
        for (i, tri) in triangles.iter_mut().enumerate() {
            tri.v1.x = self.positions[(self.indices[i * 3] * 3) as usize];
            tri.v1.y = self.positions[(self.indices[i * 3] * 3 + 1) as usize];
            tri.v1.z = self.positions[(self.indices[i * 3] * 3 + 2) as usize];
            tri.v2.x = self.positions[(self.indices[i * 3 + 1] * 3) as usize];
            tri.v2.y = self.positions[(self.indices[i * 3 + 1] * 3 + 1) as usize];
            tri.v2.z = self.positions[(self.indices[i * 3 + 1] * 3 + 2) as usize];
            tri.v3.x = self.positions[(self.indices[i * 3 + 2] * 3) as usize];
            tri.v3.y = self.positions[(self.indices[i * 3 + 2] * 3 + 1) as usize];
            tri.v3.z = self.positions[(self.indices[i * 3 + 2] * 3 + 2) as usize];

            let aabb = tri.aabb();
            // Loop through x, y and z components
            for dim in 0..3 {
                bounding_box.min[dim] = aabb.min[dim].min(bounding_box.min[dim]);
                bounding_box.max[dim] = aabb.max[dim].max(bounding_box.max[dim]);
            }
        }
        SimpleMesh {
            triangles,
            bounding_box,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::get_meshes_from_obj;
    use std::path::Path;

    // Stupid test just to check that vector indexing works as expected
    #[test]
    fn check_indexing_of_vectors() {
        let vec = Vector4::new(1.0, 2.0, 0.0, 1.0);
        assert_eq!(vec.x, vec[0]);
        assert_eq!(vec.y, vec[1]);
    }

    #[test]
    fn test_reading_and_conversion() {
        let test_obj = "./data/surface.obj";
        assert!(Path::new(test_obj).exists());

        let meshes = get_meshes_from_obj(test_obj);
        let simple_mesh = meshes[0].to_simple_mesh();

        assert!(!simple_mesh.triangles.is_empty());

        // Print first few triangles for debugging purposes
        for i in 0..3 {
            let triangle = &simple_mesh.triangles[i];
            println!(
                "v1 = {}, v2 = {}, v3 = {}",
                triangle.v1, triangle.v2, triangle.v3
            );
        }
    }
}
