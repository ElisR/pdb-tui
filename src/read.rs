use std::path::Path;
use tobj::{load_obj, LoadOptions, Mesh, Model};

pub struct PDBStructure {
    pub chains: u16,
}

// TODO Make this return a Result type
pub fn get_models_from_obj(path: &str) -> Vec<Model> {
    assert!(Path::new(path).exists());
    let (models, _materials) =
        load_obj(path, &LoadOptions::default()).expect("Failed to OBJ load file");
    models
}

pub fn get_meshes_from_obj(path: &str) -> Vec<Mesh> {
    let models = get_models_from_obj(path);
    models.into_iter().map(|model| model.mesh).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdbtbx::*;

    #[test]
    // read in a test file and deserialize it
    fn test_reading_pdb() {
        let test_pdb = "./data/rbd.pdb";
        assert!(Path::new(test_pdb).exists());

        let pdb = pdbtbx::open_pdb(test_pdb, StrictnessLevel::Medium);
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
