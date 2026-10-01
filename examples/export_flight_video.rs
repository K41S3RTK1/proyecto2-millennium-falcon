//! Exportador de producción; requiere FFmpeg en PATH, no se usa al ejecutar el diorama.
//! cargo run --release --offline --example export_flight_video -- /tmp/vuelo-silent.mp4
use falcon_diorama::{
    combat::Combat,
    flight::{Flight, TAKEOFF_SECONDS},
    gpu::Renderer,
    png,
    render::Settings,
    scene::Scene,
};
use raylib::prelude::*;
use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};
const W: usize = 1200;
const H: usize = 800;
const FPS: usize = 30;
const SECONDS: usize = 80;
const SHOTS: [f32; 9] = [55., 57., 59., 62., 64., 66., 69., 71., 73.];
fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/vuelo-silent.mp4".into());
    let (mut window, thread) = raylib::init()
        .size(640, 400)
        .hidden()
        .title("Exportacion modo nave")
        .build();
    let mut renderer = Renderer::new(&mut window, &thread, &Scene::flight())?;
    let mut mask = 0;
    let mut combat = Combat::default();
    let mut shot = 0;
    let mut canvas = window.load_render_texture(&thread, W as u32, H as u32)?;
    let mut encoder = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "warning",
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-video_size",
            "1200x800",
            "-framerate",
            "30",
            "-i",
            "pipe:0",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "21",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            &output,
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    let mut pipe = encoder.stdin.take().ok_or("No se abrió FFmpeg")?;
    for frame in 0..SECONDS * FPS {
        let time = frame as f32 / FPS as f32;
        combat.enabled = time >= TAKEOFF_SECONDS;
        combat.update(1. / FPS as f32);
        if shot < SHOTS.len() && time >= SHOTS[shot] {
            if !combat.fire() {
                return Err("Disparo del video rechazado".into());
            }
            shot += 1;
        }
        if mask != combat.mask() {
            mask = combat.mask();
            renderer = Renderer::new(&mut window, &thread, &Scene::flight_combat(mask))?;
        }
        let flight = Flight {
            active: true,
            progress: (time / TAKEOFF_SECONDS).min(1.),
            boost_age: if time >= 54. { time - 54. } else { -1. },
            ..Default::default()
        };
        let mut camera = combat.camera(flight);
        // Mostrar ambas direcciones de giro con la misma convención que el teclado.
        let label = if time < 12. {
            camera.yaw -= time * 1.5;
            "9 MODO NAVE | ASCENSO | FLECHA DERECHA: GIRO A LA DERECHA".to_string()
        } else if time < 24. {
            camera.yaw += -18. + (time - 12.) * 1.5;
            "ASCENSO | FLECHA IZQUIERDA: GIRO A LA IZQUIERDA".to_string()
        } else if !combat.enabled {
            format!(
                "DESPEGUE | {:.0}% | ESPERANDO LLEGADA AL ESPACIO",
                flight.progress * 100.
            )
        } else if combat.mask() == 0 {
            "VICTORIA | TRES TIE DESTRUIDOS | N: NUEVA OLEADA".to_string()
        } else {
            format!(
                "ESPACIO | E: IMPULSO | F: DISPARAR | VIDA TIE: {}/3   {}/3   {}/3",
                combat.hp[0], combat.hp[1], combat.hp[2]
            )
        };
        renderer.render(
            &mut window,
            &thread,
            camera,
            Settings {
                width: W,
                height: H,
                quality: 1,
                space: true,
                flight,
                combat,
                ..Default::default()
            },
        )?;
        {
            let mut draw = window.begin_texture_mode(&thread, &mut canvas);
            draw.clear_background(Color::BLACK);
            draw.draw_texture_pro(
                renderer.texture().unwrap(),
                Rectangle::new(0., 0., W as f32, -(H as f32)),
                Rectangle::new(0., 0., W as f32, H as f32),
                Vector2::zero(),
                0.,
                Color::WHITE,
            );
            draw.draw_rectangle(0, H as i32 - 64, W as i32, 64, Color::new(10, 17, 24, 235));
            draw.draw_text(&label, 24, H as i32 - 53, 20, Color::new(255, 210, 80, 255));
            draw.draw_text("Proyecto 2 | Raytracing Rust + GLSL | Recorrido renderizado con audio sincronizado",24,H as i32-25,15,Color::new(210,222,234,255));
        }
        let image = canvas.texture().load_image()?;
        let data = image.get_image_data();
        let pixels: Vec<u8> = data
            .chunks_exact(W)
            .rev()
            .flat_map(|row| row.iter().flat_map(|p| [p.r, p.g, p.b, p.a]))
            .collect();
        if [0, 600, 1496, 1658, 1790, 2240, 2350].contains(&frame) {
            png::save(format!("/tmp/vuelo-video-{frame}.png"), W, H, &pixels)?;
        }
        pipe.write_all(&pixels)?;
        if frame % (FPS * 10) == 0 {
            eprintln!("VIDEO {frame}/{}", SECONDS * FPS);
        }
    }
    drop(pipe);
    if !encoder.wait()?.success() {
        return Err("FFmpeg no pudo codificar el video".into());
    }
    eprintln!("VIDEO terminado: {SECONDS}s, {FPS}fps");
    Ok(())
}
