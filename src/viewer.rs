//! Ventana compartida por el raytracer CPU y el shader GPU.
use crate::{
    camera::Camera,
    math::V,
    png,
    render::{self, Frame, Settings},
    scene::Scene,
};
use raylib::prelude::*;
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const VIEWS: [&str; 7] = [
    "1  Principal",
    "2  Motor",
    "3  Cabina",
    "4  Refraccion",
    "5  Superior",
    "6  Droide",
    "7  Vader",
];
pub(crate) fn preset(index: usize) -> Camera {
    match index {
        1 => Camera {
            yaw: 154.,
            pitch: 25.,
            distance: 19.,
            ..Camera::default()
        },
        2 => Camera {
            yaw: 28.,
            pitch: 15.,
            distance: 5.5,
            target: V::new(4.4, 2.05, -3.6),
            ..Camera::default()
        },
        3 => Camera {
            yaw: 27.,
            pitch: 12.,
            distance: 4.5,
            target: V::new(5.5, 0.9, -4.5),
            ..Camera::default()
        },
        4 => Camera {
            yaw: 38.,
            pitch: 72.,
            distance: 24.,
            ..Camera::default()
        },
        5 => Camera {
            yaw: 28.,
            pitch: 17.,
            distance: 4.6,
            target: V::new(-5.1, 0.70, -5.3),
            ..Camera::default()
        },
        6 => Camera {
            yaw: 20.,
            pitch: 12.,
            distance: 5.7,
            target: crate::scene::VADER_ORIGIN + V::new(0.2, 1.40, -0.12),
            ..Camera::default()
        },
        _ => Camera::default(),
    }
}

struct Job {
    camera: Camera,
    settings: Settings,
    generation: u64,
    level: u8,
    save: bool,
}
struct Completed {
    frame: Option<Frame>,
    generation: u64,
    level: u8,
    saved: Option<Result<(), String>>,
}

/// Control amortiguado de resolución; busca 30 cuadros calculados/s sin asumir la CPU.
fn adapt_width(width: usize, seconds: f32) -> usize {
    let ratio = ((1. / 30.) / seconds.max(0.001)).sqrt().clamp(0.8, 1.12);
    (((width as f32 * ratio) as usize / 16) * 16).clamp(192, 640)
}

#[derive(Clone, Copy, PartialEq)]
enum MotionQuality {
    Retina,
    Sharp,
    Adaptive,
}
impl MotionQuality {
    fn next(self) -> Self {
        match self {
            Self::Retina => Self::Sharp,
            Self::Sharp => Self::Adaptive,
            Self::Adaptive => Self::Retina,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Retina => "Q  Retina constante",
            Self::Sharp => "Q  Nitido 1200 px",
            Self::Adaptive => "Q  Fluido adaptativo",
        }
    }
    fn width(self, physical: i32, adaptive: usize) -> usize {
        match self {
            Self::Retina => (physical as usize).clamp(1600, 2560),
            Self::Sharp => 1200,
            Self::Adaptive => adaptive,
        }
    }
}

