use super::show::{Hover, place, share, thumb_left, track_colour};
use super::*;

#[test]
fn the_thumb_slides_from_one_end_to_the_other() {
    assert_eq!(place(true, 0.0), 0.0);
    assert_eq!(place(true, 1.0), 1.0);
    assert_eq!(place(false, 0.0), 1.0);
    assert_eq!(place(false, 1.0), 0.0);
    // Eased: halfway in time is halfway across, slower at the ends.
    assert_eq!(place(true, 0.5), 0.5);
    assert!(place(true, 0.1) < 0.1);
    assert_eq!(thumb_left(0.0), INSET);
    assert_eq!(thumb_left(1.0), TRACK.x - THUMB - INSET);
}

#[test]
fn a_slide_is_done_after_its_time() {
    assert_eq!(share(1.0, 1.0), 0.0);
    assert_eq!(share(0.0, SLIDE_SECONDS), 1.0);
    assert_eq!(share(1.0, 5.0), 1.0);
}

#[test]
fn tracks_show_on_off_and_disabled_apart() {
    let off = track_colour(0.0, false, Hover::Away);
    let on = track_colour(1.0, false, Hover::Away);
    assert_ne!(off, on);
    assert_ne!(track_colour(1.0, true, Hover::Away), on);
    assert_ne!(track_colour(1.0, false, Hover::Over), on);
    // A disabled one doesn't light up.
    assert_eq!(
        track_colour(1.0, true, Hover::Over),
        track_colour(1.0, true, Hover::Away)
    );
}

fn app() -> App {
    let mut app = App::new();
    app.add_message::<ButtonPressed>()
        .add_message::<SwitchFlipped>()
        .init_resource::<Slides>()
        .insert_resource(Time::<Real>::default())
        .add_systems(Update, flip_switches);
    app
}

fn press(app: &mut App, entity: Entity) -> Vec<SwitchFlipped> {
    app.world_mut().write_message(ButtonPressed(entity));
    app.update();
    let messages = app.world().resource::<Messages<SwitchFlipped>>();
    messages.iter_current_update_messages().copied().collect()
}

#[test]
fn a_press_flips_a_switch_and_says_so() {
    let mut app = app();
    let id = UiId::new("test", "switch");
    let switch = app
        .world_mut()
        .spawn((Switch(false), ButtonState::Normal, id))
        .id();
    assert_eq!(press(&mut app, switch), [SwitchFlipped(switch, true)]);
    assert_eq!(app.world().get::<Switch>(switch), Some(&Switch(true)));
    assert!(
        app.world()
            .resource::<Slides>()
            .0
            .contains_key(&SlideKey::Id(id))
    );
    assert_eq!(press(&mut app, switch), [SwitchFlipped(switch, false)]);
}

#[test]
fn a_disabled_switch_stays_as_it_is() {
    let mut app = app();
    let switch = app
        .world_mut()
        .spawn((Switch(true), ButtonState::DisabledSelected))
        .id();
    assert!(press(&mut app, switch).is_empty());
    assert_eq!(app.world().get::<Switch>(switch), Some(&Switch(true)));
}

#[test]
fn a_disabled_switch_is_plainly_disabled() {
    use super::show::{mark_colour, shadow_colour, thumb_colour};
    // The same flat grey whichever way it is set.
    assert_eq!(
        track_colour(0.0, true, Hover::Away),
        track_colour(1.0, true, Hover::Away)
    );
    assert_ne!(thumb_colour(true), thumb_colour(false));
    assert_eq!(shadow_colour(true).alpha(), 0.0);
    assert!(shadow_colour(false).alpha() > 0.0);
    assert!(mark_colour(true).alpha() < mark_colour(false).alpha());
}
