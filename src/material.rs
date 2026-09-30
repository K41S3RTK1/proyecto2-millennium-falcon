use crate::math::{V, noise};
pub const HULL: usize = 0;
pub const DARK: usize = 1;
pub const GLASS: usize = 2;
pub const SAND: usize = 3;
pub const ENGINE: usize = 4;
#[derive(Clone, Copy)]
pub enum Texture {
    Hull,
    Dark,
    Glass,
    Sand,
    Engine,
}
pub struct Material {
    pub name: &'static str,
    pub texture: Texture,
    pub albedo: V,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
    pub emission: V,
}
pub fn materials() -> Vec<Material> {
    vec![
        Material {
            name: "Aleación del casco",
            texture: Texture::Hull,
            albedo: V::new(0.58, 0.61, 0.64),
            specular: 0.38,
            shininess: 55.,
            transparency: 0.,
            reflectivity: 0.10,
            ior: 1.,
            emission: V::default(),
        },
        Material {
            name: "Metal oscuro pulido",
            texture: Texture::Dark,
            albedo: V::new(0.105, 0.14, 0.17),
            specular: 0.8,
            shininess: 120.,
            transparency: 0.,
            reflectivity: 0.42,
            ior: 1.,
            emission: V::default(),
        },
        Material {
            name: "Vidrio de la cabina",
            texture: Texture::Glass,
            albedo: V::new(0.63, 0.83, 0.90),
            specular: 0.95,
            shininess: 180.,
            transparency: 0.86,
            reflectivity: 0.08,
            ior: 1.5,
            emission: V::default(),
        },
        Material {
            name: "Arenisca del puerto",
            texture: Texture::Sand,
            albedo: V::new(0.61, 0.39, 0.20),
            specular: 0.06,
            shininess: 12.,
            transparency: 0.,
            reflectivity: 0.0,
            ior: 1.,
            emission: V::default(),
        },
        Material {
            name: "Paneles del motor",
            texture: Texture::Engine,
            albedo: V::new(0.12, 0.66, 0.95),
            specular: 0.65,
            shininess: 85.,
            transparency: 0.,
            reflectivity: 0.16,
            ior: 1.,
            emission: V::new(0.035, 0.55, 1.8),
        },
    ]
}
impl Material {
    pub fn color(&self, p: V, n: V) -> V {
        let (u, v) = if n.y.abs() > 0.5 {
            (p.x, p.z)
        } else if n.x.abs() > 0.5 {
            (p.z, p.y)
        } else {
            (p.x, p.y)
        };
        let grain = noise((u * 48.).floor() as i32, (v * 48.).floor() as i32, 31);
        match self.texture {
            Texture::Hull => {
                let seam = u.rem_euclid(0.72) < 0.014 || v.rem_euclid(0.72) < 0.014;
                let panel = noise((u / 0.72).floor() as i32, (v / 0.72).floor() as i32, 7);
                let bolt = u.rem_euclid(0.72) < 0.055 && v.rem_euclid(0.72) < 0.055;
                self.albedo
                    * (if seam {
                        0.34
                    } else if bolt {
                        0.25
                    } else {
                        0.72 + panel * 0.30 + grain * 0.06
                    })
            }
            Texture::Dark => {
                self.albedo
                    * (0.7
                        + grain * 0.3
                        + if (v * 35.).fract().abs() < 0.2 {
                            0.2
                        } else {
                            0.
                        })
            }
            Texture::Glass => self.albedo * (0.97 + 0.03 * grain),
            Texture::Sand => {
                let block = noise(
                    (p.x * 2.).floor() as i32,
                    (p.z * 2.).floor() as i32,
                    (p.y * 2.).floor() as i32,
                );
                self.albedo * (0.8 + 0.20 * block + grain * 0.16)
            }
            Texture::Engine => {
                self.albedo
                    * (if u.rem_euclid(0.18) < 0.025 {
                        0.35
                    } else {
                        0.85 + grain * 0.15
                    })
            }
        }
    }
}
