use crate::math::{Ray, V};
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub lo: V,
    pub hi: V,
}
impl Bounds {
    pub fn union(self, b: Self) -> Self {
        Self {
            lo: self.lo.min(b.lo),
            hi: self.hi.max(b.hi),
        }
    }
    pub fn interval(self, r: Ray, limit: f32) -> Option<(f32, f32)> {
        let mut near = f32::NEG_INFINITY;
        let mut far = limit;
        for a in 0..3 {
            let o = r.o.axis(a);
            let d = r.d.axis(a);
            let lo = self.lo.axis(a);
            let hi = self.hi.axis(a);
            if d.abs() < 1e-9 {
                if o < lo || o > hi {
                    return None;
                }
                continue;
            }
            let t0 = (lo - o) / d;
            let t1 = (hi - o) / d;
            near = near.max(t0.min(t1));
            far = far.min(t0.max(t1));
            if near > far {
                return None;
            }
        }
        (far > 0.0005).then_some((near, far))
    }
}
#[derive(Clone, Copy)]
pub struct Block {
    pub bounds: Bounds,
    pub material: usize,
    pub tint: V,
}
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub t: f32,
    pub point: V,
    pub normal: V,
    pub object: usize,
}
impl Block {
    pub fn new(center: V, size: V, material: usize, tint: V) -> Self {
        Self {
            bounds: Bounds {
                lo: center - size * 0.5,
                hi: center + size * 0.5,
            },
            material,
            tint,
        }
    }
    pub fn intersect(self, r: Ray, limit: f32, id: usize) -> Option<Hit> {
        // Se calcula la salida completa antes de aplicar el límite del impacto más cercano.
        let (near, far) = self.bounds.interval(r, f32::INFINITY)?;
        let t = if near > 0.0005 { near } else { far };
        if t >= limit {
            return None;
        }
        let p = r.at(t);
        let mut dist = f32::INFINITY;
        let mut normal = V::default();
        for a in 0..3 {
            for (edge, sign) in [
                (self.bounds.lo.axis(a), -1.0),
                (self.bounds.hi.axis(a), 1.0),
            ] {
                let delta = (p.axis(a) - edge).abs();
                if delta < dist {
                    dist = delta;
                    normal = match a {
                        0 => V::new(sign, 0., 0.),
                        1 => V::new(0., sign, 0.),
                        _ => V::new(0., 0., sign),
                    };
                }
            }
        }
        Some(Hit {
            t,
            point: p,
            normal,
            object: id,
        })
    }
}
// La inversa se calcula una vez por recorrido, no por cada caja de la jerarquía.
struct PreparedRay {
    ray: Ray,
    inverse: [f32; 3],
    parallel: [bool; 3],
}
impl PreparedRay {
    fn new(ray: Ray) -> Self {
        let mut inverse = [0.; 3];
        let mut parallel = [false; 3];
        for a in 0..3 {
            parallel[a] = ray.d.axis(a).abs() < 1e-9;
            if !parallel[a] {
                inverse[a] = 1. / ray.d.axis(a);
            }
        }
        Self {
            ray,
            inverse,
            parallel,
        }
    }
    #[inline]
    fn interval(&self, bounds: Bounds, limit: f32) -> Option<f32> {
        let mut near = f32::NEG_INFINITY;
        let mut far = limit;
        for a in 0..3 {
            let origin = self.ray.o.axis(a);
            if self.parallel[a] {
                if origin < bounds.lo.axis(a) || origin > bounds.hi.axis(a) {
                    return None;
                }
                continue;
            }
            let t0 = (bounds.lo.axis(a) - origin) * self.inverse[a];
            let t1 = (bounds.hi.axis(a) - origin) * self.inverse[a];
            // Márgenes conservadores en la jerarquía; el bloque verifica el impacto exacto.
            let error = 4. * f32::EPSILON * t0.abs().max(t1.abs()).max(1.);
            near = near.max(t0.min(t1) - error);
            far = far.min(t0.max(t1) + error);
            if near > far {
                return None;
            }
        }
        (far > 0.0005).then_some(near)
    }
}

