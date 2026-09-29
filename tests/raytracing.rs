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
