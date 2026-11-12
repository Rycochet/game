use super::*;

fn cursor_app(cursor: FakeCursor) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Touches>()
        .insert_resource(UiScale(2.0))
        .init_resource::<PointerPosition>()
        .init_resource::<UiPointer>()
        .insert_resource(cursor)
        .add_systems(PreUpdate, point_there);
    app
}

#[test]
fn the_game_points_where_the_fake_cursor_is() {
    let at = Vec2::new(100.0, 40.0);
    let mut app = cursor_app(FakeCursor {
        at: Some(at),
        used_at: None,
    });
    app.update();
    assert_eq!(app.world().resource::<PointerPosition>().0, Some(at));
    assert_eq!(
        app.world().resource::<UiPointer>().0,
        Some(Vec2::new(50.0, 20.0))
    );
}

#[test]
fn without_one_the_pointer_is_left_alone() {
    let mut app = cursor_app(FakeCursor::default());
    app.world_mut().resource_mut::<PointerPosition>().0 = Some(Vec2::ONE);
    app.update();
    assert_eq!(app.world().resource::<PointerPosition>().0, Some(Vec2::ONE));
}

#[test]
fn quiet_fake_input_hands_the_pointer_back() {
    let mut app = cursor_app(FakeCursor {
        at: Some(Vec2::ONE),
        used_at: Some(-HAND_BACK_SECONDS - 1.0),
    });
    app.update();
    assert_eq!(app.world().resource::<FakeCursor>().at, None);
    assert_eq!(app.world().resource::<PointerPosition>().0, None);
    // One held until let go stays.
    let mut app = cursor_app(FakeCursor {
        at: Some(Vec2::ONE),
        used_at: None,
    });
    app.update();
    assert_eq!(app.world().resource::<FakeCursor>().at, Some(Vec2::ONE));
}
