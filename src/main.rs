#![allow(dead_code)]
use clap::Parser;
use pdb_tui::tui::ui::{run, shutdown, startup};
use std::io::Result;

/// Program to render PDBs within a terminal user interface
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDB file to be loaded
    #[arg(short, long, default_value_t = String::from("./data/rbd.pdb"))]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    startup()?;
    let result = run(args.input);
    shutdown()?;
    result?;
    Ok(())
}
