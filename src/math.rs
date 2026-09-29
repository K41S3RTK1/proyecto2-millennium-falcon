use std::ops::{Add, Div, Mul, Neg, Sub};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl V {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn splat(n: f32) -> Self {
        Self::new(n, n, n)
    }
    pub fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn unit(self) -> Self {
        self / self.len().max(1e-12)
    }
    pub fn axis(self, i: usize) -> f32 {
        [self.x, self.y, self.z][i]
    }
    pub fn min(self, b: Self) -> Self {
        Self::new(self.x.min(b.x), self.y.min(b.y), self.z.min(b.z))
    }
    pub fn max(self, b: Self) -> Self {
        Self::new(self.x.max(b.x), self.y.max(b.y), self.z.max(b.z))
    }
    pub fn hadamard(self, b: Self) -> Self {
        Self::new(self.x * b.x, self.y * b.y, self.z * b.z)
    }
    pub fn reflect(self, n: Self) -> Self {
        self - n * (2.0 * self.dot(n))
    }
    /// Snell; n apunta contra el rayo, eta = índice de entrada / índice de salida.
    pub fn refract(self, n: Self, eta: f32) -> Option<Self> {
        let cos = (-self.dot(n)).clamp(0.0, 1.0);
        let k = 1.0 - eta * eta * (1.0 - cos * cos);
        (k >= 0.0).then(|| (self * eta + n * (eta * cos - k.sqrt())).unit())
    }
    pub fn mix(self, b: Self, t: f32) -> Self {
        self * (1.0 - t) + b * t
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        Self::new(self.x * b, self.y * b, self.z * b)
    }
}
impl Div<f32> for V {
    type Output = Self;
    fn div(self, b: f32) -> Self {
        Self::new(self.x / b, self.y / b, self.z / b)
    }
}
impl Neg for V {
    type Output = Self;
    fn neg(self) -> Self {
        self * -1.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub o: V,
    pub d: V,
}
impl Ray {
    pub fn new(o: V, d: V) -> Self {
        Self { o, d: d.unit() }
    }
    pub fn at(self, t: f32) -> V {
        self.o + self.d * t
    }
}
pub fn hash(mut n: u32) -> u32 {
    n = n.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((n >> ((n >> 28) + 4)) ^ n).wrapping_mul(277803737);
    (word >> 22) ^ word
}
pub fn noise(x: i32, y: i32, z: i32) -> f32 {
    hash(
        (x as u32).wrapping_mul(73856093)
            ^ (y as u32).wrapping_mul(19349663)
            ^ (z as u32).wrapping_mul(83492791),
    ) as f32
        / u32::MAX as f32
}
