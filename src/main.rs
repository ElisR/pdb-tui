#![allow(dead_code)]
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};
use pdb_tui::{
    rasterizer::BasicAsciiRasterizer,
    render::{Canvas, Scene},
};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    text::Text,
    widgets::Paragraph,
};
use std::io::{stdout, Result};

/// Perform shutdown of terminal
fn shutdown() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn run() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Load and draw
    let test_obj = "./data/surface.obj";
    let mut scene = Scene::default();
    scene.load_meshes_from_path(test_obj);
    let mut canvas = Canvas::<BasicAsciiRasterizer>::default();

    canvas.draw_scene_to_canvas(&scene);

    // TODO Make all of this async
    loop {
        // TODO Update frame size dynamically
        // TODO Only draw to canvas if something about the scene has changed
        canvas.draw_scene_to_canvas(&scene);
        let out_string: String = canvas.frame_buffer.iter().collect();
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(Paragraph::new(Text::raw(&out_string)), area);
        })?;

        // TODO Move this out into separate function
        // Listen for keypress
        if event::poll(std::time::Duration::from_millis(3))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // TODO Match on capital letters to rotate
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('l') => {
                            let transform = Isometry3::translation(5f32, 0f32, 0f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('h') => {
                            let transform = Isometry3::translation(-5f32, 0f32, 0f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('k') => {
                            let transform = Isometry3::translation(0f32, 5f32, 0f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('j') => {
                            let transform = Isometry3::translation(0f32, -5f32, 0f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('u') => {
                            let transform = Isometry3::translation(0f32, 0f32, 5f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('d') => {
                            let transform = Isometry3::translation(0f32, 0f32, -5f32);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('L') => {
                            let rotation = UnitQuaternion::from_scaled_axis(
                                Vector3::y() * std::f32::consts::FRAC_PI_8,
                            );
                            let transform =
                                Isometry3::from_parts(Translation3::identity(), rotation);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('H') => {
                            let rotation = UnitQuaternion::from_scaled_axis(
                                -Vector3::y() * std::f32::consts::FRAC_PI_8,
                            );
                            let transform =
                                Isometry3::from_parts(Translation3::identity(), rotation);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('K') => {
                            let rotation = UnitQuaternion::from_scaled_axis(
                                Vector3::x() * std::f32::consts::FRAC_PI_8,
                            );
                            let transform =
                                Isometry3::from_parts(Translation3::identity(), rotation);
                            scene.transform_meshes(&transform);
                        }
                        KeyCode::Char('J') => {
                            let rotation = UnitQuaternion::from_scaled_axis(
                                -Vector3::x() * std::f32::consts::FRAC_PI_8,
                            );
                            let transform =
                                Isometry3::from_parts(Translation3::identity(), rotation);
                            scene.transform_meshes(&transform);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    startup()?;
    let result = run();
    shutdown()?;
    result?;
    Ok(())
}
