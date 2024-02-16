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

/// Enum holding the possible things that will happen after an action
enum NextAction {
    Quit,
    Translate { x: f32, y: f32, z: f32 },
    Rotate { axis: Vector3<f32>, angle: f32 },
    Nothing,
}

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

        // Listen for keypress
        if event::poll(std::time::Duration::from_millis(3))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let next_action = match key.code {
                        KeyCode::Char('q') => NextAction::Quit,
                        KeyCode::Char('l') => NextAction::Translate {
                            x: 5.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        KeyCode::Char('h') => NextAction::Translate {
                            x: 5.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        KeyCode::Char('k') => NextAction::Translate {
                            x: 0.0,
                            y: 5.0,
                            z: 0.0,
                        },
                        KeyCode::Char('j') => NextAction::Translate {
                            x: 0.0,
                            y: -5.0,
                            z: 0.0,
                        },
                        KeyCode::Char('u') => NextAction::Translate {
                            x: 0.0,
                            y: 0.0,
                            z: 5.0,
                        },
                        KeyCode::Char('d') => NextAction::Translate {
                            x: 0.0,
                            y: 0.0,
                            z: -5.0,
                        },
                        KeyCode::Char('L') => NextAction::Rotate {
                            axis: Vector3::y(),
                            angle: std::f32::consts::FRAC_PI_8,
                        },
                        KeyCode::Char('H') => NextAction::Rotate {
                            axis: Vector3::y(),
                            angle: -std::f32::consts::FRAC_PI_8,
                        },
                        KeyCode::Char('K') => NextAction::Rotate {
                            axis: Vector3::x(),
                            angle: -std::f32::consts::FRAC_PI_8,
                        },
                        KeyCode::Char('J') => NextAction::Rotate {
                            axis: Vector3::x(),
                            angle: -std::f32::consts::FRAC_PI_8,
                        },
                        _ => NextAction::Nothing,
                    };

                    match next_action {
                        NextAction::Quit => {
                            break;
                        }
                        NextAction::Rotate { axis, angle } => {
                            let rotation = UnitQuaternion::from_scaled_axis(axis * angle);
                            let transform =
                                Isometry3::from_parts(Translation3::identity(), rotation);
                            scene.transform_meshes(&transform);
                        }
                        NextAction::Translate { x, y, z } => {
                            let transform = Isometry3::translation(x, y, z);
                            scene.transform_meshes(&transform);
                        }
                        NextAction::Nothing => {}
                    };
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
