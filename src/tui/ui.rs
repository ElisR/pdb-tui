#![allow(dead_code)]
use crate::{
    rasterizer::{BasicAsciiRasterizer, Rasterizer},
    render::{Canvas, Scene},
};
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Frame, Stylize, Terminal},
    style::Color,
    text::Text,
    widgets::Paragraph,
};
use std::io::{stdout, Result};

/// Enum holding the possible things that will happen after an action
enum NextAction {
    Quit,
    Translate { x: f32, y: f32, z: f32 },
    Rotate { axis: Vector3<f32>, angle: f32 },
    Save,
    Nothing,
    Help,
}

/// Return the next action depending on the latest `KeyEvent`
fn next_action_from_key(key: KeyEvent) -> NextAction {
    let minor_rotation = std::f32::consts::FRAC_PI_8 / 2.0;
    let minor_translation = 5.0f32;
    if key.kind == KeyEventKind::Press {
        match key.code {
            KeyCode::Char('q') => NextAction::Quit,
            KeyCode::Char('l') | KeyCode::Right => NextAction::Translate {
                x: minor_translation,
                y: 0.0,
                z: 0.0,
            },
            KeyCode::Char('h') | KeyCode::Left => NextAction::Translate {
                x: -minor_translation,
                y: 0.0,
                z: 0.0,
            },
            KeyCode::Char('k') | KeyCode::Up => NextAction::Translate {
                x: 0.0,
                y: minor_translation,
                z: 0.0,
            },
            KeyCode::Char('j') | KeyCode::Down => NextAction::Translate {
                x: 0.0,
                y: -minor_translation,
                z: 0.0,
            },
            KeyCode::Char('u') => NextAction::Translate {
                x: 0.0,
                y: 0.0,
                z: minor_translation,
            },
            KeyCode::Char('d') => NextAction::Translate {
                x: 0.0,
                y: 0.0,
                z: -minor_translation,
            },
            KeyCode::Char('H') => NextAction::Rotate {
                axis: Vector3::y(),
                angle: -minor_rotation,
            },
            KeyCode::Char('L') => NextAction::Rotate {
                axis: Vector3::y(),
                angle: minor_rotation,
            },
            KeyCode::Char('K') => NextAction::Rotate {
                axis: Vector3::x(),
                angle: -minor_rotation,
            },
            KeyCode::Char('J') => NextAction::Rotate {
                axis: Vector3::x(),
                angle: minor_rotation,
            },
            KeyCode::Char('s') => NextAction::Save,
            KeyCode::Char('?') => NextAction::Help,
            _ => NextAction::Nothing,
        }
    } else {
        NextAction::Nothing
    }
}

fn update<R: Rasterizer>(app: &mut App<R, Rendering>, next_action: NextAction) {
    match next_action {
        NextAction::Rotate { axis, angle } => {
            let rotation = UnitQuaternion::from_scaled_axis(axis * angle);
            let transform = Isometry3::from_parts(Translation3::identity(), rotation);
            app.scene.transform_meshes(&transform);
            app.canvas.draw_scene_to_canvas(&app.scene);
        }
        NextAction::Translate { x, y, z } => {
            let transform = Isometry3::translation(x, y, z);
            // scene.transform_meshes(&transform);
            app.scene.transform_view(&transform);
            app.canvas.draw_scene_to_canvas(&app.scene);
        }
        NextAction::Save => {
            let now: DateTime<Local> = Local::now();
            let path = format!("canvas_screenshot_{}.png", now.format("%Y%m%d_%H%M%S"));
            // TODO Bubble this up to an error popup if something goes wrong
            let _ = app.canvas.save_image(path);
        }
        NextAction::Quit => {
            app.should_quit = true;
        }
        NextAction::Help => {
            // TODO Actually do something when help key is pressed
        }
        NextAction::Nothing => {}
    };
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
    if (area.width as usize != canvas.render_width())
        || (area.height as usize != canvas.render_height())
    {
        canvas.resize(area.width as usize, area.height as usize);
        scene.update_aspect(area.width as usize, area.height as usize);
        canvas.draw_scene_to_canvas(scene);
    }
    let out_string: String = canvas.frame_buffer.iter().collect();
    frame.render_widget(
        Paragraph::new(Text::raw(&out_string)).fg(Color::Magenta),
        area,
    );
}

// TODO Eventually derive debug
#[derive(Default)]
pub struct App<R: Rasterizer, V: ValidState> {
    pub should_quit: bool,

    pub scene: Scene,
    pub canvas: Canvas<R>,
    pub state: V,
}

/// Trait used for managing valid state of UI
pub trait ValidState {}

#[derive(Default)]
pub struct Rendering {}
impl ValidState for Rendering {}

pub fn run() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Load and draw
    let test_obj = "./data/surface.obj";

    let mut app = App::<BasicAsciiRasterizer, Rendering>::default();
    app.scene.load_meshes_from_path(test_obj);
    app.scene.meshes_to_center();
    app.canvas.draw_scene_to_canvas(&app.scene);

    // TODO Make all of this async
    loop {
        terminal.draw(|frame| ui(&mut app.canvas, &mut app.scene, frame))?;

        if event::poll(std::time::Duration::from_millis(3))? {
            if let event::Event::Key(key) = event::read()? {
                let next_action = next_action_from_key(key);
                update(&mut app, next_action);
                if app.should_quit {
                    break;
                }
            }
        }
    }
    Ok(())
}
