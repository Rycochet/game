use bevy::ecs::system::RunSystemOnce;
use bevy::math::Affine2;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};

use super::*;

/// A node laid out at `corner` with `size`, in interface pixels.
fn laid_out(corner: Vec2, size: Vec2, stack: u32) -> impl Bundle {
    (
        ComputedNode {
            size,
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation(corner + size / 2.0)),
        InheritedVisibility::VISIBLE,
        ComputedStackIndex(stack),
    )
}

struct Test {
    app: App,
    button: Entity,
}

/// A hold button from (100, 100) to (140, 140), on a panel.
fn test() -> Test {
    let mut app = App::new();
    app.init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<UiPointer>()
        .init_resource::<UiScale>()
        .init_resource::<TouchGesture>()
        .init_resource::<Holding>()
        .add_message::<ButtonPressed>();
    let world = app.world_mut();
    let panel = world.spawn_empty().id();
    world.entity_mut(panel).insert((
        PanelPart::Body(panel),
        laid_out(Vec2::new(80.0, 80.0), Vec2::new(100.0, 100.0), 0),
    ));
    let button = world
        .spawn((
            HoldButton::default(),
            ButtonState::Normal,
            laid_out(Vec2::new(100.0, 100.0), Vec2::splat(40.0), 1),
        ))
        .id();
    Test { app, button }
}

impl Test {
    fn step(&mut self) -> bool {
        self.app
            .world_mut()
            .run_system_once(hold_buttons)
            .expect("the system runs");
        self.app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        self.held()
    }

    fn held(&self) -> bool {
        self.app
            .world()
            .get::<HoldButton>(self.button)
            .expect("a button")
            .held()
    }

    fn pointer(&mut self, x: f32, y: f32) {
        self.app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(x, y));
    }

    fn mouse(&mut self, down: bool) {
        let mut mouse = self
            .app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>();
        if down {
            mouse.press(MouseButton::Left);
        } else {
            mouse.release(MouseButton::Left);
        }
    }

    fn finger(&mut self, at: Option<Vec2>) {
        self.app
            .world_mut()
            .resource_mut::<TouchGesture>()
            .first_finger = at;
    }

    fn presses(&mut self) -> usize {
        let mut messages = self
            .app
            .world_mut()
            .resource_mut::<Messages<ButtonPressed>>();
        messages.drain().count()
    }
}

#[test]
fn the_mouse_holds_it_while_down_and_on_it() {
    let mut test = test();
    test.pointer(120.0, 120.0);
    assert!(!test.step());
    test.mouse(true);
    assert!(test.step());
    assert!(test.step(), "still held while the button stays down");
    assert_eq!(test.presses(), 1, "pressed once, on becoming held");
    test.pointer(170.0, 120.0);
    assert!(!test.step(), "let go when the pointer moves off");
    test.pointer(120.0, 120.0);
    assert!(test.step(), "held again on coming back");
    test.mouse(false);
    assert!(!test.step(), "let go with the mouse button");
}

#[test]
fn a_press_elsewhere_dragged_onto_it_does_not_hold_it() {
    let mut test = test();
    test.pointer(170.0, 170.0);
    test.mouse(true);
    assert!(!test.step());
    test.pointer(120.0, 120.0);
    assert!(!test.step());
}

#[test]
fn a_finger_holds_it_while_it_stays_on_it() {
    let mut test = test();
    test.finger(Some(Vec2::new(110.0, 110.0)));
    assert!(test.step());
    test.finger(Some(Vec2::new(130.0, 125.0)));
    assert!(test.step(), "moving a little on it keeps it");
    test.finger(Some(Vec2::new(200.0, 125.0)));
    assert!(!test.step(), "sliding off lets it go");
    test.finger(None);
    assert!(!test.step());
    // A finger landing elsewhere and sliding on doesn't take it.
    test.finger(Some(Vec2::new(170.0, 170.0)));
    assert!(!test.step());
    test.finger(Some(Vec2::new(120.0, 120.0)));
    assert!(!test.step());
}

#[test]
fn the_tap_a_lifted_finger_makes_does_not_hold_it_again() {
    let mut test = test();
    test.finger(Some(Vec2::new(120.0, 120.0)));
    assert!(test.step());
    test.finger(None);
    test.app.world_mut().resource_mut::<TouchGesture>().tap = true;
    test.pointer(120.0, 120.0);
    test.mouse(true);
    assert!(!test.step());
    assert_eq!(test.presses(), 1);
}

#[test]
fn a_disabled_one_is_never_held() {
    let mut test = test();
    *test
        .app
        .world_mut()
        .get_mut::<ButtonState>(test.button)
        .expect("a button") = ButtonState::Disabled;
    test.pointer(120.0, 120.0);
    test.mouse(true);
    assert!(!test.step());
    assert_eq!(test.presses(), 0);
}
