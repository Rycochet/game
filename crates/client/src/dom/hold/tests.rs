use shared::id::Id;
use shared::item::{Item, ItemKind, Rotation};

use super::follow::{CLICK_PIXELS, CLICK_SECONDS, is_click};
use super::*;

#[test]
fn only_a_quick_press_in_one_place_is_a_click() {
    let from = Vec2::new(100.0, 100.0);
    assert!(is_click(from, from + Vec2::X, 0.1));
    assert!(!is_click(from, from + Vec2::X, CLICK_SECONDS));
    assert!(!is_click(from, from + Vec2::X * CLICK_PIXELS, 0.1));
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    add(&mut app);
    app
}

fn thing(by: Hand) -> HeldThing {
    let id = Id::for_test(1);
    let item = HeldItem {
        id,
        item: Item::new(id, ItemKind::Apple),
        rotation: Rotation::None,
        grip: (0, 0),
    };
    HeldThing {
        what: HeldWhat::Item(item),
        by,
    }
}

fn dropped(app: &mut App) -> usize {
    let mut messages = app.world_mut().resource_mut::<Messages<Dropped>>();
    messages.drain().count()
}

fn held(app: &App) -> bool {
    app.world().resource::<HeldObject>().thing().is_some()
}

#[test]
fn the_mouse_holds_until_its_button_is_let_go() {
    let mut app = app();
    let at = Vec2::new(50.0, 60.0);
    app.world_mut().resource_mut::<UiPointer>().0 = Some(at);
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.press(MouseButton::Left);
    // (Picked up long ago: letting go is no click.)
    let picked = Some((at, -10.0));
    let mut object = app.world_mut().resource_mut::<HeldObject>();
    object.pick_up(thing(Hand::Mouse), picked);
    app.update();
    assert!(held(&app));
    assert_eq!(app.world().resource::<HeldObject>().at, Some(at));
    assert_eq!(dropped(&mut app), 0);

    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.release(MouseButton::Left);
    app.update();
    assert!(!held(&app));
    assert_eq!(dropped(&mut app), 1);
}

#[test]
fn a_right_press_turns_it_and_what_is_kept_is_never_dropped() {
    let mut app = app();
    let mut object = app.world_mut().resource_mut::<HeldObject>();
    object.pick_up(thing(Hand::Kept), None);
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.press(MouseButton::Right);
    app.update();
    let object = app.world().resource::<HeldObject>();
    let item = object.thing().unwrap().item().unwrap();
    assert_eq!(item.rotation, Rotation::None.next());
    assert_eq!(dropped(&mut app), 0);

    app.world_mut().resource_mut::<HeldObject>().put_back();
    app.update();
    assert!(!held(&app));
    assert_eq!(dropped(&mut app), 0);
}

#[test]
fn turning_keeps_hold_of_the_same_cell() {
    // A sword lying across, held by its second cell along.
    let id = Id::for_test(1);
    let across = HeldItem {
        id,
        item: Item::new(id, ItemKind::Sword),
        rotation: Rotation::Quarter,
        grip: (1, 0),
    };
    let mut turned = across.clone();
    turned.turn();
    assert_eq!(turned.rotation, Rotation::Half);
    assert_eq!(turned.grip, (0, 1));
    // Four turns come back to where it started.
    let mut round = across.clone();
    for _ in 0..4 {
        round.turn();
    }
    assert_eq!(round, across);
}
