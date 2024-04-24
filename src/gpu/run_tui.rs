use tracing::error;
// use tracing::warn;
use tracing::Level;
use tracing_subscriber;
use winit::dpi::PhysicalSize;

use crate::gpu::input::{UnifiedEvent, UnifiedKeyCode};
use crate::gpu::state_windowless::WindowlessState;
use crate::gpu::{InnerState, State};

use crate::basic_rasterizer::BasicAsciiRasterizer;
use crate::rasterizer::ColoredChar;
use crate::trivial_rasterizer::chars_to_widget;

use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stdout, Result};

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
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // let rasterizer = BasicAsciiRasterizer::default();

    let width = terminal.size()?.width as u32;
    let height = terminal.size()?.height as u32;
    let mut state = State::<WindowlessState>::new(
        PhysicalSize { width, height },
        PhysicalSize {
            width: 1,
            height: 1,
        },
    )
    .await;
    state.camera_controller.speed *= 3.0;
    // state.camera_controller.speed /= 10.0;

    // Render the first frame to avoid blank screen upon loading
    if (state.render().await).is_err() {
        error!("Something went wrong with rendering.")
    }

    loop {
        terminal.draw(|frame| {
            // TODO Fix the problems arising with this resize. Maybe because of await?
            let frame_width = frame.size().width as u32;
            let frame_height = frame.size().height as u32;
            if frame_width != state.inner_state.output_size().width
                || frame_height != state.inner_state.output_size().height
            {
                state.resize(PhysicalSize {
                    width: frame_width,
                    height: frame_height,
                });
            }

            let colored_chars: Vec<_> = state
                .inner_state
                .output_image
                .chunks(4usize)
                .map(|c| c[3])
                .map(ColoredChar::from)
                .collect();
            let widget = chars_to_widget(
                colored_chars,
                state.inner_state.output_size().width as usize,
            );

            frame.render_widget(widget, frame.size());
        })?;

        let tui_event = event::read()?;
        let unified_event: UnifiedEvent = (&tui_event).into();
        if unified_event.keycode == UnifiedKeyCode::Esc {
            break;
        }

        // TODO Add logic to compare current size of frame

        state.input(unified_event);
        state.update();
        match state.render().await {
            Ok(_) => {}
            Err(_) => {
                error!("Something went wrong with rendering.")
            }
        }
        state.camera_controller.reset_velocity();
    }
    Ok(())
}
