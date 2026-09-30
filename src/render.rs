use crate::{
    camera::Camera,
    math::{Ray, V},
    scene::Scene,
};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Clone, Copy)]
pub struct Settings {
    pub width: usize,
    pub height: usize,
    pub quality: u8,
    pub reflections: bool,
    pub refractions: bool,
    pub skybox: bool,
    pub space: bool,
    pub shot: crate::blaster::Shot,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 1100,
            height: 720,
            quality: 2,
            reflections: true,
            refractions: true,
            skybox: true,
            space: false,
            shot: crate::blaster::Shot::default(),
        }
    }
}
pub struct Frame {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub elapsed: Duration,
}
fn shadow(scene: &Scene, origin: V, light: V) -> V {
    let delta = light - origin;
    let mut distance = delta.len();
    let mut ray = Ray::new(origin, delta);
    let mut visibility = V::splat(1.);
    for _ in 0..10 {
        let Some(hit) = scene.hit(ray, distance) else {
            return visibility;
        };
        let m = &scene.materials[scene.blocks[hit.object].material];
        if m.transparency <= 0. {
            return V::default();
        }
        visibility = visibility.hadamard(m.albedo.mix(V::splat(1.), 0.8)) * m.transparency.sqrt();
        distance -= hit.t + 0.002;
        ray.o = hit.point + ray.d * 0.002;
    }
    V::default()
}
/// Halo analítico alrededor del segmento emisor, recortado por el primer impacto.
/// Es un resplandor artístico; la iluminación roja usa rayos de sombra separados.
pub fn saber_glow(ray: Ray, limit: f32) -> V {
    let a = crate::scene::SABER_BOTTOM;
    let v = crate::scene::SABER_TOP - a;
    let w = ray.o - a;
    let b = ray.d.dot(v);
    let c = v.dot(v);
    let d = ray.d.dot(w);
    let e = v.dot(w);
    let t = if c - b * b > 1e-6 {
        ((e - b * d) / (c - b * b)).clamp(0., 1.)
    } else {
        (e / c).clamp(0., 1.)
    };
    let point = a + v * t;
    let along = (point - ray.o).dot(ray.d);
    if along < 0. || along > limit {
        return V::default();
    }
    let distance = (ray.at(along) - point).dot(ray.at(along) - point);
    let glow = (-distance / 0.009).exp() * 1.8 + (-distance / 0.045).exp() * 0.12;
    V::new(1., 0.003, 0.001) * glow
}
pub fn trace(scene: &Scene, ray: Ray, cfg: Settings, depth: u8, weight: f32) -> V {
    let sky = |d| {
        if cfg.skybox {
            if cfg.space {
                scene.space.sample(d)
            } else {
                scene.sky.sample(d)
            }
        } else {
            V::new(0.05, 0.06, 0.08)
        }
    };
    let Some(hit) = scene.hit(ray, f32::INFINITY) else {
        return sky(ray.d) + saber_glow(ray, f32::INFINITY) + cfg.shot.glow(ray, f32::INFINITY);
    };
    let block = scene.blocks[hit.object];
    let mat = &scene.materials[block.material];
    let front = ray.d.dot(hit.normal) < 0.;
    let n = if front { hit.normal } else { -hit.normal };
    let surface = mat.color(hit.point, hit.normal).hadamard(block.tint);
    let mut diffuse = if cfg.space {
        V::new(0.055, 0.07, 0.115)
    } else {
        V::new(0.095, 0.12, 0.17)
    };
    let mut specular = V::default();
    // Oclusión local de primer impacto para leer uniones y relieves.
    if depth == 0 && cfg.quality > 0 && mat.transparency == 0. {
        let tangent = n
            .cross(if n.y.abs() < 0.9 {
                V::new(0., 1., 0.)
            } else {
                V::new(1., 0., 0.)
            })
            .unit();
        let bitangent = n.cross(tangent);
        let mut occlusion = 0.;
        for d in [
            (n * 0.85 + tangent * 0.4 + bitangent * 0.3).unit(),
            (n * 0.85 - tangent * 0.4 - bitangent * 0.3).unit(),
        ] {
            if let Some(h) = scene.hit(Ray::new(hit.point + n * 0.002, d), 0.65) {
                occlusion += 1. - h.t / 0.65;
            }
        }
        diffuse = diffuse * (1. - occlusion * 0.28);
    }
    for (index, &light) in scene.lights.iter().enumerate() {
        let position = light.position;
        let color = if cfg.space && index < 2 {
            light.color.mix(V::new(0.48, 0.65, 1.), 0.65)
        } else {
            light.color
        };
        let intensity = light.intensity
            * light.attenuation(hit.point)
            * if cfg.space && index < 2 { 0.60 } else { 1. };
        if intensity < 0.001 {
            continue;
        }
        let l = (position - hit.point).unit();
        let ndotl = n.dot(l).max(0.);
        if ndotl <= 0. {
            continue;
        }
        let visibility = shadow(scene, hit.point + n * 0.002, position);
        diffuse = diffuse + color.hadamard(visibility) * (intensity * ndotl);
        let half = (l - ray.d).unit();
        let highlight = n.dot(half).max(0.).powf(mat.shininess) * mat.specular;
        specular = specular + color.hadamard(visibility) * (highlight * intensity);
    }
    let mut reflected = if cfg.reflections {
        mat.reflectivity
    } else {
        0.
    };
    let transparent = if cfg.refractions {
        mat.transparency
    } else {
        0.
    };
    let cos = (-ray.d.dot(n)).clamp(0., 1.);
    let r0 = ((1. - mat.ior) / (1. + mat.ior)).powi(2);
    let fresnel = r0 + (1. - r0) * (1. - cos).powi(5);
    reflected += if cfg.reflections {
        transparent * fresnel
    } else {
        0.
    };
    let mut transmitted = transparent * (1. - if cfg.reflections { fresnel } else { 0. });
    let eta = if front { 1. / mat.ior } else { mat.ior };
    let refracted = ray.d.refract(n, eta);
    if refracted.is_none() {
        reflected += transmitted;
        transmitted = 0.;
    }
    let local = (1. - reflected - transmitted).max(0.);
    let mut result = (surface.hadamard(diffuse) + specular) * local
        + mat.emission.hadamard(block.tint)
        + saber_glow(ray, hit.t)
        + cfg.shot.glow(ray, hit.t);
    let max_depth = if cfg.quality == 0 { 3 } else { 6 };
    if reflected > 0. {
        let d = ray.d.reflect(n);
        let color = if depth < max_depth && weight * reflected > 0.012 {
            trace(
                scene,
                Ray::new(hit.point + n * 0.002, d),
                cfg,
                depth + 1,
                weight * reflected,
            )
        } else {
            sky(d)
        };
        result = result + color * reflected;
    }
    if transmitted > 0.
        && let Some(d) = refracted
    {
        let color = if depth < max_depth && weight * transmitted > 0.012 {
            trace(
                scene,
                Ray::new(hit.point - n * 0.002, d),
                cfg,
                depth + 1,
                weight * transmitted,
            )
        } else {
            sky(d)
        };
        result = result + color.hadamard(surface.mix(V::splat(1.), 0.92)) * transmitted;
    }
    result
}
pub fn display(v: V) -> [u8; 3] {
    fn channel(c: f32) -> u8 {
        let c = c.max(0.) * 0.90;
        let mapped = (c * (2.51 * c + 0.03) / (c * (2.43 * c + 0.59) + 0.14)).clamp(0., 1.);
        (mapped.powf(1. / 2.2) * 255. + 0.5) as u8
    }
    [channel(v.x), channel(v.y), channel(v.z)]
}
pub fn render(scene: &Scene, camera: Camera, cfg: Settings) -> Frame {
    render_interruptible(scene, camera, cfg, &AtomicU64::new(0), 0).unwrap()
}

