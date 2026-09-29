use crate::{
    geometry::{Block, Bvh, Hit},
    material::{self, DARK, ENGINE, GLASS, HULL, Material, SAND},
    math::{Ray, V, noise},
    skybox::Skybox,
};
pub struct Scene {
    pub blocks: Vec<Block>,
    pub materials: Vec<Material>,
    pub bvh: Bvh,
    pub sky: Skybox,
    pub lights: Vec<(V, V, f32)>,
}
impl Scene {
    pub fn new() -> Self {
        let mut b = Builder { blocks: Vec::new() };
        b.port();
        b.falcon();
        let bvh = Bvh::build(&b.blocks);
        Self {
            blocks: b.blocks,
            materials: material::materials(),
            bvh,
            sky: Skybox::new(192),
            lights: vec![
                (V::new(-9., 15., -10.), V::new(1.0, 0.82, 0.62), 1.3),
                (V::new(8., 9., -1.), V::new(0.60, 0.80, 1.), 0.40),
            ],
        }
    }
    pub fn hit(&self, r: Ray, max: f32) -> Option<Hit> {
        self.bvh.hit(&self.blocks, r, max)
    }
}
impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
struct Builder {
    blocks: Vec<Block>,
}
impl Builder {
    fn block(&mut self, p: V, size: V, m: usize, tint: V) {
        self.blocks.push(Block::new(p, size, m, tint));
    }
    fn cube(&mut self, p: V, s: f32, m: usize) {
        self.block(p, V::splat(s), m, V::splat(1.));
    }
    fn port(&mut self) {
        // Base finita, construida por columnas cúbicas. No hay plano infinito.
        let s = 0.6;
        for x in -13_i32..=13 {
            for z in -14_i32..=14 {
                if x.abs() + z.abs() > 23 {
                    continue;
                }
                let px = x as f32 * s;
                let pz = z as f32 * s;
                let tint = if (px * px + pz * pz).sqrt() < 6.6 {
                    V::new(0.85, 0.91, 1.0)
                } else {
                    V::splat(1.)
                };
                self.block(V::new(px, -0.36, pz), V::new(s, 0.60, s), SAND, tint);
                if x.abs() == 13 || z.abs() == 14 || x.abs() + z.abs() == 23 {
                    self.block(
                        V::new(px, -0.81, pz),
                        V::new(s, 0.3, s),
                        SAND,
                        V::splat(0.62),
                    );
                }
            }
        }
        // Marcas discretas de la plataforma, hechas con bloques muy planos.
        for i in 0..72 {
            let a = i as f32 * std::f32::consts::TAU / 72.;
            let p = V::new(a.cos() * 6.3, -0.045, a.sin() * 6.3);
            self.block(p, V::new(0.23, 0.025, 0.23), HULL, V::new(1., 0.73, 0.29));
        }
        for x in [-3.8, 3.8] {
            for z in [-5.7, -4.9, -4.1] {
                self.block(
                    V::new(x, -0.035, z),
                    V::new(0.7, 0.04, 0.18),
                    HULL,
                    V::new(1.2, 1.1, 0.8),
                );
            }
        }
        // Edificios de arenisca: volúmenes escalonados y entradas hundidas.
        for (cx, cz, scale) in [(-5.5, 4.8, 1.0), (4.7, 6.0, 0.75)] {
            let step = 0.4;
            for y in 0..8 {
                for x in -3_i32..=3 {
                    for z in -3_i32..=3 {
                        let rad = if y < 5 { 3.8 } else { (9 - y) as f32 * 0.8 };
                        if ((x * x + z * z) as f32).sqrt() > rad {
                            continue;
                        }
                        if z <= -2 && x.abs() <= 1 && y < 4 {
                            continue;
                        }
                        self.cube(
                            V::new(
                                cx + x as f32 * step * scale,
                                (y as f32 + 0.5) * step * scale,
                                cz + z as f32 * step * scale,
                            ),
                            step * scale,
                            SAND,
                        );
                    }
                }
            }
            self.block(
                V::new(cx, 0.7 * scale, cz - 0.5 * scale),
                V::new(0.9 * scale, 1.4 * scale, 0.10),
                DARK,
                V::splat(0.5),
            );
            self.block(
                V::new(cx + 0.6, 3.7 * scale, cz),
                V::new(0.08, 1.2 * scale, 0.08),
                DARK,
                V::splat(1.),
            );
            self.cube(V::new(cx + 0.6, 4.3 * scale, cz), 0.14, ENGINE);
        }
        // Bajo muro posterior con contrafuertes y luces.
        for i in -7_i32..=7 {
            if i.abs() < 3 {
                continue;
            }
            self.block(
                V::new(i as f32 * 0.6, 0.35, 7.3),
                V::new(0.6, 0.8, 0.5),
                SAND,
                V::splat(0.95),
            );
        }
        for x in [-6.8, 6.8] {
            for z in [-3.8, 1.8, 6.8] {
                self.block(
                    V::new(x, 0.32, z),
                    V::new(0.25, 0.7, 0.25),
                    DARK,
                    V::splat(1.),
                );
                self.block(
                    V::new(x, 0.7, z),
                    V::new(0.3, 0.14, 0.3),
                    ENGINE,
                    V::new(1.8, 0.9, 0.4),
                );
            }
        }
        for (x, z, h) in [
            (-5.4, -3.5, 0.65),
            (-6.1, -3., 0.45),
            (-5.3, -2.65, 0.5),
            (5.8, 3.8, 0.6),
        ] {
            self.block(
                V::new(x, h / 2., z),
                V::splat(h),
                HULL,
                V::new(0.55, 0.47, 0.34),
            );
            self.block(
                V::new(x, h * 0.7, z - h * 0.51),
                V::new(h * 0.65, 0.08, 0.03),
                DARK,
                V::splat(1.),
            );
        }
        // Depósito de cristal: objetos interiores hacen visible la refracción al orbitar.
        let tank = V::new(5.5, 0.9, -4.5);
        self.block(
            tank + V::new(0., -0.68, 0.),
            V::new(1.1, 0.35, 0.9),
            DARK,
            V::splat(1.),
        );
        self.block(tank, V::new(0.94, 1.1, 0.74), GLASS, V::splat(1.));
        self.block(
            tank + V::new(0., 0.65, 0.),
            V::new(1.1, 0.18, 0.9),
            DARK,
            V::splat(1.),
        );
        for x in [-0.25, 0., 0.25] {
            self.block(
                tank + V::new(x, 0., 0.),
                V::new(0.10, 0.7, 0.15),
                ENGINE,
                V::new(1.4, 0.9, 0.4),
            );
        }
    }
    fn falcon(&mut self) {
        let s = 0.22;
        // Casco circular en capas. Cubos de superficie conservan la silueta voxel.
        for x in -20_i32..=20 {
            for z in -20_i32..=20 {
                let px = x as f32 * s;
                let pz = z as f32 * s;
                let r = (px * px + pz * pz).sqrt();
                if r > 4.25 {
                    continue;
                }
                let top = 2.05 + ((1. - r / 4.25) * 3.).floor() * s;
                let shade = 0.94 + noise(x, z, 5) * 0.10;
                let stripe = (x == -9 || x == -8) && z < 9 && z > -6;
                let tint = if stripe {
                    V::new(0.62, 0.23, 0.16)
                } else {
                    V::splat(shade)
                };
                self.block(V::new(px, top, pz), V::splat(s), HULL, tint);
                self.cube(V::new(px, 1.17, pz), s, HULL);
                if r > 3.98 {
                    for y in [1.39, 1.61, 1.83] {
                        let engine = pz > 2.5 && y == 1.61;
                        self.block(
                            V::new(px, y, pz),
                            V::splat(s),
                            if engine {
                                ENGINE
                            } else if y == 1.61 {
                                DARK
                            } else {
                                HULL
                            },
                            V::splat(1.),
                        );
                    }
                }
            }
        }
        // Dos mandíbulas delanteras y canal central abierto.
        for iz in 0..18 {
            let z = -3.0 - iz as f32 * s;
            let width = 2.65 - iz as f32 * 0.05;
            for ix in -13_i32..=13 {
                let x = ix as f32 * s;
                if x.abs() < 0.62 || x.abs() > width {
                    continue;
                }
                for y in [1.17, 1.39, 1.61, 1.83, 2.05] {
                    if y > 1.17 && y < 2.05 && x.abs() > 0.86 && x.abs() < width - 0.24 && iz < 17 {
                        continue;
                    }
                    let tint = if ix.abs() == 6 && iz > 10 {
                        V::new(0.62, 0.25, 0.18)
                    } else {
                        V::splat(1.)
                    };
                    self.block(
                        V::new(x, y, z),
                        V::splat(s),
                        if y == 1.61 { DARK } else { HULL },
                        tint,
                    );
                }
            }
        }
        // Tren de aterrizaje: soportes y zapatas.
        for (x, z) in [(-2.3, 1.6), (2.3, 1.6), (-1.6, -3.4), (1.6, -3.4), (0., 0.)] {
            self.block(
                V::new(x, 0.55, z),
                V::new(0.22, 1.1, 0.24),
                DARK,
                V::splat(1.),
            );
            self.block(
                V::new(x, 0.14, z),
                V::new(0.7, 0.24, 0.65),
                HULL,
                V::splat(0.85),
            );
        }
        // Seis ventiladores en la cubierta posterior, discretizados en cubos.
        for (cx, cz) in [
            (-2.1, 2.0),
            (0., 2.65),
            (2.1, 2.0),
            (-1.1, 1.0),
            (1.1, 1.0),
            (0., -0.15),
        ] {
            for x in -3_i32..=3 {
                for z in -3_i32..=3 {
                    let r = ((x * x + z * z) as f32).sqrt();
                    if r > 3.1 {
                        continue;
                    }
                    let px = cx + x as f32 * 0.16;
                    let pz = cz + z as f32 * 0.16;
                    let hull_top =
                        2.05 + ((1. - (px * px + pz * pz).sqrt() / 4.25) * 3.).floor() * s;
                    let p = V::new(px, hull_top + 0.15, pz);
                    self.block(
                        p,
                        V::new(0.16, 0.09, 0.16),
                        if r > 2.1 { HULL } else { DARK },
                        V::splat(if r > 2.1 { 0.85 } else { 0.7 }),
                    );
                    if r < 2.0 && x % 2 == 0 {
                        self.block(
                            p + V::new(0., 0.055, 0.),
                            V::new(0.045, 0.025, 0.16),
                            HULL,
                            V::splat(0.55),
                        );
                    }
                }
            }
        }
        // Relieves de paneles y conductos.
        for sign in [-1., 1.] {
            for i in 0..7 {
                self.block(
                    V::new(sign * (2.2 + i as f32 * 0.17), 2.27, -1.2),
                    V::new(0.11, 0.10, 0.85),
                    DARK,
                    V::splat(0.9),
                );
            }
            self.block(
                V::new(sign * 1.3, 2.22, -4.9),
                V::new(0.75, 0.08, 1.1),
                HULL,
                V::splat(0.78),
            );
        }
        // Torre central y cuatro cañones.
        self.block(
            V::new(0., 2.72, -0.9),
            V::new(0.92, 0.66, 0.85),
            HULL,
            V::splat(1.),
        );
        for x in [-0.23, 0.23] {
            for y in [3.0, 3.18] {
                self.block(
                    V::new(x, y, -1.7),
                    V::new(0.09, 0.09, 1.1),
                    DARK,
                    V::splat(1.),
                );
            }
        }
        // Antena: plato circular de bloques con soporte.
        self.block(
            V::new(-1.75, 2.94, -0.55),
            V::new(0.22, 1.05, 0.22),
            DARK,
            V::splat(1.),
        );
        for x in -4_i32..=4 {
            for y in -4_i32..=4 {
                if x * x + y * y > 18 {
                    continue;
                }
                self.block(
                    V::new(-1.75 + x as f32 * 0.15, 3.6 + y as f32 * 0.15, -0.6),
                    V::new(0.15, 0.15, 0.16),
                    HULL,
                    V::splat(0.95),
                );
            }
        }
        self.block(
            V::new(-1.75, 3.6, -0.85),
            V::new(0.10, 0.10, 0.55),
            DARK,
            V::splat(1.),
        );
        // Pasillo lateral diagonal hasta la cabina asimétrica.
        for i in 0..10 {
            let t = i as f32 / 9.;
            let x = 3.2 + t * 1.3;
            let z = -0.8 - t * 2.;
            self.block(
                V::new(x, 1.97, z),
                V::new(0.8, 0.72, 0.5),
                HULL,
                V::splat(0.92),
            );
        }
        // Cabina hueca y ventanal con volumen: dos interfaces aire/vidrio reales.
        self.block(
            V::new(4.5, 1.64, -3.5),
            V::new(1.1, 0.20, 1.7),
            HULL,
            V::splat(1.),
        );
        self.block(
            V::new(4.5, 2.40, -3.36),
            V::new(1.1, 0.16, 1.4),
            HULL,
            V::splat(1.),
        );
        self.block(
            V::new(4.5, 2.03, -4.19),
            V::new(1.0, 0.62, 0.16),
            GLASS,
            V::splat(1.),
        );
        for x in [3.99, 5.01] {
            self.block(
                V::new(x, 2.04, -3.5),
                V::new(0.12, 0.64, 1.45),
                GLASS,
                V::splat(1.),
            );
        }
        for x in [4.0, 4.5, 5.0] {
            self.block(
                V::new(x, 2.02, -4.30),
                V::new(0.06, 0.72, 0.07),
                DARK,
                V::splat(1.),
            );
        }
        for z in [-4.1, -3.5, -2.9] {
            for x in [3.92, 5.08] {
                self.block(
                    V::new(x, 2.02, z),
                    V::new(0.06, 0.8, 0.06),
                    DARK,
                    V::splat(1.),
                );
            }
        }
        self.block(
            V::new(4.5, 1.89, -3.83),
            V::new(0.78, 0.10, 0.30),
            ENGINE,
            V::splat(0.6),
        );
        for x in [4.25, 4.75] {
            self.block(
                V::new(x, 1.96, -3.15),
                V::new(0.25, 0.42, 0.30),
                DARK,
                V::splat(0.6),
            );
        }
        // Rampa lateral escalonada.
        for i in 0..7 {
            let t = i as f32;
            self.block(
                V::new(4.1 + t * 0.22, 1.25 - t * 0.17, 0.4),
                V::new(0.26, 0.15, 0.85),
                DARK,
                V::splat(1.),
            );
        }
    }
}
