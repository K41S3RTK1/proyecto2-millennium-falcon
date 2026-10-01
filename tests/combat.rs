use falcon_diorama::{
    combat::{Combat, TARGETS, TRAVEL},
    math::{Ray, V},
    scene::Scene,
};
#[test]
fn damage_waits_for_arrival_and_three_hits_destroy_each_tie() {
    let mut c = Combat::default();
    assert!(!c.fire());
    c.enabled = true;
    for target in 0..3 {
        for remaining in (0..3).rev() {
            assert!(c.fire());
            assert_eq!(c.target, target);
            assert!(!c.fire());
            c.update(TRAVEL / 2.);
            assert_eq!(c.hp[target], remaining + 1);
            c.update(TRAVEL / 2.);
            assert_eq!(c.hp[target], remaining);
            c.update(0.1);
            assert_eq!(c.hp[target], remaining);
        }
        assert_eq!(c.mask() & (1 << target), 0);
    }
    assert_eq!(c.hp, [0; 3]);
    assert!(!c.fire());
    c.update(2.);
    assert_eq!(c.impacts, [-1.; 3]);
}
#[test]
fn destroyed_tie_is_removed_from_raytraced_geometry() {
    let all = Scene::flight_combat(7);
    let destroyed = Scene::flight_combat(6);
    let ray = Ray::new(TARGETS[0] + V::new(0., 0., 5.), V::new(0., 0., -1.));
    assert!(all.hit(ray, 6.).is_some());
    assert!(destroyed.hit(ray, 6.).is_none());
    assert!(all.blocks.len() > destroyed.blocks.len());
    assert_eq!(
        Scene::flight_combat(0).blocks.len(),
        Scene::flight().blocks.len()
    );
}
#[test]
fn laser_respects_occlusion_and_reset_clears_combat() {
    let mut c = Combat {
        enabled: true,
        ..Default::default()
    };
    c.fire();
    c.update(0.3);
    let (a, b) = c.segment();
    let ray = Ray::new((a + b) * 0.5 + V::new(0., 5., 0.), V::new(0., -1., 0.));
    assert!(c.glow(ray, 10.).x > 0.);
    assert_eq!(c.glow(ray, 1.), V::default());
    c = Combat::default();
    assert_eq!(c.mask(), 0);
    assert_eq!(c.glow(ray, 10.), V::default());
}
