use tracing::error;
use tracing::Level;
use tracing_subscriber;
use winit::dpi::PhysicalSize;

use crate::gpu::pdb_gpu::input::{UnifiedEvent, UnifiedKeyCode};
use crate::gpu::pdb_gpu::{State, WindowlessState};

use crate::basic_rasterizer::BasicAsciiRasterizer;
use crate::rasterizer::ColoredPixel;
use crate::rasterizer::Rasterizer;
use crate::tui::ui::widget_from_frame_buffer;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stdout, Result};

pub async fn run() {
    tracing_subscriber::fmt().with_max_level(Level::WARN).init();

    // TODO Need to find a way to avoid having to do mulitples of `COPY_BYTES_PER_ROW_ALIGNMENT`
    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::<WindowlessState>::new(PhysicalSize {
        width: 256,
        height: 256,
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

// TODO Import colored char

/// Perform shutdown of terminal
pub fn shutdown() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Start the terminal
pub fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

pub async fn run_new() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let rasterizer = BasicAsciiRasterizer::default();

    let size = 256u32;
    let mut state = State::<WindowlessState>::new(PhysicalSize {
        width: size,
        height: size,
    })
    .await;

    // TODO Make all of this async
    loop {
        terminal.draw(|frame| {
            let pixels: Vec<_> = state
                .inner_state
                .output_image
                .chunks(4usize)
                .map(|c| c[3])
                .map(ColoredPixel::from)
                .collect();
            let pixel_chunks: Vec<&[ColoredPixel]> = pixels.chunks(1usize).collect();
            let chars = rasterizer.pixels_to_stdout(pixel_chunks, size as usize);

            let widget = widget_from_frame_buffer(&chars);
            frame.render_widget(widget, frame.size());
        })?;

        if event::poll(std::time::Duration::from_millis(1))? {
            if let tui_event = event::read()? {
                let unified_event: UnifiedEvent = (&tui_event).into();
                if unified_event.keycode == UnifiedKeyCode::Esc {
                    break;
                }

                state.input(&tui_event);

                state.update();
                match state.render().await {
                    Ok(_) => {}
                    Err(_) => {
                        error!("Something went wrong with rendering.")
                    }
                }
            }
        }
    }
    Ok(())
}
