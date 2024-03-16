use tracing::error;
use tracing::Level;
use tracing_subscriber;
use winit::dpi::PhysicalSize;

use crate::gpu::pdb_gpu::{State, WindowlessState};

pub async fn run() {
    tracing_subscriber::fmt().with_max_level(Level::WARN).init();

    // TODO Need to find a way to avoid having to do mulitples of `COPY_BYTES_PER_ROW_ALIGNMENT`
    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::<WindowlessState>::new(PhysicalSize {
        width: 2048,
        height: 2048,
    })
    .await;

    match state.render().await {
        Ok(_) => {}
        // Reconfigure the surface if it's lost or outdated
        Err(_) => {
            error!("Something went wrong with rendering.")
        }
    };
}
