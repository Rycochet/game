use shared::movement::Step;

use super::*;

fn app() -> App {
    let mut app = App::new();
    app.add_message::<TakeStep>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Preferences>()
        .init_resource::<ViewFacing>()
        .init_resource::<PlayerEnergy>()
        .init_resource::<PlayerMoving>()
        .add_systems(Update, move_player);
    app
}

fn steps_asked(app: &mut App) -> usize {
    app.world_mut()
        .resource_mut::<Messages<TakeStep>>()
        .drain()
        .count()
}

#[test]
fn a_held_key_never_lines_up_more_than_one_step() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Numpad8);
    app.update();
    assert_eq!(steps_asked(&mut app), 1);
    // The step is waiting for energy: holding on asks for no more.
    app.world_mut()
        .resource_mut::<PlayerEnergy>()
        .waiting
        .push_back(Step::new(0, -1).expect("a step"));
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(steps_asked(&mut app), 0);
    // Let go, and the step taken: nothing more is asked for.
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::Numpad8);
    keys.clear();
    app.world_mut()
        .resource_mut::<PlayerEnergy>()
        .waiting
        .clear();
    app.update();
    assert_eq!(steps_asked(&mut app), 0);
    assert!(!app.world().resource::<PlayerMoving>().keys);
}

#[test]
fn a_held_pad_arrow_steps_as_a_key_does() {
    let mut app = app();
    app.insert_resource(PadSteps { dx: -1, dy: 0 });
    app.update();
    assert_eq!(steps_asked(&mut app), 1);
    assert!(app.world().resource::<PlayerMoving>().keys);
    app.insert_resource(PadSteps::default());
    app.update();
    assert_eq!(steps_asked(&mut app), 0);
    assert!(!app.world().resource::<PlayerMoving>().keys);
}

/// Puts the steps asked for this frame in line, as `take_steps` does when
/// there isn't the energy to take them yet.
fn line_up(app: &mut App) {
    let asked: Vec<Step> = app
        .world_mut()
        .resource_mut::<Messages<TakeStep>>()
        .drain()
        .map(|take| take.0)
        .collect();
    app.world_mut()
        .resource_mut::<PlayerEnergy>()
        .waiting
        .extend(asked);
}

#[test]
fn a_click_lasting_a_few_frames_is_one_step() {
    let mut app = app();
    app.insert_resource(PadSteps { dx: 1, dy: 0 });
    app.update();
    // Taken at once.
    assert_eq!(steps_asked(&mut app), 1);
    // Still held next frame: the next is asked for, and waits for energy.
    app.update();
    line_up(&mut app);
    assert_eq!(app.world().resource::<PlayerEnergy>().waiting.len(), 1);
    // Let go before it is taken: it is dropped.
    app.insert_resource(PadSteps::default());
    app.update();
    assert!(app.world().resource::<PlayerEnergy>().waiting.is_empty());
}

#[test]
fn a_tap_whose_step_waits_for_energy_still_steps() {
    let mut app = app();
    app.insert_resource(PadSteps { dx: 0, dy: 1 });
    app.update();
    line_up(&mut app);
    app.update();
    app.insert_resource(PadSteps::default());
    app.update();
    assert_eq!(app.world().resource::<PlayerEnergy>().waiting.len(), 1);
}
