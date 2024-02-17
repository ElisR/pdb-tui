#![allow(dead_code)]
use crate::{
    rasterizer::{BasicAsciiRasterizer, Rasterizer},
    render::{Canvas, Scene},
};
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Frame, Terminal},
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

/// Return the next action depending on the latest `KeyEvent`
fn next_action_from_key(key: KeyEvent) -> NextAction {
    if key.kind == KeyEventKind::Press {
        match key.code {
            KeyCode::Char('q') => NextAction::Quit,
            KeyCode::Char('l') | KeyCode::Right => NextAction::Translate {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
            KeyCode::Char('h') | KeyCode::Left => NextAction::Translate {
                x: -5.0,
                y: 0.0,
                z: 0.0,
            },
            KeyCode::Char('k') | KeyCode::Up => NextAction::Translate {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            KeyCode::Char('j') | KeyCode::Down => NextAction::Translate {
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
        }
    } else {
        NextAction::Nothing
    }
}

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

fn ui<R: Rasterizer>(canvas: &mut Canvas<R>, scene: &mut Scene, frame: &mut Frame) {
    let area = frame.size();
    if (area.width as usize != canvas.width()) || (area.height as usize != canvas.height()) {
        canvas.resize(area.width as usize, area.height as usize);
        scene.update_aspect(area.width as usize, area.height as usize);
        canvas.draw_scene_to_canvas(scene);
    }
    let out_string: String = canvas.frame_buffer.iter().collect();
    frame.render_widget(Paragraph::new(Text::raw(&out_string)), area);
}

pub fn run() -> Result<()> {
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
        terminal.draw(|frame| ui(&mut canvas, &mut scene, frame))?;

        if event::poll(std::time::Duration::from_millis(3))? {
            if let event::Event::Key(key) = event::read()? {
                let next_action = next_action_from_key(key);
                match next_action {
                    NextAction::Rotate { axis, angle } => {
                        let rotation = UnitQuaternion::from_scaled_axis(axis * angle);
                        let transform = Isometry3::from_parts(Translation3::identity(), rotation);
                        scene.transform_meshes(&transform);
                        canvas.draw_scene_to_canvas(&scene);
                    }
                    NextAction::Translate { x, y, z } => {
                        let transform = Isometry3::translation(x, y, z);
                        scene.transform_meshes(&transform);
                        canvas.draw_scene_to_canvas(&scene);
                    }
                    NextAction::Quit => {
                        break;
                    }
                    NextAction::Nothing => {}
                };
            }
        }
    }
    Ok(())
}
