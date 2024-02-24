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
pub enum NextAction {
    Quit,
    Translate { x: f32, y: f32, z: f32 },
    Rotate { axis: Vector3<f32>, angle: f32 },
    Save,
    Nothing,
    Help,
    Back,
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
            KeyCode::Esc => NextAction::Back,
            _ => NextAction::Nothing,
        }
    } else {
        NextAction::Nothing
    }
}

#[derive(Default)]
pub struct App<S: StateMarker> {
    should_quit: bool,

    state: std::marker::PhantomData<S>,
}

pub trait StateMarker {}

pub enum StateWrapper {
    Rendering(App<RenderState>),
    Helping(App<HelpState>),
}

/// Marker trait used for managing valid state of UI
impl StateWrapper {
    pub fn update<R: Rasterizer>(
        mut self,
        canvas: &mut Canvas<R>,
        scene: &mut Scene,
        next_action: NextAction,
    ) -> Self {
        match self {
            Self::Rendering(ref mut app) => {
                match next_action {
                    NextAction::Rotate { axis, angle } => {
                        let rotation = UnitQuaternion::from_scaled_axis(axis * angle);
                        let transform = Isometry3::from_parts(Translation3::identity(), rotation);
                        scene.transform_meshes(&transform);
                        canvas.draw_scene_to_canvas(scene);
                    }
                    NextAction::Translate { x, y, z } => {
                        let transform = Isometry3::translation(x, y, z);
                        // scene.transform_meshes(&transform);
                        scene.transform_view(&transform);
                        canvas.draw_scene_to_canvas(scene);
                    }
                    NextAction::Save => {
                        let now: DateTime<Local> = Local::now();
                        let path = format!("canvas_screenshot_{}.png", now.format("%Y%m%d_%H%M%S"));
                        // TODO Bubble this up to an error popup if something goes wrong
                        let _ = canvas.save_image(path);
                    }
                    NextAction::Quit => {
                        app.should_quit = true;
                    }
                    NextAction::Help => {
                        // TODO Actually do something when help key is pressed
                    }
                    _ => {}
                }
                self
            }
            Self::Helping(ref mut app) => {
                match next_action {
                    NextAction::Quit => {
                        app.should_quit = true;
                    }
                    NextAction::Back => {
                        // TODO Move back to rendering state
                    }
                    _ => {}
                }
                self
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        match self {
            Self::Rendering(app) => app.should_quit,
            Self::Helping(app) => app.should_quit,
        }
    }
}

#[derive(Default)]
pub struct RenderState;

#[derive(Default)]
pub struct HelpState;

impl StateMarker for HelpState {}
impl StateMarker for RenderState {}

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

pub fn run() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Load and draw
    let test_obj = "./data/surface.obj";

    let mut app = StateWrapper::Rendering(App::<RenderState>::default());
    let mut canvas = Canvas::<BasicAsciiRasterizer>::default();
    let mut scene = Scene::default();
    scene.load_meshes_from_path(test_obj);
    scene.meshes_to_center();
    canvas.draw_scene_to_canvas(&scene);

    // TODO Make all of this async
    loop {
        terminal.draw(|frame| ui(&mut canvas, &mut scene, frame))?;

        if event::poll(std::time::Duration::from_millis(3))? {
            if let event::Event::Key(key) = event::read()? {
                let next_action = next_action_from_key(key);
                app = app.update(&mut canvas, &mut scene, next_action);
                if app.should_quit() {
                    break;
                }
            }
        }
    }
    Ok(())
}
