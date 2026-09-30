use falcon_diorama::{
    camera::Camera,
    geometry::Bounds,
    math::{Ray, V},
    scene::Scene,
};

#[test]
fn gpu_threaded_layout_preserves_all_blocks_and_nearest_hits() {
    let scene = Scene::new();
    let (nodes, blocks) = scene.bvh.gpu_data(&scene.blocks);
    assert_eq!(blocks.len(), scene.blocks.len());
    let count = nodes.len() / 3;
    let mut visited = vec![false; blocks.len()];
    for id in 0..count {
        let skip = nodes[id * 3][3] as usize;
        assert!(skip > id && skip <= count);
        if nodes[id * 3 + 1][3] == 0. {
            let left = nodes[id * 3 + 2][1] as usize;
            let right = nodes[id * 3 + 2][2] as usize;
            assert_eq!(left, id + 1);
            assert_eq!(right, nodes[left * 3][3] as usize);
            assert_eq!(skip, nodes[right * 3][3] as usize);
        }
        let start = nodes[id * 3 + 2][0] as usize;
        let size = nodes[id * 3 + 1][3] as usize;
        for item in &mut visited[start..start + size] {
            assert!(!*item);
            *item = true;
        }
    }
    assert!(visited.iter().all(|v| *v));
    let hit = |ray: Ray, limit: f32| {
        let mut id = 0;
        let mut best = limit;
        let mut found = false;
        while id < count {
            let lo = nodes[id * 3];
            let hi = nodes[id * 3 + 1];
            let bounds = Bounds {
                lo: V::new(lo[0], lo[1], lo[2]),
                hi: V::new(hi[0], hi[1], hi[2]),
            };
            if bounds.interval(ray, best).is_none() {
                id = lo[3] as usize;
                continue;
            }
            let size = hi[3] as usize;
            if size == 0 {
                id += 1;
                continue;
            }
            let start = nodes[id * 3 + 2][0] as usize;
            for b in &blocks[start..start + size] {
                if let Some(h) = b.intersect(ray, best, 0) {
                    best = h.t;
                    found = true;
                }
            }
            id = lo[3] as usize;
        }
        found.then_some(best)
    };
    for yaw in [0., 38., 90., 180., 270.] {
        let camera = Camera {
            yaw,
            ..Camera::default()
        };
        for y in 0..30 {
            for x in 0..50 {
                let ray = camera.ray(x as f32 + 0.5, y as f32 + 0.5, 50, 30);
                let expected = scene.hit(ray, 100.).map(|h| h.t);
                let actual = hit(ray, 100.);
                assert_eq!(actual.is_some(), expected.is_some());
                if let (Some(a), Some(b)) = (actual, expected) {
                    assert!((a - b).abs() < 1e-4);
                }
            }
        }
    }
}
