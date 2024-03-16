#[allow(unused_imports)]
use pdb_tui::gpu::pdb_gpu::run_tui::run;

fn main() {
    pollster::block_on(run());
}
