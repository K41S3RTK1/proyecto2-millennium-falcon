//! Proyectil analítico de duración acotada, compartido por CPU y GPU.
use crate::math::{Ray, V};
pub const DURATION: f32 = 0.68;
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub age: f32,
    pub soldier: usize,
    pub range: f32,
}
impl Default for Shot {
    fn default() -> Self {
        Self {
            age: -1.,
            soldier: 0,
            range: 6.,
        }
    }
}
impl Shot {
    pub fn active(self) -> bool {
        self.age >= 0. && self.age < DURATION
    }
    pub fn segments(self) -> (V, V, V, f32) {
        let muzzle = crate::scene::rebel_muzzle(self.soldier);
        let head = (self.age.max(0.) * 10. + 0.12).min(self.range);
        let tail = (head - 0.75).max(0.);
        let a = muzzle + V::new(tail, 0., 0.);
        let b = muzzle + V::new(head.max(tail + 0.001), 0., 0.);
        let flash = (1. - self.age / 0.12).clamp(0., 1.);
        (a, b, muzzle, flash)
    }
    pub fn glow(self, ray: Ray, limit: f32) -> V {
        if !self.active() {
            return V::default();
        }
        let (a, b, m, flash) = self.segments();
        let beam = segment_glow(ray, limit, a, b, 0.0018, 0.018);
        let flash_glow =
            segment_glow(ray, limit, m, m + V::new(0.06, 0., 0.), 0.005, 0.045) * flash;
        V::new(1., 0.028, 0.006) * (beam + flash_glow) * (1. - self.age / DURATION).min(0.8) * 5.
    }
}
fn segment_glow(ray: Ray, limit: f32, a: V, b: V, core: f32, halo: f32) -> f32 {
    let v = b - a;
    let w = ray.o - a;
    let dv = ray.d.dot(v);
    let vv = v.dot(v);
    let dw = ray.d.dot(w);
    let vw = v.dot(w);
    let t = if vv - dv * dv > 1e-6 {
        ((vw - dv * dw) / (vv - dv * dv)).clamp(0., 1.)
    } else {
        (vw / vv).clamp(0., 1.)
    };
    let point = a + v * t;
    let along = (point - ray.o).dot(ray.d);
    if along < 0. || along > limit {
        return 0.;
    }
    let delta = ray.at(along) - point;
    let d = delta.dot(delta);
    (-d / core).exp() * 2. + (-d / halo).exp() * 0.25
}
