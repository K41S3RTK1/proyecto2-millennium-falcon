use crate::math::{V, noise};
/// Cubemap de seis texturas generadas al inicio; el renderer lo muestrea al no golpear geometría.
pub struct Skybox {
    size: usize,
    faces: Vec<Vec<V>>,
}
impl Skybox {
    pub fn new(size: usize) -> Self {
        Self::generate(size, environment)
    }
    pub fn space(size: usize) -> Self {
        Self::generate(size, space_environment)
    }
    fn generate(size: usize, environment: fn(V) -> V) -> Self {
        let faces = (0..6)
            .map(|face| {
                (0..size * size)
                    .map(|i| {
                        let u = 2. * (i % size) as f32 / (size - 1) as f32 - 1.;
                        let v = 2. * (i / size) as f32 / (size - 1) as f32 - 1.;
                        environment(Self::direction(face, u, v))
                    })
                    .collect()
            })
            .collect();
        Self { size, faces }
    }
    pub fn direction(face: usize, u: f32, v: f32) -> V {
        match face {
            0 => V::new(1., v, -u),
            1 => V::new(-1., v, u),
            2 => V::new(u, 1., -v),
            3 => V::new(u, -1., v),
            4 => V::new(u, v, 1.),
            _ => V::new(-u, v, -1.),
        }
        .unit()
    }
    pub fn face_uv(d: V) -> (usize, f32, f32) {
        let (x, y, z) = (d.x.abs(), d.y.abs(), d.z.abs());
        if x >= y && x >= z {
            if d.x >= 0. {
                (0, -d.z / x, d.y / x)
            } else {
                (1, d.z / x, d.y / x)
            }
        } else if y >= z {
            if d.y >= 0. {
                (2, d.x / y, -d.z / y)
            } else {
                (3, d.x / y, d.z / y)
            }
        } else if d.z >= 0. {
            (4, d.x / z, d.y / z)
        } else {
            (5, -d.x / z, d.y / z)
        }
    }
    pub fn sample(&self, d: V) -> V {
        let (face, u, v) = Self::face_uv(d);
        let x = ((u + 1.) * 0.5 * (self.size - 1) as f32).clamp(0., (self.size - 1) as f32);
        let y = ((v + 1.) * 0.5 * (self.size - 1) as f32).clamp(0., (self.size - 1) as f32);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.size - 1), (y0 + 1).min(self.size - 1));
        let a = self.faces[face][y0 * self.size + x0]
            .mix(self.faces[face][y0 * self.size + x1], x - x0 as f32);
        let b = self.faces[face][y1 * self.size + x0]
            .mix(self.faces[face][y1 * self.size + x1], x - x0 as f32);
        a.mix(b, y - y0 as f32)
    }
    pub fn faces(&self) -> (&[Vec<V>], usize) {
        (&self.faces, self.size)
    }
}
fn environment(d: V) -> V {
    let h = d.y;
    let angle = d.x.atan2(d.z);
    let horizon = V::new(0.64, 0.40, 0.23);
    let zenith = V::new(0.08, 0.21, 0.34);
    let mut c = horizon.mix(zenith, (h.max(0.) * 1.8).clamp(0., 1.).powf(0.65));
    // Dunas lejanas pertenecen al entorno; el diorama tiene su propia base finita.
    let ridge = -0.14 + 0.017 * (angle * 7.).sin() + 0.012 * (angle * 13. + 1.).sin();
    if h < ridge {
        let dune = V::new(0.24, 0.115, 0.06);
        c = dune.mix(V::new(0.40, 0.22, 0.105), (-h * 2.).clamp(0., 1.));
    }
    for (dir, radius, tint) in [
        (
            V::new(-0.48, 0.018, 0.86).unit(),
            0.019,
            V::new(2.3, 1.8, 1.05),
        ),
        (
            V::new(-0.37, 0.007, 0.92).unit(),
            0.014,
            V::new(1.7, 0.7, 0.30),
        ),
    ] {
        let a = (d - dir).len();
        if a < radius {
            c = tint;
        } else if h > 0. {
            let glow = (1. - a / 0.22).max(0.).powi(5);
            c = c + tint * (glow * 0.10);
        }
    }
    // Leve grano fijo, sin aleatoriedad temporal.
    c * (0.995 + noise((angle * 500.).floor() as i32, (h * 500.).floor() as i32, 77) * 0.01)
}

