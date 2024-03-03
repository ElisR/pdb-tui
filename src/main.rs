#![allow(dead_code)]
use ab_glyph::InvalidFont;
use clap::Parser;
// use pdb_tui::tui::ui::{run, shutdown, startup};
// use std::io::Result;

use pdb_tui::ascii::rasterize::draw_chars;

/// Program to render PDBs within a terminal user interface
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDB file to be loaded
    #[arg(short, long, num_args=1.., default_value = "./data/rbd.pdb")]
    inputs: Vec<String>,
}

fn main() -> Result<(), InvalidFont> {
    // let args = Args::parse();
    // startup()?;
    // let result = run(args.inputs);
    // shutdown()?;
    // result?;
    // Ok(())

    draw_chars()?;
    Ok(())
}
