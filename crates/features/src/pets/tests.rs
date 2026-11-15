use super::*;

const MANIFEST: &str = include_str!("../../../../assets/pets/volt-cat/manifest.json");

fn pet(reduced: bool) -> Animator {
    Animator::new(
        Package::from_json(MANIFEST).expect("valid built-in package"),
        reduced,
    )
}

#[test]
fn blink_happy_and_sleep_transitions_have_bounded_deadlines() {
    let mut pet = pet(false);
    assert_eq!(pet.state(), State::Idle);
    pet.advance();
    assert_eq!(pet.state(), State::Blink);
    pet.advance();
    assert_eq!(pet.state(), State::Idle);
    for _ in 0..30 {
        pet.advance();
    }
    assert_eq!(pet.state(), State::Sleep);
    assert!(pet.next_delay().is_none());
    pet.react();
    assert_eq!(pet.state(), State::Happy);
    assert!(pet.next_delay().is_some());
    for _ in 0..3 {
        pet.advance();
    }
    assert_eq!(pet.state(), State::Idle);
    assert!(pet.next_delay().is_some());
}

#[test]
fn reduced_motion_never_schedules_animation_even_after_click() {
    let mut pet = pet(true);
    let frame = pet.frame().asset.clone();
    pet.react();
    pet.advance();
    assert_eq!(pet.frame().asset, frame);
    assert!(pet.next_delay().is_none());
}

#[test]
fn hiding_pauses_deadlines_and_preserves_the_current_frame() {
    let mut pet = pet(false);
    pet.react();
    pet.advance();
    let frame = pet.frame().asset.clone();
    assert!(pet.set_visible(false));
    assert!(!pet.set_visible(false));
    assert!(pet.next_delay().is_none());
    for _ in 0..50 {
        pet.advance();
    }
    pet.react();
    assert_eq!(pet.frame().asset, frame);
    assert!(pet.set_visible(true));
    assert!(pet.next_delay().is_some());
    assert_eq!(pet.frame().asset, frame);
    pet.advance();
    assert_ne!(pet.frame().asset, frame);
}

#[test]
fn missing_optional_artwork_falls_back_to_idle() {
    let mut json: serde_json::Value = serde_json::from_str(MANIFEST).expect("json");
    let animations = json["animations"].as_object_mut().expect("animations");
    animations.remove("blink");
    animations.remove("happy");
    animations.remove("sleep");
    let mut pet = Animator::new(
        Package::from_json(&json.to_string()).expect("idle fallback"),
        false,
    );
    let idle = pet.frame().asset.clone();
    pet.advance();
    assert_eq!(pet.frame().asset, idle);
    pet.react();
    assert_eq!(pet.frame().asset, idle);
}

#[test]
fn package_rejects_unsafe_paths_and_unbounded_animation_data() {
    for invalid in [
        "../cat.svg",
        "pets/../cat.svg",
        "pets//cat.svg",
        "https://host/cat.svg",
        "pets/C:/cat.svg",
        "pets/cat\\idle.svg",
    ] {
        let mut json: serde_json::Value = serde_json::from_str(MANIFEST).expect("json");
        json["animations"]["idle"][0]["asset"] = invalid.into();
        assert!(Package::from_json(&json.to_string()).is_err(), "{invalid}");
    }
    for duration in [0, 49, 60_001] {
        let mut json: serde_json::Value = serde_json::from_str(MANIFEST).expect("json");
        json["animations"]["idle"][0]["duration_ms"] = duration.into();
        assert!(Package::from_json(&json.to_string()).is_err());
    }
    for count in [0, 33] {
        let mut json: serde_json::Value = serde_json::from_str(MANIFEST).expect("json");
        let frame = json["animations"]["idle"][0].clone();
        json["animations"]["idle"] = serde_json::Value::Array(vec![frame; count]);
        assert!(Package::from_json(&json.to_string()).is_err());
    }
    for invalid in [
        "{}".to_string(),
        " ".repeat(32_769),
        MANIFEST.replace("\"schema_version\": 1", "\"schema_version\": 2"),
        MANIFEST.replace("\"idle\"", "\"unknown\""),
    ] {
        assert!(Package::from_json(&invalid).is_err());
    }
}
