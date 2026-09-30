//! Raytracing propio GLSL 330. Raylib administra el contexto y los recursos GPU.
use crate::{camera::Camera, material::Texture, math::V, render::Settings, scene::Scene};
use raylib::{ffi, prelude::*};
use std::error::Error;

fn vector(v: V) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}
fn float_texture(
    data: &[[f32; 4]],
    width: usize,
    height: usize,
) -> Result<Texture2D, Box<dyn Error>> {
    if data.len() != width * height {
        return Err("Dimensiones de textura GPU inválidas".into());
    }
    let format = PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32G32B32A32 as i32;
    // rlLoadTexture copia los floats durante la llamada. El Vec conserva su
    // propietario Rust; únicamente la textura OpenGL pasa al wrapper RAII.
    let id =
        unsafe { ffi::rlLoadTexture(data.as_ptr().cast(), width as i32, height as i32, format, 1) };
    if id == 0 {
        return Err("No se pudo cargar la textura RGBA32F".into());
    }
    Ok(unsafe {
        Texture2D::from_raw(ffi::Texture {
            id,
            width: width as i32,
            height: height as i32,
            mipmaps: 1,
            format,
        })
    })
}
pub struct Renderer {
    shader: Shader,
    data: Texture2D,
    sky: Texture2D,
    space: Texture2D,
    target: Option<RenderTexture2D>,
}
impl Renderer {
    pub fn new(
        window: &mut RaylibHandle,
        thread: &RaylibThread,
        scene: &Scene,
    ) -> Result<Self, Box<dyn Error>> {
        let (mut data, blocks) = scene.bvh.gpu_data(&scene.blocks);
        let mut shader = window.load_shader_from_memory(
            thread,
            None,
            Some(include_str!("../shaders/raytrace.fs")),
        );
        if !shader.is_shader_valid() || shader.as_ref().id == unsafe { ffi::rlGetShaderIdDefault() }
        {
            return Err("El shader de raytracing no compiló; usa el modo CPU".into());
        }
        let node_count = data.len() / 3;
        let block_base = data.len();
        for b in blocks {
            data.push([
                b.bounds.lo.x,
                b.bounds.lo.y,
                b.bounds.lo.z,
                b.material as f32,
            ]);
            data.push([
                b.bounds.hi.x,
                b.bounds.hi.y,
                b.bounds.hi.z,
                scene.materials[b.material].transparency,
            ]);
            data.push([b.tint.x, b.tint.y, b.tint.z, 0.]);
        }
        let material_base = data.len();
        for m in &scene.materials {
            data.push([m.albedo.x, m.albedo.y, m.albedo.z, m.specular]);
            data.push([m.shininess, m.transparency, m.reflectivity, m.ior]);
            let tex = match m.texture {
                Texture::Hull => 0.,
                Texture::Dark => 1.,
                Texture::Glass => 2.,
                Texture::Sand => 3.,
                Texture::Engine => 4.,
                Texture::Cloth => 5.,
                Texture::Armor => 6.,
                Texture::Plasma => 7.,
            };
            data.push([m.emission.x, m.emission.y, m.emission.z, tex]);
        }
        let light_base = data.len();
        for &light in &scene.lights {
            let p = light.position;
            let c = light.color;
            let intensity = light.intensity;
            data.push([p.x, p.y, p.z, intensity]);
            data.push([c.x, c.y, c.z, light.radius]);
        }
        let data_height = data.len().div_ceil(1024);
        data.resize(data_height * 1024, [0.; 4]);
        let data = float_texture(&data, 1024, data_height)?;
        let (faces, size) = scene.sky.faces();
        let sky: Vec<_> = faces
            .iter()
            .flatten()
            .map(|v| [v.x, v.y, v.z, 1.])
            .collect();
        let sky = float_texture(&sky, size, size * 6)?;
        let (faces, space_size) = scene.space.faces();
        let space: Vec<_> = faces
            .iter()
            .flatten()
            .map(|v| [v.x, v.y, v.z, 1.])
            .collect();
        let space = float_texture(&space, space_size, space_size * 6)?;
        for (name, value) in [
            ("nodeCount", node_count),
            ("blockBase", block_base),
            ("materialBase", material_base),
            ("lightBase", light_base),
            ("lightCount", scene.lights.len()),
            ("skySize", size),
        ] {
            let loc = shader.get_shader_location(name);
            shader.set_shader_value(loc, value as i32);
        }
        for (name, value) in [
            ("saberBottom", crate::scene::SABER_BOTTOM),
            ("saberTop", crate::scene::SABER_TOP),
        ] {
            let loc = shader.get_shader_location(name);
            shader.set_shader_value(loc, vector(value));
        }
        println!(
            "GPU: {} nodos BVH, {} bloques; GLSL 330, datos RGBA32F",
            node_count,
            scene.blocks.len()
        );
        Ok(Self {
            shader,
            data,
            sky,
            space,
            target: None,
        })
    }
    pub fn render(
        &mut self,
        window: &mut RaylibHandle,
        thread: &RaylibThread,
        camera: Camera,
        cfg: Settings,
    ) -> Result<(), Box<dyn Error>> {
        if self.target.as_ref().is_none_or(|t| {
            t.texture().width() != cfg.width as i32 || t.texture().height() != cfg.height as i32
        }) {
            let mut target =
                window.load_render_texture(thread, cfg.width as u32, cfg.height as u32)?;
            target
                .texture_mut()
                .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            self.target = Some(target);
        }
        let eye = camera.position();
        let forward = (camera.target - eye).unit();
        let right = forward.cross(V::new(0., 1., 0.)).unit();
        let up = right.cross(forward);
        for (name, value) in [
            ("eye", eye),
            ("forward", forward),
            ("right", right),
            ("up", up),
        ] {
            let loc = self.shader.get_shader_location(name);
            self.shader.set_shader_value(loc, vector(value));
        }
        for (name, value) in [
            ("quality", cfg.quality as i32),
            ("reflections", i32::from(cfg.reflections)),
            ("refractions", i32::from(cfg.refractions)),
            ("skyEnabled", i32::from(cfg.skybox)),
            ("spaceMode", i32::from(cfg.space)),
            (
                "skySize",
                if cfg.space {
                    self.space.width()
                } else {
                    self.sky.width()
                },
            ),
        ] {
            let loc = self.shader.get_shader_location(name);
            self.shader.set_shader_value(loc, value);
        }
        let loc = self.shader.get_shader_location("resolution");
        self.shader
            .set_shader_value(loc, Vector2::new(cfg.width as f32, cfg.height as f32));
        let loc = self.shader.get_shader_location("cameraScale");
        self.shader
            .set_shader_value(loc, (camera.fov.to_radians() * 0.5).tan());
        let mut target = window.begin_texture_mode(thread, self.target.as_mut().unwrap());
        let scene_loc = self.shader.get_shader_location("sceneData");
        let sky_loc = self.shader.get_shader_location("skyData");
        let raw_shader = *self.shader.as_ref();
        let mut draw = target.begin_shader_mode(&mut self.shader);
        // BeginShaderMode vacía el batch y sus samplers. Vincular después;
        // los recursos siguen vivos hasta que se termina este pase.
        unsafe {
            ffi::SetShaderValueTexture(raw_shader, scene_loc, *self.data.as_ref());
            ffi::SetShaderValueTexture(
                raw_shader,
                sky_loc,
                *if cfg.space {
                    self.space.as_ref()
                } else {
                    self.sky.as_ref()
                },
            );
        }
        draw.draw_rectangle(0, 0, cfg.width as i32, cfg.height as i32, Color::WHITE);
        Ok(())
    }
    pub fn texture(&self) -> Option<&WeakTexture2D> {
        self.target.as_ref().map(|t| t.texture())
    }
    /// La lectura sincroniza la GPU: también sirve para medir cuadros terminados.
    pub fn pixels(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let image = self.texture().ok_or("No hay cuadro GPU")?.load_image()?;
        let colors = image.get_image_data();
        let mut pixels = Vec::with_capacity(colors.len() * 4);
        for row in colors.chunks_exact(image.width() as usize).rev() {
            pixels.extend(row.iter().flat_map(|c| [c.r, c.g, c.b, c.a]));
        }
        Ok(pixels)
    }
}

