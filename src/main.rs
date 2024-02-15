#![allow(dead_code)]

use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};
use pdb_tui::rasterizer::BasicAsciiRasterizer;
use pdb_tui::render::{Canvas, Scene};

use std::time::Instant;

fn main() {
    // Playing around with lines
    // let line = create_ray(10.0f32, 20.0f32);
    // println!("{}", line)

    // TODO Load in a PDB file

    // TODO Create a molecular surface for the PDB
    // NOTE Should firstly look into how the rendering will work
    // TODO Make each chain have its own colour

    // TODO Set up scene and render

    // TODO Look into Termion for a way to render PDB

    // let meshes = get_meshes_from_obj(test_obj);
    // let mesh = &meshes[0];
    // let mut mesh = mesh.to_tri_mesh();

    let test_obj = "./data/surface.obj";
    let mut scene = Scene::default();
    scene.load_meshes_from_path(test_obj);

    let translation = Translation3::new(15.0f32, 15.0f32, -20.0f32);
    let rotation = UnitQuaternion::from_scaled_axis(Vector3::y() * std::f32::consts::FRAC_PI_8);
    let transform = Isometry3::from_parts(translation, rotation);
    scene.transform_meshes(&transform);

    let mut canvas = Canvas::<BasicAsciiRasterizer>::default();

    let now = Instant::now();
    println!("Starting to draw.");
    canvas.draw_scene_to_canvas(&scene);
    let new_now = Instant::now();
    println!("Drawn after {:?}", new_now.duration_since(now));

    // Print output to stdout
    let frame_buffer = canvas.frame_buffer.clone();
    let stdout: String = frame_buffer.iter().collect();
    print!("{}", stdout);

    let x_shift = Isometry3::from_parts(
        Translation3::new(5.0f32, 0f32, 0f32),
        UnitQuaternion::identity(),
    );
    for i in 0..10 {
        scene.transform_meshes(&x_shift);
        canvas.draw_scene_to_canvas(&scene);

        let path = format!("canvas_{}.png", i);
        canvas.save_image(path).unwrap();
    }
}

// TODO Add some tests for basic things