struct Node {
    bounds: Bounds,
    kind: Kind,
}
enum Kind {
    Leaf { start: usize, count: usize },
    Branch(usize, usize),
}
pub struct Bvh {
    nodes: Vec<Node>,
    indices: Vec<usize>,
}
impl Bvh {
    /// Nodos preorden para GPU: límites, salto de subárbol, rango de hoja
    /// e índices de hijos. El shader recorre los hijos por distancia.
    /// Los índices de esta escena caben exactamente en f32.
    pub fn gpu_data(&self, blocks: &[Block]) -> (Vec<[f32; 4]>, Vec<Block>) {
        fn end(nodes: &[Node], id: usize) -> usize {
            match nodes[id].kind {
                Kind::Branch(_, right) => end(nodes, right),
                Kind::Leaf { .. } => id + 1,
            }
        }
        let mut data = Vec::with_capacity(self.nodes.len() * 3);
        for (id, node) in self.nodes.iter().enumerate() {
            let (start, count) = match node.kind {
                Kind::Leaf { start, count } => (start, count),
                Kind::Branch(_, _) => (0, 0),
            };
            data.push([
                node.bounds.lo.x,
                node.bounds.lo.y,
                node.bounds.lo.z,
                end(&self.nodes, id) as f32,
            ]);
            data.push([
                node.bounds.hi.x,
                node.bounds.hi.y,
                node.bounds.hi.z,
                count as f32,
            ]);
            let (left, right) = match node.kind {
                Kind::Branch(a, b) => (a, b),
                _ => (0, 0),
            };
            data.push([start as f32, left as f32, right as f32, 0.]);
        }
        (data, self.indices.iter().map(|&i| blocks[i]).collect())
    }
    pub fn build(blocks: &[Block]) -> Self {
        let mut tree = Self {
            nodes: Vec::new(),
            indices: Vec::with_capacity(blocks.len()),
        };
        if !blocks.is_empty() {
            tree.branch(blocks, (0..blocks.len()).collect(), 0);
        }
        tree
    }
    fn leaf(&mut self, node: usize, ids: Vec<usize>) {
        self.nodes[node].kind = Kind::Leaf {
            start: self.indices.len(),
            count: ids.len(),
        };
        self.indices.extend(ids);
    }
    fn branch(&mut self, blocks: &[Block], ids: Vec<usize>, depth: usize) -> usize {
        const BINS: usize = 16;
        let empty = Bounds {
            lo: V::splat(f32::INFINITY),
            hi: V::splat(f32::NEG_INFINITY),
        };
        let bounds = ids
            .iter()
            .fold(empty, |acc, &i| acc.union(blocks[i].bounds));
        let id = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            kind: Kind::Leaf { start: 0, count: 0 },
        });
        if ids.len() <= 4 || depth >= 60 {
            self.leaf(id, ids);
            return id;
        }
        // SAH: separar por superficie y cantidad evita cajas grandes solapadas,
        // especialmente entre el suelo, los edificios y los pequeños bloques de la nave.
        let center =
            |i: usize, a| (blocks[i].bounds.lo.axis(a) + blocks[i].bounds.hi.axis(a)) * 0.5;
        let area = |b: Bounds| {
            let d = (b.hi - b.lo).max(V::default());
            2. * (d.x * d.y + d.x * d.z + d.y * d.z)
        };
        let mut best_cost = area(bounds) * (ids.len() as f32 - 1.);
        let mut best = None;
        for axis in 0..3 {
            let lo = ids
                .iter()
                .map(|&i| center(i, axis))
                .fold(f32::INFINITY, f32::min);
            let hi = ids
                .iter()
                .map(|&i| center(i, axis))
                .fold(f32::NEG_INFINITY, f32::max);
            if hi - lo < 1e-6 {
                continue;
            }
            let scale = BINS as f32 / (hi - lo);
            let mut boxes = [empty; BINS];
            let mut counts = [0usize; BINS];
            for &i in &ids {
                let bin = (((center(i, axis) - lo) * scale) as usize).min(BINS - 1);
                boxes[bin] = boxes[bin].union(blocks[i].bounds);
                counts[bin] += 1;
            }
            let mut right_boxes = [empty; BINS];
            let mut right_counts = [0; BINS];
            let mut right = empty;
            let mut count = 0;
            for bin in (0..BINS).rev() {
                right = right.union(boxes[bin]);
                count += counts[bin];
                right_boxes[bin] = right;
                right_counts[bin] = count;
            }
            let mut left = empty;
            let mut count = 0;
            for bin in 0..BINS - 1 {
                left = left.union(boxes[bin]);
                count += counts[bin];
                if count == 0 || right_counts[bin + 1] == 0 {
                    continue;
                }
                let cost = area(left) * count as f32
                    + area(right_boxes[bin + 1]) * right_counts[bin + 1] as f32;
                if cost < best_cost {
                    best_cost = cost;
                    best = Some((axis, lo, scale, bin));
                }
            }
        }
        if let Some((axis, lo, scale, split)) = best {
            let (left, right): (Vec<_>, Vec<_>) = ids
                .into_iter()
                .partition(|&i| (((center(i, axis) - lo) * scale) as usize).min(BINS - 1) <= split);
            let left = self.branch(blocks, left, depth + 1);
            let right = self.branch(blocks, right, depth + 1);
            self.nodes[id].kind = Kind::Branch(left, right);
        } else {
            self.leaf(id, ids);
        }
        id
    }
    pub fn hit(&self, blocks: &[Block], r: Ray, limit: f32) -> Option<Hit> {
        if self.nodes.is_empty() {
            return None;
        }
        let prepared = PreparedRay::new(r);
        let root_near = prepared.interval(self.nodes[0].bounds, limit)?;
        let mut best = None;
        let mut max = limit;
        // La construcción limita la profundidad a 60; cada entrada guarda la
        // distancia ya calculada para no volver a intersectar la misma caja.
        let mut stack = [(0usize, 0f32); 64];
        stack[0] = (0, root_near);
        let mut count = 1;
        while count > 0 {
            count -= 1;
            let (id, near) = stack[count];
            if near > max {
                continue;
            }
            match &self.nodes[id].kind {
                Kind::Leaf { start, count } => {
                    for &i in &self.indices[*start..start + count] {
                        if let Some(hit) = blocks[i].intersect(r, max, i) {
                            max = hit.t;
                            best = Some(hit);
                        }
                    }
                }
                Kind::Branch(a, b) => {
                    let ta = prepared.interval(self.nodes[*a].bounds, max);
                    let tb = prepared.interval(self.nodes[*b].bounds, max);
                    match (ta, tb) {
                        (Some(aa), Some(bb)) => {
                            let (first, last) = if aa < bb {
                                ((*a, aa), (*b, bb))
                            } else {
                                ((*b, bb), (*a, aa))
                            };
                            stack[count] = last;
                            stack[count + 1] = first;
                            count += 2;
                        }
                        (Some(t), None) => {
                            stack[count] = (*a, t);
                            count += 1;
                        }
                        (None, Some(t)) => {
                            stack[count] = (*b, t);
                            count += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        best
    }
}