pub fn check(
    scene: &Scene,
    camera: Camera,
    cfg: Settings,
    output: &str,
    benchmark: bool,
) -> Result<(), Box<dyn Error>> {
    let (mut window, thread) = raylib::init()
        .size(640, 400)
        .hidden()
        .title("Validacion GPU")
        .build();
    let mut renderer = Renderer::new(&mut window, &thread, scene)?;
    renderer.render(&mut window, &thread, camera, cfg)?;
    let pixels = renderer.pixels()?;
    crate::png::save(output, cfg.width, cfg.height, &pixels)?;
    println!("GPU: imagen guardada en {output}");
    if benchmark {
        for index in [0, 1, 2, 3, 4, 5, 6] {
            let mut cam = crate::viewer::preset(index);
            let mut times = Vec::new();
            for n in 0..13 {
                cam.yaw += 0.7;
                let start = std::time::Instant::now();
                renderer.render(&mut window, &thread, cam, cfg)?;
                let _pixels = renderer.pixels()?;
                if n > 2 {
                    times.push(start.elapsed().as_secs_f32());
                }
            }
            times.sort_by(f32::total_cmp);
            let avg = times.iter().sum::<f32>() / times.len() as f32;
            println!(
                "GPU vista {} {}x{} q{}: media {:.2} ms ({:.1} FPS), mediana {:.2}, peor {:.2}; incluye lectura GPU",
                index + 1,
                cfg.width,
                cfg.height,
                cfg.quality,
                avg * 1000.,
                1. / avg,
                times[times.len() / 2] * 1000.,
                times.last().unwrap() * 1000.
            );
        }
    }
    Ok(())
}

