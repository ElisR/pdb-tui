use pdb_tui::gpu::mine::rendering::run;

fn main() {
    pollster::block_on(run());
}
