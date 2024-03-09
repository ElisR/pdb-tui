#![allow(dead_code)]
use clap::Parser;
use pdb_tui::tui::ui::{run, shutdown, startup};
use std::io::Result;

/// Program to render PDBs within a terminal user interface
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDB file to be loaded
    #[arg(short, long, num_args=1.., default_value = "./data/surface.obj")]
    inputs: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    startup()?;
    let result = run(args.inputs);
    shutdown()?;
    result?;
    Ok(())
}
