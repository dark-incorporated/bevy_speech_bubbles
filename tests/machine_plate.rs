//! The machine plate: every system this crate adds is in [`SpeechBubblesSystems`], the settings
//! resource and the components are reflected, and the authoring component carries `Default` and serde.
//!
//! System names are the full path only with Bevy's `debug` feature, which the dev-dependency turns on;
//! the test asserts it found crate systems so a build without the feature cannot pass vacuously.

use std::any::TypeId;

use bevy::ecs::schedule::Schedules;
use bevy::prelude::*;
use bevy_speech_bubbles::{
    BarkSettings, Bubble, BubbleOffsetPx, BubblePx, BubbleTtl, SpeechBubblesPlugin, SpeechBubblesSystems,
    StartsConversation,
};

const CRATE: &str = "bevy_speech_bubbles::";

#[derive(Component)]
struct TestCamera;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_plugins(SpeechBubblesPlugin::<TestCamera>::default());
    app.update();
    app
}

#[test]
fn machine_plate_every_system_is_in_the_root_set() {
    let app = app();
    let sets = [SpeechBubblesSystems.intern()];
    let schedules = app.world().resource::<Schedules>();
    let mut seen = 0;
    let mut stray = Vec::new();
    for (_label, schedule) in schedules.iter() {
        let Ok(systems) = schedule.systems() else { continue };
        let members: Vec<_> = sets.iter().filter_map(|set| schedule.graph().systems_in_set(*set).ok()).collect();
        for (key, system) in systems {
            let name = system.name().to_string();
            if !name.starts_with(CRATE) {
                continue;
            }
            seen += 1;
            if !members.iter().any(|m| m.contains(&key)) {
                stray.push(name);
            }
        }
    }
    assert!(seen > 0, "no system named `{CRATE}*` found; is Bevy's `debug` feature on?");
    assert!(stray.is_empty(), "systems outside SpeechBubblesSystems: {stray:?}");
}

#[test]
fn machine_plate_settings_are_reflected_and_saveable() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    let id = TypeId::of::<BarkSettings>();
    assert!(registry.get_type_data::<ReflectResource>(id).is_some(), "BarkSettings has no ReflectResource");
    assert!(registry.get_type_data::<ReflectDefault>(id).is_some(), "BarkSettings has no ReflectDefault");
    assert!(registry.get_type_data::<ReflectSerialize>(id).is_some(), "BarkSettings has no ReflectSerialize");
    assert!(registry.get_type_data::<ReflectDeserialize>(id).is_some(), "BarkSettings has no ReflectDeserialize");
}

#[test]
fn machine_plate_components_are_reflected() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    for (name, id) in [
        ("Bubble", TypeId::of::<Bubble>()),
        ("BubblePx", TypeId::of::<BubblePx>()),
        ("BubbleOffsetPx", TypeId::of::<BubbleOffsetPx>()),
        ("BubbleTtl", TypeId::of::<BubbleTtl>()),
        ("StartsConversation", TypeId::of::<StartsConversation>()),
    ] {
        assert!(registry.get_type_data::<ReflectComponent>(id).is_some(), "{name} has no ReflectComponent");
    }
    let id = TypeId::of::<StartsConversation>();
    assert!(registry.get_type_data::<ReflectDefault>(id).is_some(), "StartsConversation has no ReflectDefault");
    assert!(registry.get_type_data::<ReflectSerialize>(id).is_some(), "StartsConversation has no ReflectSerialize");
    assert!(registry.get_type_data::<ReflectDeserialize>(id).is_some(), "StartsConversation has no ReflectDeserialize");
}
