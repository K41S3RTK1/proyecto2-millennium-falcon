use falcon_diorama::{
    camera::Camera,
    render::{self, Settings},
    scene::Scene,
};
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn cancelled_render_is_discarded_and_next_camera_can_render() {
    let scene = Scene::new();
    let generation = AtomicU64::new(1);
    let cfg = Settings {
        width: 32,
        height: 24,
        quality: 0,
        ..Settings::default()
    };
    assert!(render::render_interruptible(&scene, Camera::default(), cfg, &generation, 0).is_none());
    generation.store(2, Ordering::Relaxed);
    let camera = Camera {
        yaw: 140.,
        ..Camera::default()
    };
    let resumed = render::render_interruptible(&scene, camera, cfg, &generation, 2).unwrap();
    let expected = render::render(&scene, camera, cfg);
    assert_eq!(resumed.pixels, expected.pixels);
    assert!(resumed.pixels.chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn dynamic_bands_match_individual_rays_including_partial_last_band() {
    let scene = Scene::new();
    let camera = Camera::default();
    let cfg = Settings {
        width: 37,
        height: 29,
        quality: 0,
        ..Settings::default()
    };
    let frame = render::render(&scene, camera, cfg);
    for y in 0..cfg.height {
        for x in 0..cfg.width {
            let ray = camera.ray(x as f32 + 0.5, y as f32 + 0.5, cfg.width, cfg.height);
            let color = render::display(render::trace(&scene, ray, cfg, 0, 1.));
            let offset = (y * cfg.width + x) * 4;
            assert_eq!(
                &frame.pixels[offset..offset + 4],
                &[color[0], color[1], color[2], 255]
            );
        }
    }
}
