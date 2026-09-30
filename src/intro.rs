//! Prólogo de presentación. El texto se proyecta desde una textura; la escena
//! posterior continúa usando el raytracer propio CPU/GPU.
use raylib::{ffi, prelude::*};
use std::{error::Error, time::Instant};
const DURATION: f32 = 26.;
struct Intro {
    shader: Shader,
    atlas: RenderTexture2D,
    font: WeakFont,
    // Mantiene vivo el recurso al que apunta WeakFont, si hay fuente del sistema.
    _owned_font: Option<Font>,
}
impl Intro {
    fn new(window: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Box<dyn Error>> {
        let glyphs: String = (32u8..=126)
            .map(char::from)
            .chain("áéíóúñÁÉÍÓÚÑ¿¡…".chars())
            .collect();
        let font_path = [
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "C:/Windows/Fonts/arialbd.ttf",
        ]
        .into_iter()
        .find(|p| std::path::Path::new(p).is_file());
        let owned_font =
            font_path.and_then(|p| window.load_font_ex(thread, p, 72, Some(&glyphs)).ok());
        // La fuente propia se mantiene viva en Intro; WeakFont no la descarga.
        let font = owned_font
            .as_ref()
            .map(|f| unsafe { WeakFont::from_raw(*f.as_ref()) })
            .unwrap_or_else(|| window.get_font_default());
        let mut atlas = window.load_render_texture(thread, 1400, 1900)?;
        atlas
            .texture_mut()
            .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        {
            let mut draw = window.begin_texture_mode(thread, &mut atlas);
            draw.clear_background(Color::BLANK);
            let lines = [
                ("EPISODIO II", 72., 100.),
                ("UN UNIVERSO DE RAYOS", 68., 220.),
                ("Un joven programador aceptó", 57., 430.),
                ("un desafío: construir un universo", 57., 520.),
                ("de bloques y darle vida", 57., 610.),
                ("con rayos de luz.", 57., 700.),
                ("Bajo los soles de Tatooine,", 57., 900.),
                ("el Halcón Milenario aguarda.", 57., 990.),
                ("Entre sombras, reflejos y cristal,", 57., 1190.),
                ("una presencia oscura se aproxima.", 57., 1280.),
                ("El destino de esta escena", 57., 1480.),
                ("está en tus manos…", 57., 1570.),
            ];
            for (text, size, y) in lines {
                let w = font.measure_text(text, size, 1.).x;
                draw.draw_text_ex(
                    &font,
                    text,
                    Vector2::new((1400. - w) * 0.5, y),
                    size,
                    1.,
                    Color::new(255, 204, 55, 255),
                );
            }
        }
        let shader =
            window.load_shader_from_memory(thread, None, Some(include_str!("../shaders/crawl.fs")));
        if !shader.is_shader_valid() || shader.as_ref().id == unsafe { ffi::rlGetShaderIdDefault() }
        {
            return Err("No se pudo compilar la introducción".into());
        }
        Ok(Self {
            shader,
            atlas,
            font,
            _owned_font: owned_font,
        })
    }
    fn draw<D: RaylibDraw>(
        &mut self,
        draw: &mut D,
        time: f32,
        physical: Vector2,
        logical: Vector2,
    ) {
        let loc = self.shader.get_shader_location("resolution");
        self.shader.set_shader_value(loc, physical);
        let loc = self.shader.get_shader_location("elapsed");
        self.shader.set_shader_value(loc, time);
        let loc = self.shader.get_shader_location("crawl");
        let raw = *self.shader.as_ref();
        {
            let mut pass = draw.begin_shader_mode(&mut self.shader);
            // Vincular el sampler después de que BeginShaderMode vacíe el batch.
            unsafe {
                ffi::SetShaderValueTexture(raw, loc, *self.atlas.texture().as_ref());
            }
            pass.draw_rectangle(0, 0, logical.x as i32, logical.y as i32, Color::WHITE);
        }
        if time < 4.2 {
            let (text, size, color) = if time < 2.2 {
                (
                    "En una galaxia muy lejana…",
                    logical.x * 0.032,
                    Color::new(93, 213, 240, 255),
                )
            } else {
                (
                    "MILLENNIUM FALCON",
                    logical.x * 0.052,
                    Color::new(255, 204, 55, 255),
                )
            };
            let alpha = if time < 2.2 {
                (time * 2.).min(1.) * ((2.2 - time) * 3.).min(1.)
            } else {
                ((time - 2.2) * 3.).min(1.) * ((4.2 - time) * 3.).min(1.)
            };
            let w = self.font.measure_text(text, size, 1.).x;
            draw.draw_text_ex(
                &self.font,
                text,
                Vector2::new((logical.x - w) * 0.5, logical.y * 0.44),
                size,
                1.,
                Color::new(color.r, color.g, color.b, (alpha * 255.) as u8),
            );
        }
    }
}
pub fn play(window: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), Box<dyn Error>> {
    let mut intro = Intro::new(window, thread)?;
    let path = [
        "assets/audio/intro.ogg",
        "assets/audio/intro.mp3",
        "assets/audio/intro.wav",
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).is_file());
    let audio = if path.is_some() {
        RaylibAudio::init_audio_device().ok()
    } else {
        None
    };
    let music = audio
        .as_ref()
        .and_then(|a| path.and_then(|p| a.new_music(p).ok()));
    if let Some(m) = &music {
        m.set_volume(0.45);
        m.play_stream();
    }
    let start = Instant::now();
    let mut muted = false;
    while !window.window_should_close() {
        let time = start.elapsed().as_secs_f32();
        if time >= DURATION
            || window.is_key_pressed(KeyboardKey::KEY_ENTER)
            || window.is_key_pressed(KeyboardKey::KEY_SPACE)
        {
            break;
        }
        if window.is_key_pressed(KeyboardKey::KEY_M) {
            muted = !muted;
        }
        if let Some(m) = &music {
            m.update_stream();
            m.set_volume(if muted {
                0.
            } else {
                0.45 * ((DURATION - time) / 2.).clamp(0., 1.)
            });
        }
        let logical = Vector2::new(
            window.get_screen_width() as f32,
            window.get_screen_height() as f32,
        );
        let physical = Vector2::new(
            window.get_render_width() as f32,
            window.get_render_height() as f32,
        );
        let mut draw = window.begin_drawing(thread);
        draw.clear_background(Color::BLACK);
        intro.draw(&mut draw, time, physical, logical);
        draw.draw_text(
            "ENTER / ESPACIO  Saltar     M  Silenciar",
            24,
            logical.y as i32 - 34,
            16,
            Color::new(150, 164, 183, 255),
        );
    }
    if let Some(m) = &music {
        m.stop_stream();
    }
    Ok(())
}
/// Captura reproducible para revisar perspectiva, texto y fases sin automatizar entradas.
pub fn snapshot(seconds: f32, path: &str) -> Result<(), Box<dyn Error>> {
    let (mut window, thread) = raylib::init()
        .size(1000, 650)
        .hidden()
        .title("Intro")
        .build();
    let mut intro = Intro::new(&mut window, &thread)?;
    let mut target = window.load_render_texture(&thread, 1200, 800)?;
    {
        let mut draw = window.begin_texture_mode(&thread, &mut target);
        intro.draw(
            &mut draw,
            seconds,
            Vector2::new(1200., 800.),
            Vector2::new(1200., 800.),
        );
    }
    let image = target.texture().load_image()?;
    crate::png::save(path, 1200, 800, &image.get_image_data_u8(true))?;
    Ok(())
}
