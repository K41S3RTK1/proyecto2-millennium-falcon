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
struct Node {
    bounds: Bounds,
    kind: Kind,
}
enum Kind {
    Leaf(Vec<usize>),
    Branch(usize, usize),
}
pub struct Bvh {
    nodes: Vec<Node>,
}
impl Bvh {
    pub fn build(blocks: &[Block]) -> Self {
        let mut tree = Self { nodes: Vec::new() };
        if !blocks.is_empty() {
            tree.branch(blocks, (0..blocks.len()).collect());
        }
        tree
    }
    fn branch(&mut self, blocks: &[Block], mut ids: Vec<usize>) -> usize {
        let bounds = ids
            .iter()
            .skip(1)
            .fold(blocks[ids[0]].bounds, |acc, &i| acc.union(blocks[i].bounds));
        let id = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            kind: Kind::Leaf(Vec::new()),
        });
        if ids.len() <= 6 {
            self.nodes[id].kind = Kind::Leaf(ids);
            return id;
        }
        let span = bounds.hi - bounds.lo;
        let axis = if span.x >= span.y && span.x >= span.z {
            0
        } else if span.y >= span.z {
            1
        } else {
            2
        };
        ids.sort_unstable_by(|&a, &b| {
            let ac = blocks[a].bounds.lo + blocks[a].bounds.hi;
            let bc = blocks[b].bounds.lo + blocks[b].bounds.hi;
            ac.axis(axis).total_cmp(&bc.axis(axis))
        });
        let right = ids.split_off(ids.len() / 2);
        let left = self.branch(blocks, ids);
        let right = self.branch(blocks, right);
        self.nodes[id].kind = Kind::Branch(left, right);
        id
    }
    pub fn hit(&self, blocks: &[Block], r: Ray, limit: f32) -> Option<Hit> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best = None;
        let mut max = limit;
        let mut stack = [0usize; 128];
        let mut count = 1;
        while count > 0 {
            count -= 1;
            let id = stack[count];
            let node = &self.nodes[id];
            if node.bounds.interval(r, max).is_none() {
                continue;
            }
            match &node.kind {
                Kind::Leaf(ids) => {
                    for &i in ids {
                        if let Some(hit) = blocks[i].intersect(r, max, i) {
                            max = hit.t;
                            best = Some(hit);
                        }
                    }
                }
                Kind::Branch(a, b) => {
                    let ta = self.nodes[*a].bounds.interval(r, max);
                    let tb = self.nodes[*b].bounds.interval(r, max);
                    match (ta, tb) {
                        (Some(aa), Some(bb)) => {
                            let (first, last) = if aa.0 < bb.0 { (*a, *b) } else { (*b, *a) };
                            stack[count] = last;
                            stack[count + 1] = first;
                            count += 2;
                        }
                        (Some(_), None) => {
                            stack[count] = *a;
                            count += 1;
                        }
                        (None, Some(_)) => {
                            stack[count] = *b;
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
