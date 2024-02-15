#![allow(dead_code)]
use nalgebra::{Isometry3, Translation3, UnitDualQuaternion, UnitQuaternion, Vector3};
use pdb_tui::{
    rasterizer::BasicAsciiRasterizer,
    render::{Canvas, Scene},
};
use std::time::Instant;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    text::Text,
    widgets::Paragraph,
};
use std::io::{stdout, Result};

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // TODO main loop

    // Load and draw
    let test_obj = "./data/surface.obj";
    let mut scene = Scene::default();
    scene.load_meshes_from_path(test_obj);
    // let translation = Translation3::new(5.0f32, 0.0f32, 0.0f32);
    // let rotation = UnitQuaternion::from_scaled_axis(Vector3::y() * std::f32::consts::FRAC_PI_8);
    // let rotation = UnitQuaternion::identity();
    // let transform = Isometry3::from_parts(translation, rotation);
    let mut canvas = Canvas::<BasicAsciiRasterizer>::default();

    canvas.draw_scene_to_canvas(&scene);

    // Print output to stdout
    // print!("{}", out_string);

    loop {
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
                        _ => {}
                    }
                }
            }
        }
    }

    // let x_shift = Isometry3::from_parts(
    //     Translation3::new(5.0f32, 0f32, 0f32),
    //     UnitQuaternion::identity(),
    // );
    // for i in 0..10 {
    //     scene.transform_meshes(&x_shift);
    //     canvas.draw_scene_to_canvas(&scene);
    //     let path = format!("canvas_{}.png", i);
    //     canvas.save_image(path).unwrap();

    // Cleanup
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
