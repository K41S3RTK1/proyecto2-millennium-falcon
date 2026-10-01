//! Compara el coste completo GPU + presentación + lectura sincronizada a igual salida.
use falcon_diorama::{
    camera::Camera, combat::Combat, display::Display, flight::Flight, gpu::Renderer, png,
    render::Settings, scene::Scene,
};
use raylib::prelude::*;
use std::{error::Error, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    let (mut window, thread) = raylib::init()
        .size(640, 400)
        .hidden()
        .title("Comparar nitidez")
        .build();
    let mut filter = Display::new(&mut window, &thread);
    if !filter.available() {
        return Err("No compila el shader de suavizado".into());
    }
    let mut canvas = window.load_render_texture(&thread, 1600, 1000)?;
    for case in 0..3 {
        let flight = Flight {
            active: case > 0,
            progress: 1.,
            boost_age: if case == 2 { 1. } else { -1. },
            ..Default::default()
        };
        let combat = Combat {
            enabled: case > 0,
            ..Default::default()
        };
        let scene = if case == 0 {
            Scene::new()
        } else {
            Scene::flight_combat(7)
        };
        let mut renderer = Renderer::new(&mut window, &thread, &scene)?;
        for (width, aa) in [(1200, false), (1200, true), (1600, true), (1200, false)] {
            filter.enabled = aa;
            let mut times = Vec::new();
            for i in 0..44 {
                let mut camera = if case == 0 {
                    Camera::default()
                } else {
                    combat.camera(flight)
                };
                camera.yaw += i as f32 * 0.25;
                let started = Instant::now();
                renderer.render(
                    &mut window,
                    &thread,
                    camera,
                    Settings {
                        width,
                        height: width * 5 / 8,
                        quality: 0,
                        space: case > 0,
                        flight,
                        combat,
                        ..Default::default()
                    },
                )?;
                {
                    let mut draw = window.begin_texture_mode(&thread, &mut canvas);
                    draw.clear_background(Color::BLACK);
                    filter.draw(
                        &mut draw,
                        renderer.texture().unwrap(),
                        Rectangle::new(0., 0., 1600., 1000.),
                    );
                }
                let image = canvas.texture().load_image()?; // Sincroniza: incluye raytracing, filtro y transferencia GPU.
                let elapsed = started.elapsed().as_secs_f64() * 1000.;
                if i >= 12 {
                    times.push(elapsed);
                }
                if i == 0 {
                    let data = image.get_image_data();
                    let bytes: Vec<u8> = data
                        .chunks_exact(1600)
                        .rev()
                        .flat_map(|row| row.iter().flat_map(|p| [p.r, p.g, p.b, p.a]))
                        .collect();
                    png::save(
                        format!("/tmp/calidad-{case}-{width}-{aa}.png"),
                        1600,
                        1000,
                        &bytes,
                    )?;
                }
            }
            times.sort_by(f64::total_cmp);
            println!(
                "MEDICION caso={case} ancho={width} suavizado={aa} mediana={:.2}ms peor={:.2}ms",
                times[times.len() / 2],
                times[times.len() - 1]
            );
        }
    }
    Ok(())
}
