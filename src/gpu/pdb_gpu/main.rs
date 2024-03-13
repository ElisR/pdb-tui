#[allow(unused_imports)]
use pdb_tui::gpu::pdb_gpu::run::run_windowed;

fn main() {
    pollster::block_on(run_windowed());
}
