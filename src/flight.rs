//! Estado visual del modo nave. Coordenadas locales y BVH inmutable durante el vuelo.
use crate::{
    camera::Camera,
    geometry::Bounds,
    math::{Ray, V, noise},
};

pub const SCALE: f32 = 1.65;
pub const TAKEOFF_SECONDS: f32 = 49.834_667;
pub const BOOST_SECONDS: f32 = 19.797_333;

#[derive(Clone, Copy, Debug)]
pub struct Flight {
    pub active: bool,
    pub progress: f32,
    pub boost_age: f32,
    pub boost_duration: f32,
}
impl Default for Flight {
    fn default() -> Self {
        Self {
            active: false,
            progress: 0.,
            boost_age: -1.,
            boost_duration: BOOST_SECONDS,
        }
    }
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
impl Flight {
    pub fn boost(self) -> f32 {
        if !self.active
            || self.progress < 1.
            || self.boost_age < 0.
            || self.boost_age >= self.boost_duration
        {
            return 0.;
        }
        smooth(self.boost_age / 0.55) * smooth((self.boost_duration - self.boost_age) / 1.5)
    }
    pub fn offset(self) -> V {
        if !self.active {
            return V::default();
        }
        let p = smooth(self.progress);
        V::new(0., 18. * p, -5. * p - 1.8 * self.boost())
    }
    pub fn camera(self) -> Camera {
        Camera {
            yaw: 128.,
            pitch: 18.,
            distance: 27.,
            target: V::new(0., 2.1 * SCALE, 0.) + self.offset()
                - V::new(0., 2. * self.progress * (1. - self.progress), 0.),
            fov: 48.,
        }
    }
    /// Integración de una lámina emisiva que nace en el arco posterior del motor.
    /// El límite es el primer impacto: la estela no atraviesa el casco.
    pub fn glow(self, ray: Ray, limit: f32) -> V {
        let power = self.boost();
        if power <= 0. {
            return V::default();
        }
        let length = (1. + 11. * power) * SCALE;
        let bounds = Bounds {
            lo: V::new(-3.35 * SCALE, 0.7 * SCALE, 2.5 * SCALE),
            hi: V::new(3.35 * SCALE, 2.5 * SCALE, 4.36 * SCALE + length),
        };
        let Some((start, end)) = bounds.interval(ray, limit) else {
            return V::default();
        };
        let start = start.max(0.);
        let step = (end - start) / 12.;
        if step <= 0. {
            return V::default();
        }
        let mut color = V::default();
        for i in 0..12 {
            let p = ray.at(start + (i as f32 + 0.5) * step);
            let x = p.x / SCALE;
            let nozzle = (4.25_f32.powi(2) - x * x).max(0.).sqrt() * SCALE + 0.10 * SCALE;
            let u = (p.z - nozzle) / length;
            if !(0.0..1.0).contains(&u) {
                continue;
            }
            let width = 3.35 * SCALE * (1. - 0.32 * u);
            let edge = (1. - (p.x / width).powi(4)).max(0.);
            let height = (0.12 + 0.43 * u) * SCALE;
            let dy = (p.y - 1.61 * SCALE) / height;
            let pulse = 0.83 + 0.17 * (p.z * 5. - self.boost_age * 18. + p.x * 2.).sin();
            let density = (-dy * dy).exp() * edge * (1. - u).powi(2) * pulse * power;
            color = color
                + V::new(0.08, 0.8, 2.8) * (density * step * 1.4)
                + V::new(0.7, 1.2, 1.5) * (density * (-dy * dy * 4.).exp() * step * 0.7);
        }
        color
    }
}

/// Cielo exclusivo del vuelo: atmósfera que se oscurece hasta un campo de estrellas.
pub fn sky(d: V, progress: f32) -> V {
    let p = smooth(progress);
    let h = ((d.y + 0.3 + p * 0.45) * 1.4).clamp(0., 1.);
    let atmosphere = V::new(0.58, 0.34, 0.17).mix(V::new(0.025, 0.15, 0.32), h);
    let u = (d.z.atan2(d.x) / std::f32::consts::TAU + 0.5) * 1100.;
    let v = d.y.clamp(-1., 1.).acos() / std::f32::consts::PI * 550.;
    let seed = noise(u.floor() as i32, v.floor() as i32, 173);
    let r = (u.fract() - 0.5).powi(2) + (v.fract() - 0.5).powi(2);
    let star = if seed > 0.995 {
        (1. - r / 0.22).max(0.).powi(2) * (0.8 + (seed - 0.995) * 240.)
    } else {
        0.
    };
    let space = V::new(0.002, 0.004, 0.012) + V::new(0.72, 0.86, 1.0) * star;
    atmosphere.mix(space, smooth((progress - 0.15) / 0.85))
}
