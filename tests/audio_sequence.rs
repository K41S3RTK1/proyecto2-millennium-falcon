use falcon_diorama::audio::{Event, Sequence, Track};

#[test]
fn intro_is_silent_until_title_and_skip_plays_whole_tie_before_cantina() {
    let mut s = Sequence::default();
    s.event(Event::BeginIntro);
    assert_eq!(s.track, None);
    s.event(Event::View(6));
    assert_eq!(s.track, None);
    s.event(Event::StartTheme);
    assert_eq!(s.track, Some(Track::Theme));
    s.event(Event::SkipIntro);
    assert_eq!(s.track, Some(Track::Tie));
    s.event(Event::Finished(Track::Theme));
    assert_eq!(s.track, Some(Track::Tie));
    s.event(Event::Finished(Track::Tie));
    assert_eq!(s.track, Some(Track::Cantina));
}

#[test]
fn completed_theme_returns_to_scene_and_intro_can_restart() {
    let mut s = Sequence::default();
    for _ in 0..2 {
        s.event(Event::BeginIntro);
        assert!(!s.theme_finished);
        s.event(Event::StartTheme);
        s.event(Event::Finished(Track::Theme));
        assert!(s.theme_finished);
        assert_eq!(s.track, None);
        s.event(Event::FinishIntro);
        assert_eq!(s.track, Some(Track::Cantina));
    }
}

#[test]
fn vader_theme_starts_only_after_saber_and_leaves_on_other_views() {
    let mut s = Sequence::default();
    for next_view in 0..6 {
        s.event(Event::View(6));
        assert_eq!(s.track, Some(Track::Saber));
        s.event(Event::Finished(Track::Saber));
        assert_eq!(s.track, Some(Track::Vader));
        s.event(Event::View(next_view));
        assert!(!matches!(s.track, Some(Track::Saber | Track::Vader)));
    }
    s.event(Event::View(6));
    s.event(Event::Finished(Track::Saber));
    s.event(Event::View(6));
    assert_eq!(s.track, Some(Track::Saber));
    s.event(Event::BeginIntro);
    s.event(Event::Finished(Track::Saber));
    assert_eq!(s.track, None);
}

#[test]
fn effects_return_to_cantina_and_interruption_cannot_resume_wrong_track() {
    let mut s = Sequence::default();
    for (view, effect) in [(5, Track::Droid), (1, Track::Falcon)] {
        s.event(Event::View(view));
        assert_eq!(s.track, Some(effect));
        s.event(Event::Finished(effect));
        assert_eq!(s.track, Some(Track::Cantina));
    }
    s.event(Event::View(5));
    s.event(Event::View(1));
    s.event(Event::Finished(Track::Droid));
    assert_eq!(s.track, Some(Track::Falcon));
    s.event(Event::View(6));
    s.event(Event::View(0));
    s.event(Event::Finished(Track::Saber));
    assert_eq!(s.track, Some(Track::Cantina));
    s.event(Event::Stop);
    s.event(Event::Finished(Track::Falcon));
    assert_eq!(s.track, None);
}

#[test]
fn mute_survives_intro_and_camera_changes_without_changing_sequence() {
    let mut s = Sequence::default();
    s.event(Event::ToggleMute);
    for event in [
        Event::BeginIntro,
        Event::StartTheme,
        Event::SkipIntro,
        Event::Finished(Track::Tie),
        Event::View(6),
        Event::Finished(Track::Saber),
    ] {
        s.event(event);
        assert!(s.muted);
    }
    assert_eq!(s.track, Some(Track::Vader));
    s.event(Event::ToggleMute);
    assert!(!s.muted);
    assert_eq!(s.track, Some(Track::Vader));
}

#[test]
fn rebel_entry_and_manual_shot_are_distinct_and_limited_to_view_eight() {
    let mut s = Sequence::default();
    s.event(Event::FinishIntro);
    s.event(Event::Fire);
    assert_eq!(s.track, Some(Track::Cantina));
    s.event(Event::View(7));
    assert_eq!(s.track, Some(Track::Blaster));
    s.event(Event::Fire);
    assert_eq!(s.track, Some(Track::Shot));
    s.event(Event::Finished(Track::Blaster));
    assert_eq!(s.track, Some(Track::Shot));
    s.event(Event::Finished(Track::Shot));
    assert_eq!(s.track, Some(Track::Cantina));
    s.event(Event::Fire);
    s.event(Event::View(6));
    s.event(Event::Finished(Track::Shot));
    s.event(Event::Fire);
    assert_eq!(s.track, Some(Track::Saber));
    s.event(Event::BeginIntro);
    s.event(Event::Fire);
    assert_eq!(s.track, None);
}
