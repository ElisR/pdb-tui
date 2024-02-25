#![allow(dead_code)]
use crate::{
    rasterizer::{BasicAsciiRasterizer, Rasterizer},
    render::{Canvas, Scene},
    tui::{
        popup::Popup,
        state::{App, BenchmarkState, HelpState, RenderState},
    },
};
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};

use chrono::{DateTime, Local};
use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
// TODO Consider just importing everything from `prelude` and `widgets`
use ratatui::{
    prelude::{CrosstermBackend, Frame, Rect, Style, Stylize, Terminal},
    style::Color,
    text::{Line, Text},
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


pub enum StateWrapper {
    Rendering(App<RenderState>),
    Helping(App<HelpState>),
}

// Unhappy with how this requires matching every state arm
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
                        self
                    }
                    NextAction::Translate { x, y, z } => {
                        let transform = Isometry3::translation(x, y, z);
                        // scene.transform_meshes(&transform);
                        scene.transform_view(&transform);
                        canvas.draw_scene_to_canvas(scene);
                        self
                    }
                    NextAction::Save => {
                        let now: DateTime<Local> = Local::now();
                        let path = format!("canvas_screenshot_{}.png", now.format("%Y%m%d_%H%M%S"));
                        // TODO Bubble this up to an error popup if something goes wrong
                        let _ = canvas.save_image(path);
                        self
                    }
                    NextAction::Quit => {
                        app.should_quit = true;
                        self
                    }
                    NextAction::Help => {
                        // TODO Actually do something when help key is pressed
                        StateWrapper::Helping(App::<HelpState>::from(*app))
                    }
                    _ => self,
                }
            }
            Self::Helping(ref mut app) => {
                match next_action {
                    NextAction::Quit => {
                        app.should_quit = true;
                        self
                    }
                    NextAction::Back => {
                        // TODO Move back to rendering state
                        StateWrapper::Rendering(App::<RenderState>::from(*app))
                    }
                    _ => self,
                }
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        match self {
            Self::Rendering(app) => app.should_quit,
            Self::Helping(app) => app.should_quit,
        }
    }

    pub fn ui<R: Rasterizer>(&self, canvas: &mut Canvas<R>, scene: &mut Scene, frame: &mut Frame) {
        // TODO Once line colour issue is fixed, change this back to be the whole screen
        let area = frame.size();
        let render_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height - 1,
        };
        if (render_area.width as usize != canvas.render_width())
            || (render_area.height as usize != canvas.render_height())
        {
            canvas.resize(render_area.width as usize, render_area.height as usize);
            scene.update_aspect(render_area.width as usize, render_area.height as usize);
            canvas.draw_scene_to_canvas(scene);
        }
        let out_string: String = canvas.frame_buffer.iter().collect();
        let widget = Paragraph::new(Text::raw(&out_string)).fg(Color::Blue);
        frame.render_widget(widget, render_area);

        match self {
            Self::Helping(_) => {
                let popup_area = Rect {
                    x: area.width / 3,
                    y: area.height / 4,
                    width: area.width / 3,
                    height: area.height / 2,
                };

                // TODO Move this to constant in another module
                let help_text = vec![
                    Line::from("q:      Quit the application."),
                    Line::from("<Esc>:  Back"),
                    Line::from(""),
                    Line::from("d:      Zoom out."),
                    Line::from("u:      Zoom in."),
                    Line::from(""),
                    Line::from("h:      Move left."),
                    Line::from("l:      Move right."),
                    Line::from("k:      Move up."),
                    Line::from("j:      Move down."),
                    Line::from(""),
                    Line::from("H:      Rotate left."),
                    Line::from("L:      Rotate right."),
                    Line::from("K:      Rotate up."),
                    Line::from("J:      Rotate down."),
                ];

                let popup = HelpPopup::default()
                    .content(help_text)
                    .style(Style::new().black())
                    .title("Help")
                    .title_style(Style::new().bold())
                    .border_style(Style::new().red());
                frame.render_widget(popup, popup_area);
            }
            Self::Rendering(_) => {
                let bottom = Rect {
                    x: 0,
                    y: area.height - 1,
                    width: area.width,
                    height: 1,
                }
                .clamp(area);
                // TODO Work out how to avoid whole line being coloured the same
                let text = Text::raw("Press ? for help.")
                    .style(Style::new().red())
                    .alignment(ratatui::layout::Alignment::Right);
                frame.render_widget(text, bottom);
            }
        }
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
        terminal.draw(|frame| app.ui(&mut canvas, &mut scene, frame))?;

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
