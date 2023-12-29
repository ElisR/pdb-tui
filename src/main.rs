// use pdb_tui::read::PDBStructure;
// use nalgebra::Vector4;
use pdb_tui::render::create_line;
// use pdbtbx::*;

fn main() {
    // Playing around with lines
    let line = create_line(10.0f32, 20.0f32);
    println!("{}", line)

    // TODO Load in a PDB file

    // TODO Create a molecular surface for the PDB
    // NOTE Should firstly look into how the rendering will work
    // TODO Make each chain have its own colour

    // TODO Set up scene and render

    // TODO Look into Termion for a way to render PDB
}

// TODO Add some tests for basic things
