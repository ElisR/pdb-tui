use nalgebra::Isometry3;
use parry3d::shape::{Ball, Compound, SharedShape};
use pdbtbx::{open_pdb, Atom, StrictnessLevel};
use std::path::Path;
use std::sync::Arc;
use tobj::{load_obj, LoadOptions, Mesh, Model};

pub const CARBON_RADIUS: f32 = 3.0;

pub struct PDBStructure {
    pub chains: u16,
}

// TODO Make this return a Result type
pub fn get_models_from_obj<Q>(path: Q) -> Vec<Model>
where
    Q: AsRef<Path>,
{
    assert!(path.as_ref().exists());
    // TODO Swap out unwrap for something
    let (models, _materials) = load_obj(path.as_ref(), &LoadOptions::default()).unwrap();
    models
}

pub fn get_meshes_from_obj<Q>(path: Q) -> Vec<Mesh>
where
    Q: AsRef<Path>,
{
    let models = get_models_from_obj(path);
    models.into_iter().map(|model| model.mesh).collect()
}

// TODO Decide on a radius for each atom type
pub fn get_compound_from_atoms(atoms: &[&Atom]) -> Compound {
    let mut balls = vec![];

    for atom in atoms.iter() {
        let sphere = SharedShape(Arc::new(Ball::new(CARBON_RADIUS)));
        let t = Isometry3::translation(atom.x() as f32, atom.y() as f32, atom.z() as f32);

        balls.push((t, sphere));
    }
    Compound::new(balls)
}

/// Create compound shapes for each chain in the PDB
pub fn get_shapes_from_pdb<Q>(path: Q) -> Vec<Compound>
where
    Q: AsRef<str>,
{
    // TODO Handle errors correctly
    // PDBtbx library does not expect `AsRef<Path>` but rather `AsRef<str>`!
    let (pdb, _errors) = open_pdb(path, StrictnessLevel::Medium).unwrap();

    let bb_atoms: Vec<Vec<&Atom>> = pdb
        .chains()
        .map(|c| c.atoms().filter(|a| a.is_backbone()).collect())
        .collect();
    bb_atoms
        .iter()
        .map(|atoms| get_compound_from_atoms(&atoms[..]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // read in a test file and deserialize it
    fn test_reading_pdb() {
        let test_pdb = "./data/rbd.pdb";
        assert!(Path::new(test_pdb).exists());

        let pdb = open_pdb(test_pdb, StrictnessLevel::Medium);
        assert!(pdb.is_ok())
    }

    #[test]
    fn test_reading_obj() {
        let test_obj = "./data/surface.obj";
        assert!(Path::new(test_obj).exists());

        let (models, _materials) = tobj::load_obj(test_obj, &tobj::LoadOptions::default())
            .expect("Failed to OBJ load file");

        println!("Number of models          = {}", models.len());
        for (i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            println!("model[{}].name = \'{}\'", i, m.name);
            println!("model[{}].face_count = {}", i, mesh.face_arities.len());
            println!("model[{}].indices = {}", i, mesh.indices.len())
        }
    }
}
