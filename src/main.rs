#![allow(dead_code)]
use pdb_tui::tui::ui::{run, shutdown, startup};
use std::io::Result;

fn main() -> Result<()> {
    startup()?;
    let result = run();
    shutdown()?;
    result?;
    Ok(())
}
