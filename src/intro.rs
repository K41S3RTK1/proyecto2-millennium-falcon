//! Prólogo de presentación. El texto se proyecta desde una textura; la escena
//! posterior continúa usando el raytracer propio CPU/GPU.
use crate::audio::{Audio, Event, Track};
use raylib::{ffi, prelude::*};
use std::{error::Error, time::Instant};
pub const BLUE_SECONDS: f32 = 4.;
pub const DEFAULT_DURATION: f32 = BLUE_SECONDS + 90.112;
const TITLE_END: f32 = BLUE_SECONDS + 5.;
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
        let mut atlas = window.load_render_texture(thread, 1400, 4000)?;
        atlas
            .texture_mut()
            .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        {
            let mut draw = window.begin_texture_mode(thread, &mut atlas);
            draw.clear_background(Color::BLANK);
            let story = [
                "En una galaxia muy lejana,",
                "una poderosa IA se apoderó",
                "de muchos de los androides,",
                "creando una nueva guerra.",
                "",
                "Con la destrucción de la",
                "Estrella de la Muerte, el Imperio",
                "entendió que ya no tenía el poder.",
                "",
                "Darth Vader se unió al equipo",
                "del Halcón Milenario y a los",
                "rebeldes, en busca de la",
                "derrota de la IA.",
                "",
                "Un joven programador aceptó",
                "un desafío: construir un universo",
                "de bloques y darle vida",
                "con rayos de luz.",
                "",
                "Bajo los soles de Tatooine,",
                "el Halcón Milenario aguarda",
                "su próxima misión.",
                "",
                "Entre sombras, reflejos y cristal,",
                "una presencia oscura se aproxima.",
                "",
                "El destino de esta escena",
                "está en tus manos…",
            ];
            let mut lines = vec![("EPISODIO II", 72., 100.), ("UNIVERSO DE RAYOS", 68., 230.)];
            lines.extend(
                story
                    .iter()
                    .enumerate()
                    .map(|(i, &line)| (line, 57., 480. + i as f32 * 115.)),
            );
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
        duration: f32,
        physical: Vector2,
        logical: Vector2,
    ) {
        let loc = self.shader.get_shader_location("resolution");
        self.shader.set_shader_value(loc, physical);
        let loc = self.shader.get_shader_location("elapsed");
        self.shader.set_shader_value(loc, time);
        let loc = self.shader.get_shader_location("duration");
        self.shader.set_shader_value(loc, duration);
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
        if time < TITLE_END {
            let (text, size, color) = if time < BLUE_SECONDS {
                (
                    "En una galaxia muy lejana…",
                    logical.x * 0.032,
                    Color::new(93, 213, 240, 255),
                )
            } else {
                (
                    "MILLENIUM FALCON",
                    logical.x * 0.052,
                    Color::new(255, 204, 55, 255),
                )
            };
            let alpha = if time < BLUE_SECONDS {
                (time * 2.).min(1.) * ((BLUE_SECONDS - time) * 2.).min(1.)
            } else {
                ((TITLE_END - time) * 1.5).clamp(0., 1.)
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
pub fn play(
    window: &mut RaylibHandle,
    thread: &RaylibThread,
    audio: &Audio,
) -> Result<(), Box<dyn Error>> {
    audio.event(Event::BeginIntro);
    let mut intro = Intro::new(window, thread)?;
    let duration = BLUE_SECONDS + audio.theme_duration();
    let start = Instant::now();
    let mut theme_started = false;
    // El reloj del audio gobierna las letras para evitar desfase si un cuadro tarda.
    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_ENTER)
            || window.is_key_pressed(KeyboardKey::KEY_SPACE)
        {
            audio.event(Event::SkipIntro);
            return Ok(());
        }
        if window.is_key_pressed(KeyboardKey::KEY_M) {
            audio.event(Event::ToggleMute);
        }
        let elapsed = start.elapsed().as_secs_f32();
        if !theme_started && elapsed >= BLUE_SECONDS {
            audio.event(Event::StartTheme);
            theme_started = true;
        }
        let status = audio.status();
        if theme_started && status.theme_finished {
            audio.event(Event::FinishIntro);
            return Ok(());
        }
        let time = if theme_started {
            BLUE_SECONDS
                + if status.track == Some(Track::Theme) {
                    status.position
                } else {
                    0.
                }
        } else {
            elapsed
        };
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
        intro.draw(&mut draw, time, duration, physical, logical);
        draw.draw_text(
            if status.muted {
                "ESPACIO / ENTER  Saltar     M  Activar audio"
            } else {
                "ESPACIO / ENTER  Saltar     M  Silenciar"
            },
            24,
            logical.y as i32 - 34,
            16,
            Color::new(150, 164, 183, 255),
        );
    }
    audio.event(Event::Stop);
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
            DEFAULT_DURATION,
            Vector2::new(1200., 800.),
            Vector2::new(1200., 800.),
        );
    }
    let image = target.texture().load_image()?;
    crate::png::save(path, 1200, 800, &image.get_image_data_u8(true))?;
    Ok(())
}
