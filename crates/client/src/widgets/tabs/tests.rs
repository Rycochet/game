use bevy::ecs::system::RunSystemOnce;

use super::*;

#[test]
fn exactly_one_tab_stays_chosen() {
    let mut app = App::new();
    app.add_message::<ButtonPressed>()
        .add_message::<TabChosen>();
    let row = app.world_mut().spawn(Tabs { selected: 0 }).id();
    let tab = |index| (Tab { row, index }, state_of(index, 0));
    let first = app.world_mut().spawn(tab(0)).id();
    let second = app.world_mut().spawn(tab(1)).id();

    // The one already chosen: nothing happens.
    app.world_mut().write_message(ButtonPressed(first));
    app.world_mut().run_system_once(press_tabs).unwrap();
    assert_eq!(
        app.world()
            .resource::<Messages<TabChosen>>()
            .iter_current_update_messages()
            .count(),
        0
    );

    app.world_mut().write_message(ButtonPressed(second));
    app.world_mut().run_system_once(press_tabs).unwrap();
    let state = |entity| *app.world().get::<ButtonState>(entity).unwrap();
    assert_eq!(state(first), ButtonState::Normal);
    assert_eq!(state(second), ButtonState::Selected);
    let chosen: Vec<_> = app
        .world()
        .resource::<Messages<TabChosen>>()
        .iter_current_update_messages()
        .copied()
        .collect();
    assert_eq!(chosen, vec![TabChosen(row, 1)]);
}
