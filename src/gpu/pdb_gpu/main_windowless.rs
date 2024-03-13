#[allow(unused_imports)]
use pdb_tui::gpu::pdb_gpu::run::run_windowless;

fn main() {
    pollster::block_on(run_windowless());
}
