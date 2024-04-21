#[allow(unused_imports)]
use pdb_tui::gpu::run_windowed::run;

fn main() {
    pollster::block_on(run());
}