pub fn run(
    scene: Scene,
    mut camera: Camera,
    mut settings: Settings,
    auto_orbit: bool,
    adaptive: bool,
    prefer_gpu: bool,
) -> Result<(), Box<dyn Error>> {
    let (mut window, thread) = raylib::init()
        .size(1200, 800)
        .resizable()
        .highdpi()
        .title("Millennium Falcon | Diorama con Raytracing")
        .build();
    window.set_target_fps(60);
    window.set_window_min_size(1000, 600);
    let scene = Arc::new(scene);
    let mut gpu = None;
    let mut use_gpu = prefer_gpu;
    if prefer_gpu {
        match crate::gpu::Renderer::new(&mut window, &thread, &scene) {
            Ok(renderer) => gpu = Some(renderer),
            Err(error) => {
                eprintln!("GPU no disponible: {error}. Se utiliza CPU.");
                use_gpu = false;
            }
        }
    }
    let audio = crate::audio::Audio::new();
    if let Err(error) = crate::intro::play(&mut window, &thread, &audio) {
        eprintln!("Intro: {error}");
        audio.event(crate::audio::Event::FinishIntro);
    }
    // El Espacio que omite la intro no activa también el recorrido.
    let mut just_left_intro = true;
    let mut fade_in = Instant::now();
    let worker_scene = scene.clone();
    let cancel = Arc::new(AtomicU64::new(0));
    let worker_cancel = cancel.clone();
    let (jobs, requests) = mpsc::channel::<Job>();
    let (results, frames) = mpsc::channel::<Completed>();
    let worker = std::thread::spawn(move || {
        while let Ok(job) = requests.recv() {
            let frame = render::render_interruptible(
                &worker_scene,
                job.camera,
                job.settings,
                &worker_cancel,
                job.generation,
            );
            let saved = if job.save {
                frame.as_ref().map(|f| {
                    png::save("renders/captura.png", f.width, f.height, &f.pixels)
                        .map_err(|e| e.to_string())
                })
            } else {
                None
            };
            if results
                .send(Completed {
                    frame,
                    generation: job.generation,
                    level: job.level,
                    saved,
                })
                .is_err()
            {
                break;
            }
        }
    });
    let mut texture: Option<Texture2D> = None;
    let mut generation = 0;
    let mut busy = false;
    let mut running_level = 0;
    let mut preview_width = if use_gpu { 960 } else { 352 };
    let mut motion_quality = if adaptive {
        MotionQuality::Adaptive
    } else {
        MotionQuality::Sharp
    };
    let mut dirty = true;
    let mut refined_level = 0;
    let mut last_input = Instant::now();
    let mut orbit = auto_orbit;
    let mut help = true;
    let mut request_save = false;
    let mut notice = String::new();
    let mut notice_until = Instant::now();
    let mut frame_ms = 0.;
    let mut frame_size = (0, 0);
    let mut shown_level = 0;
    let mut previous_size = (0, 0, 0);
    let mut presented_frames = std::collections::VecDeque::new();
    use KeyboardKey::*;
    while !window.window_should_close() {
        let now = Instant::now();
        let dt = window.get_frame_time().min(0.05);
        let width = window.get_screen_width();
        let height = window.get_screen_height();
        let viewport = Rectangle::new(0., 96., width as f32, (height - 156) as f32);
        let aspect = viewport.width / viewport.height;
        let render_width = window.get_render_width();
        let mouse = window.get_mouse_position();
        let inside = mouse.y >= viewport.y && mouse.y < viewport.y + viewport.height;
        let click = window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mut changed = false;
        let quality_button = Rectangle::new(width as f32 - 254., 15., 234., 29.);
        if window.is_key_pressed(KEY_Q)
            || (click && quality_button.check_collision_point_rec(mouse))
        {
            motion_quality = motion_quality.next();
            generation += 1;
            cancel.store(generation, Ordering::Relaxed);
            changed = true;
        }
        if previous_size != (width, height, render_width) {
            previous_size = (width, height, render_width);
            changed = true;
        }
        if window.is_key_pressed(KEY_T) {
            if gpu.is_none() {
                match crate::gpu::Renderer::new(&mut window, &thread, &scene) {
                    Ok(renderer) => gpu = Some(renderer),
                    Err(error) => {
                        notice = format!("GPU no disponible: {error}");
                        notice_until = now + Duration::from_secs(8);
                    }
                }
            }
            if gpu.is_some() {
                use_gpu = !use_gpu;
                preview_width = if use_gpu { 960 } else { 352 };
                generation += 1;
                cancel.store(generation, Ordering::Relaxed);
                presented_frames.clear();
                changed = true;
            }
        }
        if !just_left_intro && window.is_key_pressed(KEY_SPACE) {
            orbit = !orbit;
        }
        if !just_left_intro && window.is_key_pressed(KEY_M) {
            audio.event(crate::audio::Event::ToggleMute);
        }
        if !just_left_intro && window.is_key_pressed(KEY_I) {
            if let Err(error) = crate::intro::play(&mut window, &thread, &audio) {
                eprintln!("Intro: {error}");
                audio.event(crate::audio::Event::FinishIntro);
            }
            camera = Camera::default();
            orbit = false;
            generation += 1;
            cancel.store(generation, Ordering::Relaxed);
            fade_in = Instant::now();
            changed = true;
        }
        just_left_intro = false;
        if window.is_key_pressed(KEY_C) {
            settings.space = !settings.space;
            changed = true;
        }
        if window.is_key_pressed(KEY_H) {
            help = !help;
        }
        if window.is_key_pressed(KEY_S) {
            request_save = true;
            orbit = false;
        }
        let keys = [
            KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN,
        ];
        for (i, key) in keys.iter().enumerate() {
            let button = Rectangle::new(20. + i as f32 * 116., 59., 108., 28.);
            if window.is_key_pressed(*key) || (click && button.check_collision_point_rec(mouse)) {
                audio.event(crate::audio::Event::View(i));
                camera = preset(i);
                orbit = false;
                changed = true;
            }
        }
        if window.is_key_pressed(KEY_R) {
            audio.event(crate::audio::Event::View(0));
            camera = Camera::default();
            orbit = false;
            changed = true;
        }
        for (key, flag) in [
            (KEY_F, &mut settings.reflections),
            (KEY_G, &mut settings.refractions),
            (KEY_B, &mut settings.skybox),
        ] {
            if window.is_key_pressed(key) {
                *flag = !*flag;
                changed = true;
            }
        }
        if inside && window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = window.get_mouse_delta();
            if delta.x != 0. || delta.y != 0. {
                camera.yaw += delta.x * 0.25;
                camera.pitch += delta.y * 0.25;
                changed = true;
            }
        }
        let horizontal =
            i32::from(window.is_key_down(KEY_RIGHT)) - i32::from(window.is_key_down(KEY_LEFT));
        let vertical =
            i32::from(window.is_key_down(KEY_UP)) - i32::from(window.is_key_down(KEY_DOWN));
        // Leer caracteres respeta la distribución del teclado (español, inglés y Shift).
        // Los eventos también capturan pulsaciones breves entre dos cuadros.
        let mut zoom_steps = 0;
        while let Some(character) = window.get_char_pressed() {
            zoom_steps += match character {
                '+' | '=' => -1,
                '-' | '−' => 1,
                _ => 0,
            };
        }
        if zoom_steps == 0 {
            zoom_steps = i32::from(window.is_key_pressed(KEY_KP_SUBTRACT))
                - i32::from(window.is_key_pressed(KEY_KP_ADD));
        }
        let zoom_in_button = Rectangle::new(width as f32 - 126., 59., 48., 28.);
        let zoom_out_button = Rectangle::new(width as f32 - 68., 59., 48., 28.);
        if click && zoom_in_button.check_collision_point_rec(mouse) {
            zoom_steps -= 1;
        }
        if click && zoom_out_button.check_collision_point_rec(mouse) {
            zoom_steps += 1;
        }
        let wheel = if inside {
            window.get_mouse_wheel_move()
        } else {
            0.
        };
        if orbit || horizontal != 0 || vertical != 0 || zoom_steps != 0 || wheel != 0. {
            camera.yaw += (horizontal as f32 * 45. + if orbit { 12. } else { 0. }) * dt;
            camera.pitch += vertical as f32 * 35. * dt;
            camera.distance *= ((zoom_steps as f32 * 0.12 - wheel * 0.10) * 0.8).exp();
            changed = true;
        }
        camera.pitch = camera.pitch.clamp(-10., 82.);
        camera.distance = camera.distance.clamp(3., 45.);
        camera.yaw = camera.yaw.rem_euclid(360.);
        if changed {
            last_input = now;
            dirty = true;
            refined_level = 0;
            // Las previsualizaciones cortas terminan; los renders de calidad se cancelan por fila.
            if busy && running_level > 0 {
                generation += 1;
                cancel.store(generation, Ordering::Relaxed);
            }
            request_save = false;
        }
        while let Ok(done) = frames.try_recv() {
            busy = false;
            if let Some(saved) = done.saved {
                notice = match saved {
                    Ok(()) => "Imagen guardada en renders/captura.png".into(),
                    Err(e) => format!("No se pudo guardar: {e}"),
                };
                notice_until = now + Duration::from_secs(5);
            }
            if use_gpu || done.generation != generation {
                continue;
            }
            if let Some(frame) = done.frame {
                if done.level == 0 && motion_quality == MotionQuality::Adaptive {
                    preview_width = adapt_width(preview_width, frame.elapsed.as_secs_f32());
                }
                frame_ms = frame.elapsed.as_secs_f32() * 1000.;
                frame_size = (frame.width, frame.height);
                shown_level = done.level;
                presented_frames.push_back(now);
                let needs_texture = texture.as_ref().is_none_or(|t| {
                    t.width() != frame.width as i32 || t.height() != frame.height as i32
                });
                if needs_texture {
                    let image = Image::gen_image_color(
                        frame.width as i32,
                        frame.height as i32,
                        Color::BLACK,
                    );
                    let mut new_texture = window.load_texture_from_image(&thread, &image)?;
                    new_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                    new_texture.update_texture(&frame.pixels)?;
                    texture = Some(new_texture);
                } else if let Some(t) = &mut texture {
                    t.update_texture(&frame.pixels)?;
                }
            }
        }
        while presented_frames
            .front()
            .is_some_and(|t| now.duration_since(*t).as_secs_f32() > 1.)
        {
            presented_frames.pop_front();
        }
        if !busy && !use_gpu {
            let idle = now.duration_since(last_input).as_secs_f32();
            let level = if request_save {
                Some(3)
            } else if dirty {
                Some(0)
            } else if idle > 0.22 && refined_level == 0 {
                Some(1)
            } else if idle > 0.8 && refined_level == 1 {
                Some(2)
            } else {
                None
            };
            if let Some(level) = level {
                let w = match level {
                    0 => motion_quality.width(render_width, preview_width),
                    1 => motion_quality.width(render_width, (width as usize).clamp(1000, 1280)),
                    2 => (render_width as usize).clamp(1600, 2560),
                    _ => 2560,
                };
                let cfg = Settings {
                    width: w,
                    height: ((w as f32 / aspect) as usize).max(1),
                    quality: match level {
                        0 => 0,
                        1 => 1,
                        _ => 2,
                    },
                    ..settings
                };
                jobs.send(Job {
                    camera,
                    settings: cfg,
                    generation,
                    level,
                    save: level == 3,
                })?;
                busy = true;
                running_level = level;
                dirty = false;
                request_save = false;
                if level > 0 {
                    refined_level = level;
                }
            }
        }
        if use_gpu {
            let idle = now.duration_since(last_input).as_secs_f32();
            let level = if request_save {
                Some(3)
            } else if dirty {
                Some(0)
            } else if idle > 0.22 && refined_level == 0 {
                Some(1)
            } else if idle > 0.8 && refined_level == 1 {
                Some(2)
            } else {
                None
            };
            if let Some(level) = level {
                // Solo el modo Fluido cambia resolución al moverse; Q conserva
                // los dos modos de nitidez fija y todos los efectos ópticos.
                if level == 0 && motion_quality == MotionQuality::Adaptive && frame_size.0 > 0 {
                    let ratio = ((1. / 30.) / window.get_frame_time().max(0.001))
                        .sqrt()
                        .clamp(0.9, 1.06);
                    preview_width =
                        (((preview_width as f32 * ratio) as usize / 32) * 32).clamp(480, 1200);
                }
                let w = match level {
                    0 => motion_quality.width(render_width, preview_width),
                    1 => motion_quality.width(render_width, 1200),
                    2 => (render_width as usize).clamp(1600, 2560),
                    _ => 2560,
                };
                let cfg = Settings {
                    width: w,
                    height: ((w as f32 / aspect) as usize).max(1),
                    quality: if level == 0 {
                        0
                    } else if level == 1 {
                        1
                    } else {
                        2
                    },
                    ..settings
                };
                let renderer = gpu.as_mut().unwrap();
                match renderer.render(&mut window, &thread, camera, cfg) {
                    Ok(()) => {
                        if level == 3 {
                            match renderer.pixels().and_then(|pixels| {
                                png::save("renders/captura.png", cfg.width, cfg.height, &pixels)
                                    .map_err(Into::into)
                            }) {
                                Ok(()) => {
                                    notice = "Imagen GPU guardada en renders/captura.png".into()
                                }
                                Err(error) => notice = format!("No se pudo guardar: {error}"),
                            }
                            notice_until = now + Duration::from_secs(5);
                        }
                        frame_size = (cfg.width, cfg.height);
                        frame_ms = window.get_frame_time() * 1000.;
                        shown_level = level;
                        presented_frames.push_back(now);
                        dirty = false;
                        request_save = false;
                        if level > 0 {
                            refined_level = level;
                        }
                    }
                    Err(error) => {
                        notice = format!("GPU: {error}; se utiliza CPU");
                        notice_until = now + Duration::from_secs(8);
                        use_gpu = false;
                        dirty = true;
                    }
                }
            }
        }
        let ui_fps = window.get_fps();
        let mut draw = window.begin_drawing(&thread);
        let bg = Color::new(13, 19, 28, 255);
        let accent = Color::new(242, 186, 105, 255);
        let muted = Color::new(166, 183, 198, 255);
        draw.clear_background(bg);
        if use_gpu && let Some(t) = gpu.as_ref().and_then(|g| g.texture()) {
            draw.draw_texture_pro(
                t,
                Rectangle::new(0., 0., t.width() as f32, -(t.height() as f32)),
                viewport,
                Vector2::zero(),
                0.,
                Color::WHITE,
            );
        } else if let Some(t) = &texture {
            draw.draw_texture_pro(
                t,
                Rectangle::new(0., 0., t.width() as f32, t.height() as f32),
                viewport,
                Vector2::zero(),
                0.,
                Color::WHITE,
            );
        } else {
            draw.draw_text("Preparando el diorama...", 35, height / 2, 24, accent);
        }
        draw.draw_text("MILLENNIUM FALCON", 20, 17, 27, Color::WHITE);
        draw.draw_text(
            if settings.space {
                "ORBITA / ESTRELLA DE LA MUERTE"
            } else {
                "DOCKING BAY 94 / TATOOINE"
            },
            400,
            24,
            16,
            accent,
        );
        for (i, label) in VIEWS.iter().enumerate() {
            draw.draw_rectangle_rounded(
                Rectangle::new(20. + i as f32 * 116., 59., 108., 28.),
                0.2,
                4,
                Color::new(32, 44, 57, 255),
            );
            draw.draw_text(label, 28 + i as i32 * 116, 65, 16, muted);
        }
        for (button, label) in [(zoom_in_button, "+"), (zoom_out_button, "-")] {
            draw.draw_rectangle_rounded(button, 0.2, 4, Color::new(32, 44, 57, 255));
            draw.draw_text(label, button.x as i32 + 17, button.y as i32 + 3, 24, accent);
        }
        draw.draw_rectangle_rounded(quality_button, 0.2, 4, Color::new(32, 44, 57, 255));
        draw.draw_text(
            motion_quality.label(),
            quality_button.x as i32 + 10,
            22,
            18,
            accent,
        );
        let state = if shown_level == 0 {
            match motion_quality {
                MotionQuality::Retina => "RETINA",
                MotionQuality::Sharp => "NITIDO",
                MotionQuality::Adaptive => "FLUIDO",
            }
        } else if shown_level == 1 {
            "REFINANDO"
        } else {
            "DETALLE"
        };
        draw.draw_text(
            &format!(
                "{} {state}  |  {}x{}  |  {:.0} ms  |  {} imagenes/s  |  UI {} FPS  |  Zoom {:.1}",
                if use_gpu { "GPU" } else { "CPU" },
                frame_size.0,
                frame_size.1,
                frame_ms,
                presented_frames.len(),
                ui_fps,
                camera.distance
            ),
            20,
            height - 49,
            16,
            muted,
        );
        draw.draw_text(
            &format!(
                "F Reflejos {}   G Refraccion {}   B Cielo {}   C Entorno {}   |   T CPU/GPU   I Intro   M Audio {}   H Ayuda",
                if settings.reflections { "SI" } else { "NO" },
                if settings.refractions { "SI" } else { "NO" },
                if settings.skybox { "SI" } else { "NO" },
                if settings.space { "ESPACIO" } else { "TATOOINE" },
                if audio.status().muted { "NO" } else { "SI" }
            ),
            20,
            height - 26,
            16,
            accent,
        );
        if help {
            draw.draw_rectangle(16, 110, 470, 69, Color::new(13, 19, 28, 220));
            draw.draw_text(
                "Arrastrar / flechas: girar   Rueda / +/-: zoom",
                28,
                123,
                17,
                Color::WHITE,
            );
            draw.draw_text(
                "Espacio: recorrido   R: inicio   S: guardar PNG",
                28,
                148,
                17,
                muted,
            );
        }
        let fade = (1. - fade_in.elapsed().as_secs_f32()).max(0.);
        if fade > 0. {
            draw.draw_rectangle(
                0,
                0,
                width,
                height,
                Color::new(0, 0, 0, (fade * 255.) as u8),
            );
        }
        if now < notice_until {
            draw.draw_text(&notice, 24, height - 87, 18, accent);
        }
    }
    cancel.fetch_add(1, Ordering::Relaxed);
    drop(jobs);
    worker
        .join()
        .map_err(|_| "El hilo de renderizado termino inesperadamente")?;
    Ok(())
}

