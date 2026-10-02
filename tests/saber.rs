use falcon_diorama::{
    camera::Camera,
    geometry::Bvh,
    material::PLASMA,
    math::{Ray, V},
    render::{self, Settings},
    scene::{SABER_BOTTOM, SABER_LIGHT_INDEX, Scene, VADER_ORIGIN},
};

#[test]
fn off_blade_is_absent_for_every_segment_but_hilt_remains() {
    let scene = Scene::new();
    for i in 0..22 {
        let point = SABER_BOTTOM + V::new(0., 0.04 + i as f32 * 0.08, 0.);
        let ray = Ray::new(point + V::new(0.5, 0., 0.), V::new(-1., 0., 0.));
        let hit = scene.hit_with_saber(ray, 0.6, true).unwrap();
        assert_eq!(scene.blocks[hit.object].material, PLASMA);
        assert!(scene.hit_with_saber(ray, 0.6, false).is_none());
    }
    let hilt = SABER_BOTTOM - V::new(0., 0.18, 0.);
    let ray = Ray::new(hilt + V::new(0.5, 0., 0.), V::new(-1., 0., 0.));
    let on = scene.hit_with_saber(ray, 0.6, true).unwrap();
    let off = scene.hit_with_saber(ray, 0.6, false).unwrap();
    assert_eq!(on.object, off.object);
    assert_ne!(scene.blocks[off.object].material, PLASMA);
}

#[test]
fn off_render_matches_physically_removed_blade_and_light() {
    let scene = Scene::new();
    let mut removed = Scene::new();
    removed.blocks.retain(|block| block.material != PLASMA);
    removed.bvh = Bvh::build(&removed.blocks);
    removed.lights.remove(SABER_LIGHT_INDEX);
    let camera = Camera {
        yaw: 20.,
        pitch: 12.,
        distance: 5.7,
        target: VADER_ORIGIN + V::new(0.2, 1.40, -0.12),
        ..Default::default()
    };
    let settings = Settings {
        width: 96,
        height: 64,
        quality: 1,
        saber_on: false,
        ..Default::default()
    };
    let off = render::render(&scene, camera, settings);
    let reference = render::render(&removed, camera, settings);
    assert_eq!(
        off.pixels, reference.pixels,
        "No deben quedar luz, oclusión, sombras ni reflejos de la hoja retirada"
    );
    let on = render::render(
        &scene,
        camera,
        Settings {
            saber_on: true,
            ..settings
        },
    );
    assert_ne!(off.pixels, on.pixels);
}

#[test]
fn off_saber_cannot_leave_a_halo_on_an_empty_background() {
    let mut scene = Scene::new();
    scene.blocks.clear();
    scene.bvh = Bvh::build(&scene.blocks);
    scene.lights.clear();
    let ray = Ray::new(SABER_BOTTOM + V::new(0.02, 0.8, 1.), V::new(0., 0., -1.));
    let settings = Settings {
        skybox: false,
        saber_on: false,
        ..Default::default()
    };
    let off = render::trace(&scene, ray, settings, 0, 1.);
    assert_eq!(off, V::new(0.05, 0.06, 0.08));
    let on = render::trace(
        &scene,
        ray,
        Settings {
            saber_on: true,
            ..settings
        },
        0,
        1.,
    );
    assert!(on.x > off.x + 0.1);
}
