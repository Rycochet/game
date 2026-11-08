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
