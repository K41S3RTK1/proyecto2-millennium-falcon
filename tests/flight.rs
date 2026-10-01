use falcon_diorama::{
    audio::{Event, Sequence, Track},
    flight::{Flight, SCALE},
    math::{Ray, V},
    render::{self, Settings},
    scene::Scene,
};

#[test]
fn flight_audio_waits_for_takeoff_then_boost_returns_to_space_and_exit_cancels_it() {
    let mut s = Sequence::default();
    s.event(Event::View(8));
    assert_eq!(s.track, Some(Track::Takeoff));
    assert!(!s.flight_ready);
    s.event(Event::Boost);
    assert_eq!(s.track, Some(Track::Takeoff));
    s.event(Event::Finished(Track::Takeoff));
    assert!(s.flight_ready);
    assert_eq!(s.track, Some(Track::FlightAmbient));
    s.event(Event::Boost);
    assert_eq!(s.track, Some(Track::Boost));
    s.event(Event::Finished(Track::Boost));
    assert_eq!(s.track, Some(Track::FlightAmbient));
    s.event(Event::ToggleMute);
    s.event(Event::Boost);
    assert!(s.muted);
    s.event(Event::View(7));
    s.event(Event::Finished(Track::Boost));
    assert_eq!(s.track, Some(Track::Blaster));
    assert!(!s.flight_ready);
    s.event(Event::Boost);
    assert_eq!(s.track, Some(Track::Blaster));
    s.event(Event::View(8));
    assert_eq!(s.track, Some(Track::Takeoff));
    s.event(Event::Finished(Track::Takeoff));
    s.event(Event::View(8));
    assert!(!s.flight_ready);
    assert_eq!(s.track, Some(Track::Takeoff));
    s.event(Event::BeginIntro);
    s.event(Event::Boost);
    s.event(Event::Finished(Track::Takeoff));
    assert_eq!(s.track, None);
    assert!(!s.flight_ready);
}

#[test]
fn exhaust_is_bounded_behind_engine_and_respects_first_hit() {
    let f = Flight {
        active: true,
        progress: 1.,
        boost_age: 1.,
        ..Default::default()
    };
    let origin = V::new(0., 1.61 * SCALE, 10. * SCALE);
    let r = Ray::new(origin + V::new(0., 4., 0.), V::new(0., -1., 0.));
    assert!(f.glow(r, 10.).z > 0.1);
    assert_eq!(f.glow(r, 0.5), V::default());
    assert_eq!(f.glow(Ray::new(r.o, -r.d), 20.), V::default());
    assert_eq!(
        f.glow(Ray::new(V::new(0., 7., -6.), V::new(0., -1., 0.)), 20.),
        V::default()
    );
    for age in [-1., 0., f.boost_duration, f.boost_duration + 1.] {
        assert_eq!(
            Flight {
                boost_age: age,
                ..f
            }
            .glow(r, 20.),
            V::default()
        );
    }
    assert_eq!(Flight { progress: 0.5, ..f }.glow(r, 20.), V::default());
}

#[test]
fn flight_has_only_enlarged_ship_and_render_translation_is_rigid() {
    let scene = Scene::flight();
    assert!(scene.blocks.len() > 3000 && scene.blocks.len() < 5902);
    assert!(scene.blocks.iter().all(|b| b.bounds.lo.y > 1.0 * SCALE));
    assert_eq!(scene.lights.len(), 2);
    let f = Flight {
        active: true,
        progress: 0.6,
        ..Default::default()
    };
    let cfg = Settings {
        width: 30,
        height: 20,
        quality: 0,
        flight: f,
        ..Settings::default()
    };
    let cam = f.camera();
    let ray = cam.ray(15., 10., 30, 20);
    let a = render::trace(&scene, ray, cfg, 0, 1.);
    // Trasladar rayo y geometría por igual conserva el resultado óptico.
    let local = Ray::new(ray.o - f.offset(), ray.d);
    let b = render::trace(
        &scene,
        local,
        Settings {
            flight: Flight { progress: 0., ..f },
            skybox: false,
            ..cfg
        },
        0,
        1.,
    );
    let c = render::trace(
        &scene,
        ray,
        Settings {
            skybox: false,
            ..cfg
        },
        0,
        1.,
    );
    assert!((b - c).len() < 0.001);
    assert!(a.x.is_finite() && a.y.is_finite() && a.z.is_finite());
    assert_eq!(Scene::new().blocks.len(), 5902);
}
