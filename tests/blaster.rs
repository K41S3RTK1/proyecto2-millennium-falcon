use falcon_diorama::{
    blaster::{DURATION, Shot},
    math::{Ray, V},
    scene::Scene,
};
#[test]
fn projectile_moves_from_muzzle_and_disappears_at_end() {
    let early = Shot {
        age: 0.03,
        ..Shot::default()
    };
    let late = Shot { age: 0.20, ..early };
    let (_, a, m, f) = early.segments();
    let (_, b, _, g) = late.segments();
    assert!(a.x > m.x && b.x > a.x && f > g);
    assert!(!Shot::default().active());
    assert!(
        !Shot {
            age: DURATION,
            ..early
        }
        .active()
    );
    assert!(
        Shot {
            soldier: 1,
            ..early
        }
        .segments()
        .2
        .z > m.z
    );
}
#[test]
fn beam_is_occluded_and_never_glows_behind_camera() {
    let shot = Shot {
        age: 0.1,
        ..Shot::default()
    };
    let (a, b, _, _) = shot.segments();
    let middle = (a + b) * 0.5;
    let ray = Ray::new(middle + V::new(0., 0., -2.), V::new(0., 0., 1.));
    assert!(shot.glow(ray, 3.).x > 1.);
    assert_eq!(shot.glow(ray, 1.), V::default());
    assert_eq!(shot.glow(Ray::new(ray.o, -ray.d), 3.), V::default());
}
#[test]
fn both_barrels_fire_outward_without_hitting_soldiers_or_ship() {
    let scene = Scene::new();
    for i in 0..2 {
        let muzzle = falcon_diorama::scene::rebel_muzzle(i);
        assert!(
            scene
                .hit(Ray::new(muzzle, V::new(1., 0., 0.)), 6.)
                .is_none()
        );
    }
}
