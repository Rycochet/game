use bevy::ecs::system::RunSystemOnce;
use bevy::math::Affine2;
use bevy::ui::UiGlobalTransform;

use super::*;
use crate::widgets::UiId;
use crate::widgets::key_box::KeyWaiting;
use crate::widgets::text_box::TextFocus;

/// An app with a widget `pad/up` laid out from (100, 50), 40 by 20 interface
/// pixels, shown at an interface size of `scale`.
fn app(scale: f32) -> App {
    let mut app = App::new();
    app.insert_resource(UiScale(scale))
        .init_resource::<TextFocus>()
        .init_resource::<KeyWaiting>();
    let (corner, size) = (Vec2::new(100.0, 50.0), Vec2::new(40.0, 20.0));
    // Laid out in physical pixels, the window's own scale being 1 here.
    app.world_mut().spawn((
        UiId::new("pad", "up"),
        ComputedNode {
            size: size * scale,
            inverse_scale_factor: 1.0 / scale,
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation((corner + size / 2.0) * scale)),
        InheritedVisibility::VISIBLE,
    ));
    app
}

fn point(app: &mut App, id: &'static str, from_corner: Option<Vec2>) -> BrpResult<Vec2> {
    app.world_mut()
        .run_system_once(move |places: WidgetPlaces| places.point(id, from_corner))
        .expect("the search runs")
}

#[test]
fn a_widget_is_found_by_id_in_logical_pixels() {
    let mut app = app(1.0);
    assert_eq!(
        point(&mut app, "pad/up", None).ok(),
        Some(Vec2::new(120.0, 60.0))
    );
    assert_eq!(
        point(&mut app, "pad/up", Some(Vec2::new(5.0, 5.0))).ok(),
        Some(Vec2::new(105.0, 55.0))
    );
    assert!(point(&mut app, "pad/down", None).is_err());
}

#[test]
fn the_interface_size_is_allowed_for() {
    // At 150% the widget is drawn half as big again, further from the corner.
    let mut app = app(1.5);
    assert_eq!(
        point(&mut app, "pad/up", None).ok(),
        Some(Vec2::new(180.0, 90.0))
    );
}
