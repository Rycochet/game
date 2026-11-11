use shared::ability::Ability;

use super::*;
use crate::widgets::AbilityLook;

fn app() -> App {
    let mut app = App::new();
    app.add_systems(Update, (draw_ability_grids, draw_ability_slots));
    app
}

fn thing(ability: Ability) -> AbilityThing {
    let look = AbilityLook {
        icon: Handle::default(),
        name: ability.name().to_string(),
        passive: !ability.is_active(),
        usable: true,
        colour: Color::WHITE,
    };
    AbilityThing { ability, look }
}

fn cells(app: &mut App) -> Vec<AbilityCell> {
    let mut cells = app.world_mut().query::<&AbilityCell>();
    cells.iter(app.world()).copied().collect()
}

fn parts(app: &mut App) -> Vec<SlotPart> {
    let mut parts = app.world_mut().query::<&SlotPart>();
    parts.iter(app.world()).copied().collect()
}

#[test]
fn a_list_draws_each_ability_as_its_own() {
    let mut app = app();
    let shown = AbilityGridShown {
        columns: 8,
        things: vec![thing(Ability::Attack), thing(Ability::Heal)],
    };
    let list = app.world_mut().spawn(shown).id();
    app.update();
    let drawn = cells(&mut app);
    assert_eq!(drawn.len(), 2);
    assert!(drawn.iter().all(|cell| cell.of == list));
    assert!(drawn.iter().any(|cell| cell.ability == Ability::Heal));
}

#[test]
fn a_slot_draws_its_ability_and_how_it_is_used() {
    let mut app = app();
    let slot = app.world_mut().spawn(AbilitySlotShown::default()).id();
    app.update();
    // Empty: a frame, and nothing to find.
    assert!(cells(&mut app).is_empty());
    assert!(parts(&mut app).is_empty());

    let shown = AbilitySlotShown {
        thing: Some(thing(Ability::Attack)),
        key: "1".to_string(),
        on: true,
        queued: true,
        shade: true,
        ..default()
    };
    *app.world_mut().get_mut::<AbilitySlotShown>(slot).unwrap() = shown;
    app.update();
    let drawn = cells(&mut app);
    assert_eq!(drawn.len(), 1);
    assert_eq!((drawn[0].of, drawn[0].ability), (slot, Ability::Attack));
    let found = parts(&mut app);
    assert!(found.contains(&SlotPart::On(slot)));
    assert!(found.contains(&SlotPart::Queued(slot)));
    assert!(found.contains(&SlotPart::Shade(slot)));
}
