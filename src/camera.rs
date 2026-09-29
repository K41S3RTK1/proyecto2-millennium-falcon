use crate::math::{Ray, V};
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: V,
    pub fov: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 38.,
            pitch: 22.,
            distance: 25.,
            target: V::new(0., 1.1, -0.4),
            fov: 48.,
        }
    }
}
impl Camera {
    pub fn position(self) -> V {
        let y = self.yaw.to_radians();
        let p = self.pitch.to_radians();
        self.target + V::new(y.sin() * p.cos(), p.sin(), -y.cos() * p.cos()) * self.distance
    }
    pub fn ray(self, x: f32, y: f32, w: usize, h: usize) -> Ray {
        let eye = self.position();
        let forward = (self.target - eye).unit();
        let right = forward.cross(V::new(0., 1., 0.)).unit();
        let up = right.cross(forward);
        let s = (self.fov.to_radians() * 0.5).tan();
        let sx = (2. * x / w as f32 - 1.) * w as f32 / h as f32 * s;
        let sy = (1. - 2. * y / h as f32) * s;
        Ray::new(eye, forward + right * sx + up * sy)
    }
}
