//! Visor modal: al salir se destruye la escena de vuelo y se conserva el diorama.
use crate::{
    audio::{Audio, Effect, Event, Track},
    flight::Flight,
    gpu::Renderer,
    render::{self, Settings},
    scene::Scene,
};
use raylib::prelude::*;
use std::{error::Error, time::Instant};

pub fn run(
    window: &mut RaylibHandle,
    thread: &RaylibThread,
    audio: &Audio,
    prefer_gpu: bool,
) -> Result<usize, Box<dyn Error>> {
    let mut display = crate::display::Display::new(window, thread);
    let mut scene = Scene::flight();
    let mut combat = crate::combat::Combat::default();
    let mut scene_mask = 0;
    let mut combat_clock = Instant::now();
    let mut gpu = Renderer::new(window, thread, &scene).ok();
    let mut use_gpu = prefer_gpu && gpu.is_some();
    let mut cpu_texture: Option<Texture2D> = None;
    let mut yaw = 0.;
    let mut pitch = 0.;
    let mut zoom = 1.;
    let mut quality_width = 1200;
    let mut waiting = true;
    let mut message = String::new();
    let mut message_until = Instant::now();
    audio.event(Event::View(8));
    use KeyboardKey::*;
    while !window.window_should_close() {
        let mut typed = Vec::new();
        while let Some(c) = window.get_char_pressed() {
            typed.push(c);
        }
        let pressed = |key, c: char| {
            window.is_key_pressed(key) || typed.iter().any(|v| v.to_ascii_lowercase() == c)
        };

        for (i, key) in [
            KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN, KEY_EIGHT,
        ]
        .iter()
        .enumerate()
        {
            if pressed(*key, char::from(b'1' + i as u8)) {
                return Ok(i);
            }
        }
        if pressed(KEY_NINE, '9') {
            combat = crate::combat::Combat::default();
            audio.event(Event::View(8));
            waiting = true;
        }
        if pressed(KEY_M, 'm') {
            audio.event(Event::ToggleMute);
        }
        if pressed(KEY_J, 'j') {
            display.enabled = !display.enabled;
        }
        if pressed(KEY_Q, 'q') {
            quality_width = match quality_width {
                1200 => 1600,
                1600 => 800,
                _ => 1200,
            };
        }
        if pressed(KEY_T, 't') && gpu.is_some() {
            use_gpu = !use_gpu;
        }
        if pressed(KEY_R, 'r') {
            yaw = 0.;
            pitch = 0.;
            zoom = 1.;
        }
        let save = pressed(KEY_S, 's');
        let status = audio.status();
        if status.track == Some(Track::Takeoff) {
            waiting = false;
        }
        let progress = if waiting {
            0.
        } else if status.flight_ready {
            1.
        } else {
            (status.position / status.durations[Track::Takeoff as usize].max(0.1)).clamp(0., 1.)
        };
        let now = Instant::now();
        let elapsed = now.duration_since(combat_clock).as_secs_f32();
        combat_clock = now;
        combat.enabled = progress >= 1.;
        let previous_hp = combat.hp;
        combat.update(elapsed);
        for (&before, &after) in previous_hp.iter().zip(&combat.hp) {
            if before > 0 && after == 0 {
                audio.effect(Effect::Explosion);
            }
        }
        if pressed(KEY_N, 'n') && combat.enabled {
            audio.stop_effects();
            combat = crate::combat::Combat {
                enabled: true,
                ..Default::default()
            };
        }
        if pressed(KEY_F, 'f') {
            if combat.fire() {
                audio.effect(Effect::FalconShot);
                message.clear();
            } else {
                message = if !combat.enabled {
                    "Disparos disponibles al llegar al espacio"
                } else if combat.mask() == 0 {
                    "Escuadron destruido. N: nueva oleada"
                } else {
                    "Laser en vuelo..."
                }
                .into();
                message_until = now + std::time::Duration::from_secs(2);
            }
        }
        let flight = Flight {
            active: true,
            progress,
            boost_age: if !waiting && status.track == Some(Track::Boost) {
                status.position
            } else {
                -1.
            },
            boost_duration: status.durations[Track::Boost as usize],
        };
        let width = window.get_screen_width();
        let height = window.get_screen_height();
        let viewport = Rectangle::new(0., 100., width as f32, (height - 174) as f32);
        let mouse = window.get_mouse_position();
        let click = window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let back = Rectangle::new(width as f32 - 192., 18., 172., 32.);
        let boost = Rectangle::new(width as f32 - 225., 61., 205., 30.);
        if click && back.check_collision_point_rec(mouse) {
            return Ok(0);
        }
        if pressed(KEY_E, 'e') || (click && boost.check_collision_point_rec(mouse)) {
            if !waiting && status.flight_ready {
                audio.event(Event::Boost);
            } else {
                message = "Impulso disponible al llegar al espacio".into();
                message_until = Instant::now() + std::time::Duration::from_secs(3);
            }
        }
        if combat.mask() != scene_mask {
            scene_mask = combat.mask();
            scene = Scene::flight_combat(scene_mask);
            gpu = Renderer::new(window, thread, &scene).ok();
            use_gpu &= gpu.is_some();
        }
        let dt = window.get_frame_time().min(0.05);
        if viewport.check_collision_point_rec(mouse) {
            if window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                let delta = window.get_mouse_delta();
                yaw += delta.x * 0.25;
                pitch += delta.y * 0.25;
            }
            zoom *= (-window.get_mouse_wheel_move() * 0.08).exp();
        }
        yaw += (i32::from(window.is_key_down(KEY_LEFT)) - i32::from(window.is_key_down(KEY_RIGHT)))
            as f32
            * 40.
            * dt;
        pitch += (i32::from(window.is_key_down(KEY_UP)) - i32::from(window.is_key_down(KEY_DOWN)))
            as f32
            * 30.
            * dt;
        for &c in &typed {
            match c {
                '+' | '=' => zoom *= 0.9,
                '-' | '−' => zoom /= 0.9,
                _ => {}
            }
        }
        if window.is_key_pressed(KEY_KP_ADD) {
            zoom *= 0.9;
        }
        if window.is_key_pressed(KEY_KP_SUBTRACT) {
            zoom /= 0.9;
        }
        zoom = zoom.clamp(0.55, 2.0);
        pitch = pitch.clamp(-23., 62.);
        let mut camera = combat.camera(flight);
        camera.yaw += yaw;
        camera.pitch += pitch;
        camera.distance *= zoom;
        let w = if use_gpu { quality_width } else { 400 };
        let cfg = Settings {
            width: w,
            height: (w as f32 * viewport.height / viewport.width).max(1.) as usize,
            quality: 0,
            space: true,
            flight,
            combat,
            ..Settings::default()
        };
        if use_gpu {
            if let Err(e) = gpu.as_mut().unwrap().render(window, thread, camera, cfg) {
                use_gpu = false;
                message = format!("GPU: {e}; usando CPU");
                message_until = Instant::now() + std::time::Duration::from_secs(5);
                continue;
            }
        } else {
            let frame = render::render(&scene, camera, cfg);
            if cpu_texture
                .as_ref()
                .is_none_or(|t| t.width() != cfg.width as i32 || t.height() != cfg.height as i32)
            {
                let image =
                    Image::gen_image_color(cfg.width as i32, cfg.height as i32, Color::BLACK);
                let t = window.load_texture_from_image(thread, &image)?;
                t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                cpu_texture = Some(t);
            }
            cpu_texture
                .as_mut()
                .unwrap()
                .update_texture(&frame.pixels)?;
        }
        if save {
            let result = if use_gpu {
                gpu.as_ref().unwrap().pixels()
            } else {
                Ok(render::render(&scene, camera, cfg).pixels)
            };
            let result = result.and_then(|p| {
                crate::png::save("renders/vuelo-captura.png", cfg.width, cfg.height, &p)
                    .map_err(Into::into)
            });
            message = match result {
                Ok(()) => "Guardado: renders/vuelo-captura.png".into(),
                Err(e) => format!("Captura: {e}"),
            };
            message_until = Instant::now() + std::time::Duration::from_secs(4);
        }
        let fps = window.get_fps();
        let mut draw = window.begin_drawing(thread);
        let accent = Color::new(95, 218, 255, 255);
        draw.clear_background(Color::new(8, 15, 26, 255));
        if use_gpu {
            let t = gpu.as_ref().unwrap().texture().unwrap();
            display.draw(&mut draw, t, viewport);
        } else if let Some(t) = &cpu_texture {
            draw.draw_texture_pro(
                t,
                Rectangle::new(0., 0., t.width() as f32, t.height() as f32),
                viewport,
                Vector2::zero(),
                0.,
                Color::WHITE,
            );
        }
        draw.draw_text("MILLENNIUM FALCON / MODO NAVE", 20, 23, 24, Color::WHITE);
        let phase = if progress < 1. {
            format!(
                "ASCENSO {:03.0}%  |  {:.1} / {:.1} s",
                progress * 100.,
                if waiting { 0. } else { status.position },
                status.durations[Track::Takeoff as usize]
            )
        } else if flight.boost_age >= 0. {
            format!(
                "IMPULSO  |  {:.1} / {:.1} s",
                flight.boost_age, flight.boost_duration
            )
        } else {
            "EN EL ESPACIO  |  E para activar la estela".into()
        };
        draw.draw_text(&phase, 20, 67, 19, accent);
        draw.draw_rectangle_rounded(back, 0.2, 4, Color::new(27, 48, 65, 255));
        draw.draw_text(
            "1  Volver al diorama",
            back.x as i32 + 10,
            27,
            16,
            Color::WHITE,
        );
        draw.draw_rectangle_rounded(
            boost,
            0.2,
            4,
            if progress >= 1. {
                Color::new(20, 84, 112, 255)
            } else {
                Color::new(35, 44, 52, 255)
            },
        );
        draw.draw_text(
            "E  Impulso / reiniciar",
            boost.x as i32 + 12,
            69,
            16,
            accent,
        );
        draw.draw_text(
            &format!("Arrastrar / flechas: girar   Rueda / +/-: zoom   R: camara   9: ascenso   J Suavizado {}", if use_gpu && display.enabled && display.available() {"SI"} else {"NO"}),
            20,
            height - 60,
            16,
            Color::WHITE,
        );
        draw.draw_text(&format!("1-8: diorama   M: audio {}   Q: {}   T: CPU/GPU   S: captura   |   {} {}x{}  {} FPS",if status.muted {"NO"}else{"SI"},&format!("{}px",quality_width),if use_gpu {"GPU"}else{"CPU"},cfg.width,cfg.height,fps),20,height-32,16,accent);
        if combat.enabled {
            let alive = combat.hp.iter().filter(|&&hp| hp > 0).count();
            let label = if alive == 0 {
                "VICTORIA | 3 TIE destruidos | N: nueva oleada".to_string()
            } else {
                format!(
                    "F: disparar | TIE 1: {}/3   TIE 2: {}/3   TIE 3: {}/3 | N: reiniciar",
                    combat.hp[0], combat.hp[1], combat.hp[2]
                )
            };
            draw.draw_rectangle(12, 103, 760, 30, Color::new(8, 15, 26, 230));
            draw.draw_text(&label, 24, 110, 18, Color::new(255, 195, 100, 255));
        }
        if Instant::now() < message_until {
            draw.draw_text(&message, 24, 145, 18, Color::WHITE);
        }
    }
    Ok(0)
}