/// Comparación reproducible de ambos motores con la misma escena y cámara.
/// Incluye las siete vistas y cada interruptor óptico; requiere un contexto GPU.
pub fn validate(scene: &Scene) -> Result<(), Box<dyn Error>> {
    let (mut window, thread) = raylib::init()
        .size(640, 400)
        .hidden()
        .title("Comparacion CPU GPU")
        .build();
    let mut renderer = Renderer::new(&mut window, &thread, scene)?;
    let mut failed = false;
    for index in 0..7 {
        for variant in 0..6 {
            let cfg = Settings {
                width: 400,
                height: 240,
                quality: if variant == 1 { 2 } else { 0 },
                reflections: variant != 2,
                refractions: variant != 3,
                skybox: variant != 4,
                space: variant == 5,
            };
            let camera = crate::viewer::preset(index);
            renderer.render(&mut window, &thread, camera, cfg)?;
            let gpu = renderer.pixels()?;
            let cpu = crate::render::render(scene, camera, cfg).pixels;
            let mut error = 0u64;
            let mut large = 0;
            for (a, b) in cpu.chunks_exact(4).zip(gpu.chunks_exact(4)) {
                let delta: [u8; 3] = std::array::from_fn(|i| a[i].abs_diff(b[i]));
                error += delta.iter().map(|&v| v as u64).sum::<u64>();
                if delta.iter().any(|&v| v > 32) {
                    large += 1;
                }
            }
            let mean = error as f64 / (cfg.width * cfg.height * 3) as f64;
            let outliers = large as f64 * 100. / (cfg.width * cfg.height) as f64;
            println!(
                "CPU/GPU vista {} variante {}: error medio {:.4}/255; pixeles >32: {:.3}%",
                index + 1,
                variant,
                mean,
                outliers
            );
            // Bordes compartidos y redondeo f32 pueden seleccionar caras distintas.
            failed |= mean > 1.0 || outliers > 0.5;
            if index == 3 && variant <= 1 {
                crate::png::save(
                    format!("/tmp/falcon-glass-gpu-q{}.png", cfg.quality),
                    cfg.width,
                    cfg.height,
                    &gpu,
                )?;
                crate::png::save(
                    format!("/tmp/falcon-glass-cpu-q{}.png", cfg.quality),
                    cfg.width,
                    cfg.height,
                    &cpu,
                )?;
            }
        }
    }
    if failed {
        Err("La comparación CPU/GPU excedió la tolerancia visual".into())
    } else {
        println!("Validación CPU/GPU: 42 comparaciones aprobadas");
        Ok(())
    }
}