/// Cancelación por fila: un cambio de cámara interrumpe el refinamiento sin bloquear la ventana.
pub fn render_interruptible(
    scene: &Scene,
    camera: Camera,
    cfg: Settings,
    generation: &AtomicU64,
    expected: u64,
) -> Option<Frame> {
    let start = Instant::now();
    let mut pixels = vec![0u8; cfg.width * cfg.height * 4];
    let threads = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1)
        .saturating_sub(1)
        .clamp(1, 8);
    // Bandas pequeñas permiten que los núcleos rápidos del M1 tomen más trabajo.
    let rows = 8;
    let stride = cfg.width * 4;
    let eye = camera.position();
    let forward = (camera.target - eye).unit();
    let right = forward.cross(V::new(0., 1., 0.)).unit();
    let up = right.cross(forward);
    let scale = (camera.fov.to_radians() * 0.5).tan();
    let bands = Mutex::new(pixels.chunks_mut(rows * stride).enumerate());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            let bands = &bands;
            scope.spawn(move || {
                loop {
                    let Some((chunk_id, chunk)) = bands.lock().unwrap().next() else {
                        break;
                    };
                    for (i, px) in chunk.chunks_exact_mut(4).enumerate() {
                        if i % cfg.width == 0 && generation.load(Ordering::Relaxed) != expected {
                            return;
                        }
                        let x = i % cfg.width;
                        let y = chunk_id * rows + i / cfg.width;
                        let samples = if cfg.quality >= 2 { 4 } else { 1 };
                        let mut color = V::default();
                        for sample in 0..samples {
                            let (ox, oy) = if samples == 1 {
                                (0.5, 0.5)
                            } else {
                                (
                                    0.25 + (sample % 2) as f32 * 0.5,
                                    0.25 + (sample / 2) as f32 * 0.5,
                                )
                            };
                            let sx = (2. * (x as f32 + ox) / cfg.width as f32 - 1.)
                                * cfg.width as f32
                                / cfg.height as f32
                                * scale;
                            let sy = (1. - 2. * (y as f32 + oy) / cfg.height as f32) * scale;
                            color = color
                                + trace(
                                    scene,
                                    Ray::new(eye, forward + right * sx + up * sy),
                                    cfg,
                                    0,
                                    1.,
                                );
                        }
                        let rgb = display(color / samples as f32);
                        px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                    }
                }
            });
        }
    });
    if generation.load(Ordering::Relaxed) != expected {
        return None;
    }
    Some(Frame {
        pixels,
        width: cfg.width,
        height: cfg.height,
        elapsed: start.elapsed(),
    })
}
