use falcon_diorama::{
    camera::Camera,
    geometry::{Block, Bvh},
    material,
    math::{Ray, V, noise},
    skybox::Skybox,
};
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-4, "{a} != {b}");
}
#[test]
fn snell_bends_toward_normal_when_entering_glass() {
    let incident = V::new(0.5, -(3f32).sqrt() / 2., 0.);
    let refracted = incident.refract(V::new(0., 1., 0.), 1. / 1.5).unwrap();
    close(refracted.x, 1. / 3.);
    close(refracted.len(), 1.);
    assert!(refracted.y < incident.y);
}
#[test]
fn total_internal_reflection_has_no_transmitted_ray() {
    let incident = V::new(0.8660254, 0.5, 0.);
    assert!(incident.refract(V::new(0., -1., 0.), 1.5).is_none());
}
#[test]
fn glass_slab_preserves_outgoing_angle() {
    let initial = V::new(0.4, 0., -1.).unit();
    let inside = initial.refract(V::new(0., 0., 1.), 1. / 1.5).unwrap();
    let outside = inside.refract(V::new(0., 0., 1.), 1.5).unwrap();
    close(outside.x, initial.x);
    close(outside.z, initial.z);
}
#[test]
fn box_handles_inside_parallel_and_behind_rays() {
    let b = Block::new(V::new(0., 0., -3.), V::splat(2.), 0, V::splat(1.));
    let r = Ray::new(V::default(), V::new(0., 0., -1.));
    let hit = b.intersect(r, 100., 0).unwrap();
    close(hit.t, 2.);
    assert_eq!(hit.normal, V::new(0., 0., 1.));
    assert!(
        b.intersect(Ray::new(V::new(2., 0., 0.), r.d), 100., 0)
            .is_none()
    );
    assert!(b.intersect(Ray::new(V::default(), -r.d), 100., 0).is_none());
    let inside = b
        .intersect(Ray::new(V::new(0., 0., -3.), V::new(1., 0., 0.)), 100., 0)
        .unwrap();
    close(inside.t, 1.);
    assert_eq!(inside.normal, V::new(1., 0., 0.));
    assert!(
        b.intersect(Ray::new(V::new(0., 0., -3.), V::new(1., 0., 0.)), 0.5, 0)
            .is_none()
    );
}
#[test]
fn accelerated_intersections_match_brute_force() {
    let blocks: Vec<_> = (0..80)
        .map(|i| {
            Block::new(
                V::new(
                    noise(i, 1, 2) * 10. - 5.,
                    noise(i, 3, 4) * 10. - 5.,
                    noise(i, 5, 6) * 10. - 5.,
                ),
                V::splat(0.3 + noise(i, 7, 8)),
                0,
                V::splat(1.),
            )
        })
        .collect();
    let bvh = Bvh::build(&blocks);
    for i in 0..400 {
        let ray = Ray::new(
            V::new(0., 0., 12.),
            V::new(noise(i, 2, 3) - 0.5, noise(i, 4, 5) - 0.5, -1.),
        );
        let fast = bvh.hit(&blocks, ray, 100.);
        let slow = blocks
            .iter()
            .enumerate()
            .filter_map(|(j, b)| b.intersect(ray, 100., j))
            .min_by(|a, b| a.t.total_cmp(&b.t));
        assert_eq!(fast.is_some(), slow.is_some());
        if let (Some(a), Some(b)) = (fast, slow) {
            close(a.t, b.t);
            assert_eq!(a.object, b.object);
        }
    }
}
#[test]
fn orbit_and_zoom_keep_target_at_image_center() {
    let mut c = Camera::default();
    for yaw in [0., 90., 180., 270.] {
        c.yaw = yaw;
        for distance in [8., 23., 40.] {
            c.distance = distance;
            let r = c.ray(320., 200., 640, 400);
            close((c.target - c.position()).len(), distance);
            close((r.at(distance) - c.target).len(), 0.);
        }
    }
}
#[test]
fn cubemap_directions_round_trip_on_all_six_faces() {
    for face in 0..6 {
        for (u, v) in [(-0.7, 0.3), (0.2, -0.4), (0., 0.)] {
            let d = Skybox::direction(face, u, v);
            let (f, uu, vv) = Skybox::face_uv(d);
            assert_eq!(face, f);
            close(u, uu);
            close(v, vv);
        }
    }
    let sky = Skybox::new(32);
    let a = sky.sample(V::new(1.0001, 0.4, 1.).unit());
    let b = sky.sample(V::new(1., 0.4, 1.0001).unit());
    assert!((a - b).len() < 0.02);
}
#[test]
fn materials_preserve_energy_and_have_valid_optical_parameters() {
    let mats = material::materials();
    assert_eq!(mats.len(), 5);
    for m in &mats {
        assert!(
            m.reflectivity >= 0. && m.transparency >= 0. && m.reflectivity + m.transparency <= 1.
        );
        assert!(m.ior >= 1.);
        assert!(m.specular >= 0. && m.specular <= 1.);
        assert!(m.shininess > 0.);
    }
}

#[test]
fn bvh_handles_parallel_grazing_inside_and_limited_rays_in_full_scene() {
    use falcon_diorama::scene::Scene;
    let scene = Scene::new();
    let check = |ray: Ray, limit| {
        let fast = scene.hit(ray, limit);
        let slow = scene
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(i, b)| b.intersect(ray, limit, i))
            .min_by(|a, b| a.t.total_cmp(&b.t));
        assert_eq!(fast.is_some(), slow.is_some(), "ray={ray:?}, limit={limit}");
        if let (Some(a), Some(b)) = (fast, slow) {
            close(a.t, b.t);
            // En aristas compartidas pueden existir varios bloques a igual distancia.
            assert!((a.point - b.point).len() < 1e-4);
        }
    };
    for (i, block) in scene.blocks.iter().enumerate().step_by(41) {
        let center = (block.bounds.lo + block.bounds.hi) * 0.5;
        for direction in [
            V::new(1., 0., 0.),
            V::new(0., -1., 0.),
            V::new(0., 0., 1.),
            V::new(1e-10, 1., -1e-10),
            V::new(1., 1e-8, 0.),
        ] {
            for origin in [center, block.bounds.lo, block.bounds.hi] {
                check(Ray::new(origin, direction), 100.);
                check(Ray::new(origin, direction), 0.05 + noise(i as i32, 3, 4));
            }
        }
    }
    for i in 0..1500 {
        let origin = V::new(
            noise(i, 1, 3) * 24. - 12.,
            noise(i, 2, 7) * 10. - 2.,
            noise(i, 4, 6) * 24. - 12.,
        );
        let direction = V::new(
            noise(i, 8, 2) - 0.5,
            noise(i, 3, 9) - 0.5,
            noise(i, 6, 2) - 0.5,
        );
        check(Ray::new(origin, direction), 0.1 + noise(i, 3, 1) * 50.);
    }
}

#[test]
fn bvh_accepts_empty_and_coincident_geometry() {
    let ray = Ray::new(V::default(), V::new(0., 0., -1.));
    assert!(Bvh::build(&[]).hit(&[], ray, 100.).is_none());
    let blocks = vec![Block::new(V::new(0., 0., -4.), V::splat(2.), 0, V::splat(1.)); 1000];
    let tree = Bvh::build(&blocks);
    close(tree.hit(&blocks, ray, 100.).unwrap().t, 3.);
    assert!(tree.hit(&blocks, ray, 2.).is_none());
}