pub fn benchmark(scene: &Scene) {
    println!(
        "Benchmark CPU: todos los efectos activos, sin ventana ni PNG; promedio y peor cuadro."
    );
    for index in [0, 1, 2, 3] {
        let mut camera = preset(index);
        let mut width = 352;
        let mut times = Vec::new();
        for frame in 0..30 {
            camera.yaw += 0.7;
            let result = render::render(
                scene,
                camera,
                Settings {
                    width,
                    height: width * 9 / 16,
                    quality: 0,
                    ..Settings::default()
                },
            );
            width = adapt_width(width, result.elapsed.as_secs_f32());
            if frame >= 10 {
                times.push(result.elapsed.as_secs_f32());
            }
        }
        let avg = times.iter().sum::<f32>() / times.len() as f32;
        let max = times.iter().copied().fold(0_f32, f32::max);
        println!(
            "{}: ancho final {} px, media {:.1} ms ({:.1} cuadros/s), peor {:.1} ms",
            VIEWS[index],
            width,
            avg * 1000.,
            1. / avg,
            max * 1000.
        );
    }
}

pub fn benchmark_sharp(scene: &Scene) {
    println!(
        "Nitidez fija: efectos activos, 1 muestra/pixel, calidad interactiva; 1 calentamiento + 4 cuadros/vista."
    );
    for width in [1200, 2400] {
        for index in [0, 2] {
            let mut camera = preset(index);
            let mut times = Vec::new();
            for n in 0..5 {
                camera.yaw += 0.7;
                let frame = render::render(
                    scene,
                    camera,
                    Settings {
                        width,
                        height: width * 644 / 1200,
                        quality: 0,
                        ..Settings::default()
                    },
                );
                if n > 0 {
                    times.push(frame.elapsed.as_secs_f32());
                }
            }
            let avg = times.iter().sum::<f32>() / times.len() as f32;
            println!(
                "{} | {} px | media {:.1} ms ({:.1} cuadros/s CPU) | peor {:.1} ms",
                VIEWS[index],
                width,
                avg * 1000.,
                1. / avg,
                times.iter().copied().fold(0_f32, f32::max) * 1000.
            );
        }
    }
}