/// Entorno espacial original: estrellas deterministas y estación parcialmente
/// destruida proyectada sobre las seis caras. No añade geometría al diorama.
fn space_environment(d: V) -> V {
    let haze = ((d.x * 4. + d.z * 3.).sin() * (d.y * 7. - d.z * 2.).cos())
        .abs()
        .powi(5);
    let mut color = V::new(0.0015, 0.003, 0.009) + V::new(0.003, 0.006, 0.016) * haze;
    let u = (d.z.atan2(d.x) / std::f32::consts::TAU + 0.5) * 1200.;
    let v = d.y.clamp(-1., 1.).acos() / std::f32::consts::PI * 600.;
    let seed = noise(u.floor() as i32, v.floor() as i32, 112);
    if seed > 0.995 {
        let du = u.fract() - 0.5;
        let dv = v.fract() - 0.5;
        let star = (1. - (du * du + dv * dv) / 0.18).max(0.).powi(2);
        color = color + V::new(0.65, 0.80, 1.) * (star * (0.5 + (seed - 0.995) * 350.));
    }
    let axis = V::new(-0.56, -0.27, 0.82).unit();
    let right = axis.cross(V::new(0., 1., 0.)).unit();
    let up = right.cross(axis);
    let z = d.dot(axis);
    if z <= 0. {
        return color;
    }
    let x = d.dot(right) / (z * 0.19);
    let y = d.dot(up) / (z * 0.19);
    let r2 = x * x + y * y;
    if r2 > 1. {
        return color + V::new(0.009, 0.017, 0.032) * (-(r2 - 1.) * 30.).exp();
    }
    let n = V::new(x, y, (1. - r2).sqrt());
    let grid = noise((x * 48.).floor() as i32, (y * 65.).floor() as i32, 47);
    let damaged = x > 0.18 + 0.12 * ((y * 29.).floor() * 2.3).sin() && y > 0.08;
    let framework = (x * 44.).rem_euclid(1.) < 0.13 || (y * 58.).rem_euclid(1.) < 0.12;
    if damaged && grid > 0.24 {
        if !framework || grid > 0.75 {
            return color;
        }
        return V::new(0.055, 0.10, 0.17) * (0.7 + grid);
    }
    let longitude = n.x.atan2(n.z);
    let latitude = n.y.asin();
    let panel = noise(
        (longitude * 40.).floor() as i32,
        (latitude * 65.).floor() as i32,
        71,
    );
    let seams = (longitude * 35.).rem_euclid(1.) < 0.10 || (latitude * 60.).rem_euclid(1.) < 0.12;
    let light = 0.12 + 0.88 * n.dot(V::new(-0.5, 0.7, 0.65).unit()).max(0.);
    let mut surface =
        V::new(0.25, 0.37, 0.53) * (light * (0.65 + panel * 0.50) * if seams { 0.5 } else { 1. });
    if (y + 0.065).abs() < 0.018 {
        surface = surface * 0.22;
    } else if (y + 0.065).abs() < 0.030 {
        surface = surface * 1.4;
    }
    let dish = ((x + 0.31).powi(2) + (y - 0.31).powi(2)).sqrt() / 0.25;
    if dish < 1. {
        surface = if dish > 0.88 {
            surface * 1.35
        } else {
            V::new(0.065, 0.12, 0.19) * (0.35 + 0.65 * dish)
        };
        if dish < 0.18 {
            surface = surface * 0.35;
        }
    }
    surface
}
