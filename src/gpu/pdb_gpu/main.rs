use pdb_tui::gpu::pdb_gpu::run;

fn main() {
    pollster::block_on(run());
}
