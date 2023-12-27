// use pdb_tui::read::PDBStructure;
// use nalgebra::Vector4;
use pdb_tui::render::create_line;
// use pdbtbx::*;

fn main() {
    // Playing around with lines
    let line = create_line();
    println!("{}", line)

    // TODO Load in a PDB file

    // let mut avg_b_factor = 0.0;
    // for atom in pdb.atoms() {
    //     avg_b_factor += atom.b_factor()
    // }
    // avg_b_factor /= pdb.atom_count() as f64;

    // TODO Create a molecular surface for the PDB
    // NOTE Should firstly look into how the rendering will work

    // TODO Look into Termion for a way to render PDB

    // TODO Make each chain have its own colour
}

// TODO Add some tests for basic things
