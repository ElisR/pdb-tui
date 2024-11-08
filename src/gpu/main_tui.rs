#[allow(unused_imports)]
use pdb_tui::gpu::run_tui::{run_new, shutdown, startup};
use std::io::Result;

// fn main() {
//     pollster::block_on(run());
// }

fn main() -> Result<()> {
    startup()?;
    let result = pollster::block_on(run_new());
    shutdown()?;
    result?;
    Ok(())
}
