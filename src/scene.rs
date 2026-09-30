use crate::{
    geometry::{Block, Bvh, Hit},
    material::{self, ARMOR, CLOTH, DARK, ENGINE, GLASS, HULL, Material, PLASMA, SAND},
    math::{Ray, V, noise},
    skybox::Skybox,
};
pub const VADER_ORIGIN: V = V::new(1.1, 0., -7.05);
pub const SABER_BOTTOM: V = V::new(1.91, 1.20, -7.42);
pub const SABER_TOP: V = V::new(1.91, 2.96, -7.42);
#[derive(Clone, Copy)]
pub struct Light {
    pub position: V,
    pub color: V,
    pub intensity: f32,
    pub radius: f32,
}
impl Light {
    pub fn attenuation(self, p: V) -> f32 {
        if self.radius == 0. {
            1.
        } else {
            (1. - (self.position - p).len() / self.radius)
                .max(0.)
                .powi(2)
        }
    }
}
pub struct Scene {
    pub blocks: Vec<Block>,
    pub materials: Vec<Material>,
    pub bvh: Bvh,
    pub sky: Skybox,
    pub space: Skybox,
    pub lights: Vec<Light>,
}
impl Scene {
    pub fn new() -> Self {
        let mut b = Builder { blocks: Vec::new() };
        b.port();
        b.falcon();
        b.maintenance();
        b.vader();
        let bvh = Bvh::build(&b.blocks);
        Self {
            blocks: b.blocks,
            materials: material::materials(),
            bvh,
            sky: Skybox::new(192),
            space: Skybox::space(768),
            lights: vec![
                Light {
                    position: V::new(-9., 15., -10.),
                    color: V::new(1., 0.80, 0.59),
                    intensity: 1.05,
                    radius: 0.,
                },
                Light {
                    position: V::new(8., 9., -1.),
                    color: V::new(0.48, 0.70, 1.),
                    intensity: 0.32,
                    radius: 0.,
                },
                Light {
                    position: SABER_BOTTOM + V::new(0., 0.30, -0.09),
                    color: V::new(1., 0.012, 0.006),
                    intensity: 3.8,
                    radius: 3.0,
                },
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
                    V::new(0.62, 0.72, 0.86)
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
    fn vader(&mut self) {
        let o = VADER_ORIGIN;
        let white = V::splat(1.);
        // Capa de tiras cúbicas: hombros estrechos, faldón ancho y pliegues.
        for row in 0..13 {
            let y = 0.14 + row as f32 * 0.12;
            let half = if row < 4 {
                6
            } else if row < 9 {
                5
            } else {
                4
            };
            for x in -half..=half {
                let z = 0.20 + (13 - row) as f32 * 0.023 + if x % 2 == 0 { 0.035 } else { 0. };
                self.block(
                    o + V::new(x as f32 * 0.12, y, z),
                    V::new(0.12, 0.12, 0.10),
                    CLOTH,
                    V::splat(if x % 2 == 0 { 1.2 } else { 0.85 }),
                );
            }
        }
        for side in [-1., 1.] {
            self.block(
                o + V::new(side * 0.21, 0.12, -0.13),
                V::new(0.30, 0.32, 0.49),
                ARMOR,
                white,
            );
            self.block(
                o + V::new(side * 0.21, 0.49, 0.),
                V::new(0.24, 0.60, 0.29),
                CLOTH,
                white,
            );
            self.block(
                o + V::new(side * 0.21, 0.40, -0.17),
                V::new(0.18, 0.37, 0.07),
                ARMOR,
                white,
            );
            self.block(
                o + V::new(side * 0.42, 1.41, 0.),
                V::new(0.27, 0.26, 0.40),
                ARMOR,
                white,
            );
        }
        self.block(
            o + V::new(0., 1.06, 0.),
            V::new(0.62, 0.70, 0.36),
            CLOTH,
            V::splat(1.5),
        );
        self.block(
            o + V::new(0., 1.43, -0.055),
            V::new(0.65, 0.19, 0.38),
            ARMOR,
            white,
        );
        // Ribetes de la pechera y cinturón.
        for x in [-0.24, 0.24] {
            self.block(
                o + V::new(x, 1.41, -0.255),
                V::new(0.065, 0.16, 0.05),
                HULL,
                V::splat(0.50),
            );
        }
        self.block(
            o + V::new(0., 0.80, -0.02),
            V::new(0.69, 0.13, 0.41),
            ARMOR,
            white,
        );
        self.block(
            o + V::new(0., 0.80, -0.25),
            V::new(0.15, 0.105, 0.04),
            HULL,
            V::splat(0.6),
        );
        self.block(
            o + V::new(0., 1.17, -0.235),
            V::new(0.30, 0.30, 0.10),
            ARMOR,
            V::splat(0.45),
        );
        for (x, c) in [
            (-0.085, V::new(0.9, 0.025, 0.015)),
            (0., V::new(0.4, 0.9, 1.)),
            (0.085, V::new(0.2, 0.5, 1.)),
        ] {
            self.block(
                o + V::new(x, 1.23, -0.30),
                V::new(0.049, 0.066, 0.025),
                HULL,
                c * 2.,
            );
        }
        for x in [-0.085, 0., 0.085] {
            self.block(
                o + V::new(x, 1.11, -0.30),
                V::new(0.038, 0.07, 0.025),
                HULL,
                V::splat(0.7),
            );
        }
        // Brazo izquierdo bajo; derecho extendido sujetando la empuñadura.
        self.block(
            o + V::new(-0.49, 1.08, 0.),
            V::new(0.23, 0.47, 0.28),
            CLOTH,
            white,
        );
        self.block(
            o + V::new(-0.50, 0.83, -0.06),
            V::new(0.24, 0.25, 0.28),
            ARMOR,
            white,
        );
        self.block(
            o + V::new(0.54, 1.23, -0.04),
            V::new(0.20, 0.30, 0.29),
            CLOTH,
            white,
        );
        self.block(
            o + V::new(0.69, 1.08, -0.20),
            V::new(0.36, 0.21, 0.27),
            CLOTH,
            white,
        );
        self.block(
            o + V::new(0.81, 1.06, -0.34),
            V::new(0.23, 0.20, 0.23),
            ARMOR,
            white,
        );
        // Cúpula, faldón del casco, ojos hundidos y respirador triangular escalonado.
        for (y, w, d) in [
            (1.70, 0.73, 0.57),
            (1.82, 0.66, 0.57),
            (1.94, 0.62, 0.54),
            (2.06, 0.50, 0.45),
            (2.15, 0.31, 0.30),
        ] {
            self.block(o + V::new(0., y, 0.025), V::new(w, 0.12, d), ARMOR, white);
        }
        for side in [-1., 1.] {
            self.block(
                o + V::new(side * 0.35, 1.68, 0.045),
                V::new(0.14, 0.30, 0.56),
                ARMOR,
                white,
            );
            self.block(
                o + V::new(side * 0.155, 1.86, -0.277),
                V::new(0.23, 0.065, 0.035),
                ARMOR,
                V::splat(0.18),
            );
            self.block(
                o + V::new(side * 0.18, 1.77, -0.285),
                V::new(0.12, 0.08, 0.045),
                ARMOR,
                V::splat(1.5),
            );
        }
        self.block(
            o + V::new(0., 1.86, -0.31),
            V::new(0.065, 0.22, 0.075),
            ARMOR,
            V::splat(1.35),
        );
        for (y, w, z) in [
            (1.75, 0.12, -0.34),
            (1.68, 0.21, -0.36),
            (1.61, 0.30, -0.36),
        ] {
            self.block(
                o + V::new(0., y, z),
                V::new(w, 0.07, 0.08),
                ARMOR,
                V::splat(0.5),
            );
        }
        for x in [-0.075, 0., 0.075] {
            self.block(
                o + V::new(x, 1.64, -0.407),
                V::new(0.018, 0.13, 0.012),
                HULL,
                V::splat(0.28),
            );
        }
        for x in [-0.24, 0.24] {
            self.block(
                o + V::new(x, 1.60, -0.285),
                V::new(0.08, 0.08, 0.07),
                HULL,
                V::splat(0.4),
            );
        }
        // Empuñadura estriada y hoja de plasma en segmentos cúbicos.
        let h = SABER_BOTTOM - V::new(0., 0.18, 0.);
        self.block(h, V::new(0.13, 0.36, 0.13), HULL, V::splat(0.7));
        for i in 0..5 {
            self.block(
                h + V::new(0., -0.12 + i as f32 * 0.055, 0.),
                V::new(0.145, 0.025, 0.145),
                ARMOR,
                white,
            );
        }
        self.block(
            SABER_BOTTOM - V::new(0., 0.02, 0.),
            V::new(0.19, 0.09, 0.19),
            ARMOR,
            white,
        );
        for i in 0..22 {
            self.block(
                SABER_BOTTOM + V::new(0., 0.04 + i as f32 * 0.08, 0.),
                V::splat(0.08),
                PLASMA,
                white,
            );
        }
    }
    fn maintenance(&mut self) {
        // Astromecánico blanco y azul, construido con cubos sobre la plataforma.
        let origin = V::new(-4.65, 0., -5.45);
        let blue = V::new(0.08, 0.22, 0.90);
        let white = V::splat(1.);
        let step = 0.12;
        for y in 0..5 {
            for x in -2_i32..=2 {
                for z in -2_i32..=2 {
                    if x * x + z * z > 5 || (y > 0 && y < 4 && x.abs() < 2 && z.abs() < 2) {
                        continue;
                    }
                    let panel = z == -2 && ((y == 1 && x.abs() <= 1) || (y == 3 && x == 0));
                    self.block(
                        origin + V::new(x as f32 * step, 0.44 + y as f32 * step, z as f32 * step),
                        V::splat(step),
                        HULL,
                        if panel { blue } else { white },
                    );
                }
            }
        }
        for (y, radius_squared) in [(0, 5), (1, 2), (2, 1)] {
            for x in -2_i32..=2 {
                for z in -2_i32..=2 {
                    if x * x + z * z > radius_squared {
                        continue;
                    }
                    self.block(
                        origin + V::new(x as f32 * step, 1.04 + y as f32 * step, z as f32 * step),
                        V::splat(step),
                        HULL,
                        if y == 0 || x == 0 {
                            blue
                        } else {
                            V::splat(0.75)
                        },
                    );
                }
            }
        }
        // Patas laterales y tercer apoyo: silueta reconocible sin piezas curvas.
        for side in [-1., 1.] {
            self.block(
                origin + V::new(side * 0.37, 0.80, 0.),
                V::new(0.20, 0.24, 0.24),
                HULL,
                white,
            );
            self.block(
                origin + V::new(side * 0.46, 0.44, 0.),
                V::new(0.16, 0.56, 0.18),
                HULL,
                white,
            );
            self.block(
                origin + V::new(side * 0.46, 0.40, -0.105),
                V::new(0.07, 0.30, 0.035),
                HULL,
                blue,
            );
            self.block(
                origin + V::new(side * 0.46, 0.10, -0.08),
                V::new(0.30, 0.28, 0.46),
                HULL,
                white,
            );
            self.block(
                origin + V::new(side * 0.46, -0.025, -0.08),
                V::new(0.31, 0.07, 0.47),
                DARK,
                white,
            );
        }
        self.block(
            origin + V::new(0., 0.23, -0.12),
            V::new(0.12, 0.30, 0.14),
            DARK,
            white,
        );
        self.block(
            origin + V::new(0., 0.055, -0.20),
            V::new(0.24, 0.22, 0.38),
            HULL,
            white,
        );
        self.block(
            origin + V::new(0., 1.08, -0.31),
            V::new(0.16, 0.11, 0.055),
            DARK,
            white,
        );
        self.block(
            origin + V::new(0.18, 1.04, -0.30),
            V::new(0.055, 0.055, 0.055),
            ENGINE,
            V::splat(0.7),
        );
        for x in [-0.12, 0.12] {
            self.block(
                origin + V::new(x, 0.44, -0.315),
                V::new(0.055, 0.14, 0.04),
                DARK,
                white,
            );
        }
        // Consola de mantenimiento y cable bajo, con los mismos materiales de la escena.
        let station = V::new(-6.15, 0., -4.9);
        self.block(
            station + V::new(0., 0.08, 0.),
            V::new(0.64, 0.28, 0.62),
            DARK,
            white,
        );
        self.block(
            station + V::new(0., 0.52, 0.),
            V::new(0.44, 0.66, 0.36),
            HULL,
            V::splat(0.55),
        );
        self.block(
            station + V::new(0., 0.93, 0.),
            V::new(0.72, 0.30, 0.22),
            DARK,
            white,
        );
        self.block(
            station + V::new(0., 0.95, -0.122),
            V::new(0.51, 0.16, 0.035),
            ENGINE,
            V::splat(0.6),
        );
        for x in [-0.18, 0., 0.18] {
            self.block(
                station + V::new(x, 0.78, -0.13),
                V::new(0.07, 0.07, 0.06),
                HULL,
                blue,
            );
        }
        for i in 0..9 {
            self.cube(V::new(-5.95 + i as f32 * 0.10, -0.015, -5.10), 0.09, DARK);
        }
        for i in 0..4 {
            self.cube(V::new(-5.15, -0.015, -5.19 - i as f32 * 0.09), 0.09, DARK);
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
