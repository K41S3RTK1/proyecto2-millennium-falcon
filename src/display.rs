//! Pase ligero de presentación; el raytracer y sus exportaciones conservan su salida original.
use raylib::{ffi, prelude::*};
pub struct Display {
    shader: Option<Shader>,
    pub enabled: bool,
}
impl Display {
    pub fn new(window: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let shader = window.load_shader_from_memory(
            thread,
            None,
            Some(include_str!("../shaders/display.fs")),
        );
        let valid = shader.is_shader_valid()
            && shader.as_ref().id != unsafe { ffi::rlGetShaderIdDefault() };
        if !valid {
            eprintln!("Suavizado no disponible: se conserva la imagen original");
        }
        Self {
            shader: valid.then_some(shader),
            enabled: true,
        }
    }
    pub fn available(&self) -> bool {
        self.shader.is_some()
    }
    pub fn draw(
        &mut self,
        draw: &mut (impl RaylibShaderModeExt + RaylibDraw),
        texture: &impl RaylibTexture2D,
        viewport: Rectangle,
    ) {
        let source = Rectangle::new(0., 0., texture.width() as f32, -(texture.height() as f32));
        if self.enabled
            && let Some(shader) = &mut self.shader
        {
            let loc = shader.get_shader_location("texelSize");
            shader.set_shader_value(
                loc,
                Vector2::new(1. / texture.width() as f32, 1. / texture.height() as f32),
            );
            let mut pass = draw.begin_shader_mode(shader);
            pass.draw_texture_pro(texture, source, viewport, Vector2::zero(), 0., Color::WHITE);
        } else {
            draw.draw_texture_pro(texture, source, viewport, Vector2::zero(), 0., Color::WHITE);
        }
    }
}
