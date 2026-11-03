use super::*;
use shared::id::Id;
use shared::item::{ItemKind, Made, Quality};

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(give_auras).initialize(&mut World::new());
    IntoSystem::into_system(pulse_auras).initialize(&mut World::new());
}

#[test]
fn only_what_is_known_to_be_special_glows() {
    let plain = Item::new(Id::for_test(1), ItemKind::Hat);
    assert_eq!(Aura::of(&plain), None);
    let magical = Item {
        made: Made {
            magical: true,
            ..plain.made
        },
        quality: Some(Quality::Rare),
        ..plain.clone()
    };
    assert_eq!(Aura::of(&magical).unwrap().colours(), (MAGICAL, MAGICAL));
    let both = Item {
        favour: Some(Favour::Cursed),
        ..magical
    };
    assert_eq!(Aura::of(&both).unwrap().colours(), (MAGICAL, CURSED));
    let blessed = Item {
        favour: Some(Favour::Blessed),
        ..plain
    };
    assert_eq!(Aura::of(&blessed).unwrap().colours(), (BLESSED, BLESSED));
}

#[test]
fn a_mask_is_solid_only_on_its_cells() {
    let image = shape_mask(Shape::rectangle(2, 1));
    let data = image.data.unwrap();
    assert_eq!(data.len(), 16 * 8 * 4);
    assert!(data.iter().all(|&byte| byte == 255));
}
