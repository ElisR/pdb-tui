use pdb_tui::gpu::pdb_gpu::run::run;

fn main() {
    pollster::block_on(run());
}
