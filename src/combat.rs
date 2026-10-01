//! Combate del modo nave: un proyectil en vuelo y tres impactos por objetivo.
use crate::{
    camera::Camera,
    flight::Flight,
    math::{Ray, V},
};
pub const TRAVEL: f32 = 0.55;
pub const TARGETS: [V; 3] = [
    V::new(-8., 7., -18.),
    V::new(0., 10., -23.),
    V::new(8., 7., -18.),
];
pub const MUZZLE: V = V::new(0., 5.2, -0.6);
#[derive(Clone, Copy, Debug)]
pub struct Combat {
    pub enabled: bool,
    pub hp: [u8; 3],
    pub shot_age: f32,
    pub target: usize,
    pub impacts: [f32; 3],
}
impl Default for Combat {
    fn default() -> Self {
        Self {
            enabled: false,
            hp: [3; 3],
            shot_age: -1.,
            target: 0,
            impacts: [-1.; 3],
        }
    }
}
impl Combat {
    pub fn mask(self) -> u8 {
        if !self.enabled {
            return 0;
        }
        self.hp
            .iter()
            .enumerate()
            .fold(0, |mask, (i, &hp)| mask | (u8::from(hp > 0) << i))
    }
    pub fn fire(&mut self) -> bool {
        if !self.enabled || self.shot_age >= 0. {
            return false;
        }
        let Some(target) = self.hp.iter().position(|&hp| hp > 0) else {
            return false;
        };
        self.target = target;
        self.shot_age = 0.;
        true
    }
    pub fn update(&mut self, dt: f32) {
        if !self.enabled || !dt.is_finite() || dt < 0. {
            return;
        }
        for age in &mut self.impacts {
            if *age >= 0. {
                *age += dt;
                if *age > 1.6 {
                    *age = -1.;
                }
            }
        }
        if self.shot_age >= 0. {
            self.shot_age += dt;
            if self.shot_age >= TRAVEL {
                self.hp[self.target] = self.hp[self.target].saturating_sub(1);
                self.impacts[self.target] = (self.shot_age - TRAVEL).min(1.6);
                self.shot_age = -1.;
            }
        }
    }
    pub fn camera(self, flight: Flight) -> Camera {
        if !self.enabled {
            return flight.camera();
        }
        Camera {
            yaw: 165.,
            pitch: 20.,
            distance: 43.,
            target: V::new(0., 5., -7.) + flight.offset(),
            fov: 48.,
        }
    }
    pub fn segment(self) -> (V, V) {
        let direction = (TARGETS[self.target] - MUZZLE).unit();
        let length = (TARGETS[self.target] - MUZZLE).len();
        let head = (self.shot_age.max(0.) / TRAVEL).min(1.) * length;
        (
            MUZZLE + direction * (head - 3.).max(0.),
            MUZZLE + direction * head.max(0.001),
        )
    }
    pub fn glow(self, ray: Ray, limit: f32) -> V {
        if !self.enabled {
            return V::default();
        }
        let mut color = V::default();
        if self.shot_age >= 0. && self.shot_age < TRAVEL {
            let (a, b) = self.segment();
            color = color
                + V::new(2.8, 0.06, 0.015)
                    * crate::blaster::segment_glow(ray, limit, a, b, 0.012, 0.09);
        }
        for (i, &center) in TARGETS.iter().enumerate() {
            let age = self.impacts[i];
            let destroyed = self.hp[i] == 0;
            let duration = if destroyed { 1.6 } else { 0.35 };
            if age < 0. || age >= duration {
                continue;
            }
            let radius = if destroyed {
                0.6 + age * 3.5
            } else {
                0.4 + age * 1.5
            };
            let fade = 1. - age / duration;
            let along = (center - ray.o).dot(ray.d);
            if along > 0. && along - radius < limit {
                let delta = ray.at(along) - center;
                let r2 = delta.dot(delta) / (radius * radius);
                let body = (-r2 * 3.).exp() * fade;
                color = color
                    + V::new(3., 0.65, 0.035) * body
                    + V::new(2., 1.5, 0.5) * ((-r2 * 16.).exp() * fade * fade);
            }
            if destroyed {
                for j in 0..12 {
                    let angle = j as f32 * 2.399963;
                    let d = V::new(angle.cos(), (j as f32 * 1.7).sin(), angle.sin()).unit();
                    let a = center + d * (age * 6.);
                    color = color
                        + V::new(2., 0.32, 0.025)
                            * (crate::blaster::segment_glow(
                                ray,
                                limit,
                                a,
                                a + d * 0.6,
                                0.009,
                                0.04,
                            ) * fade);
                }
            }
        }
        color
    }
}
