use bevy::ecs::system::RunSystemOnce;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};

use super::super::{ButtonPressed, ButtonState, Pressable, RaisePanel};
use super::place::{in_new_window, on_screen};
use super::*;
use crate::common::{TouchGesture, UiPointer};

#[test]
fn panels_stay_on_the_screen() {
    let window = Vec2::new(800.0, 600.0);
    let size = Vec2::new(200.0, 100.0);
    let place = |x, y| on_screen(Vec2::new(x, y), size, window);
    assert_eq!(place(50.0, 60.0), Vec2::new(50.0, 60.0));
    assert_eq!(place(-50.0, -60.0), Vec2::ZERO);
    assert_eq!(place(790.0, 590.0), Vec2::new(600.0, 500.0));
    // Too big to fit: the title bar stays.
    let big = Vec2::new(1000.0, 700.0);
    assert_eq!(on_screen(Vec2::new(50.0, 50.0), big, window), Vec2::ZERO);
}

#[test]
fn panels_keep_their_share_of_a_changed_window() {
    let old = Vec2::new(800.0, 600.0);
    let at = Vec2::new(400.0, 150.0);
    let bigger = Vec2::new(1600.0, 1200.0);
    assert_eq!(in_new_window(at, old, bigger), Vec2::new(800.0, 300.0));
    assert_eq!(in_new_window(at, old, old), at);
}

/// A node on the screen, as the layout would leave it with the interface drawn
/// `scale` times its size (`UiScale`, in a window with no scaling of its own):
/// `corner` and `size` in the interface's pixels.
fn laid_out(corner: Vec2, size: Vec2, stack: u32, scale: f32) -> impl Bundle {
    (
        ComputedNode {
            size: size * scale,
            inverse_scale_factor: 1.0 / scale,
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation((corner + size / 2.0) * scale)),
        InheritedVisibility::VISIBLE,
        ComputedStackIndex(stack),
    )
}

struct Test {
    app: App,
    root: Entity,
    button: Entity,
}

/// A panel at (100, 100) with a title bar along its top, a close button
/// at the bar's right and a button below.
fn test() -> Test {
    test_at(1.0)
}

/// The same, with the interface drawn `scale` times its size.
fn test_at(scale: f32) -> Test {
    let mut app = App::new();
    app.init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<UiPointer>()
        .insert_resource(UiScale(scale))
        .init_resource::<TouchGesture>()
        .init_resource::<Dragging>()
        .add_message::<ButtonPressed>()
        .add_message::<RaisePanel>();
    let world = app.world_mut();
    let root = world
        .spawn((
            MoveablePanel {
                at: Vec2::new(100.0, 100.0),
                window: None,
            },
            GlobalZIndex(FIRST_PANEL_Z),
        ))
        .id();
    world.entity_mut(root).insert((
        PanelPart::Body(root),
        laid_out(Vec2::new(100.0, 100.0), Vec2::new(200.0, 100.0), 0, scale),
    ));
    world.spawn((
        PanelPart::TitleBar(root),
        laid_out(Vec2::new(100.0, 100.0), Vec2::new(160.0, 30.0), 1, scale),
    ));
    world.spawn((
        PanelPart::Close(root),
        laid_out(Vec2::new(260.0, 100.0), Vec2::new(40.0, 30.0), 1, scale),
    ));
    let button = world
        .spawn((
            Pressable,
            ButtonState::Normal,
            laid_out(Vec2::new(110.0, 150.0), Vec2::new(80.0, 30.0), 1, scale),
        ))
        .id();
    Test { app, root, button }
}

impl Test {
    fn step(&mut self) {
        self.app
            .world_mut()
            .run_system_once(take_presses)
            .expect("the system runs");
    }

    fn click(&mut self, x: f32, y: f32) {
        self.app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(x, y));
        let mut mouse = self
            .app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>();
        mouse.release(MouseButton::Left);
        mouse.clear();
        mouse.press(MouseButton::Left);
        self.step();
    }

    fn still_pressed(&self) -> bool {
        self.app
            .world()
            .resource::<ButtonInput<MouseButton>>()
            .just_pressed(MouseButton::Left)
    }

    fn panel_at(&self) -> Vec2 {
        self.app
            .world()
            .get::<MoveablePanel>(self.root)
            .expect("a panel")
            .at
    }

    fn presses(&mut self) -> Vec<Entity> {
        self.app
            .world_mut()
            .resource_mut::<Messages<ButtonPressed>>()
            .drain()
            .map(|pressed| pressed.0)
            .collect()
    }
}

#[test]
fn a_button_takes_its_click_unless_it_is_disabled_when_it_only_takes_it() {
    let mut test = test();
    test.click(120.0, 160.0);
    assert!(!test.still_pressed());
    assert_eq!(test.presses(), vec![test.button]);

    let button = test.button;
    *test
        .app
        .world_mut()
        .get_mut::<ButtonState>(button)
        .expect("a state") = ButtonState::Disabled;
    test.click(120.0, 160.0);
    assert!(!test.still_pressed());
    assert!(test.presses().is_empty());
}

#[test]
fn a_button_under_another_panel_gets_nothing() {
    let mut test = test();
    let root = test.root;
    // Another panel's body, in front of the button.
    test.app.world_mut().spawn((
        PanelPart::Body(root),
        laid_out(Vec2::new(110.0, 150.0), Vec2::new(80.0, 30.0), 5, 1.0),
    ));
    test.click(120.0, 160.0);
    assert!(!test.still_pressed());
    assert!(test.presses().is_empty());
}

#[test]
fn a_part_left_for_its_owner_hides_what_is_under_it_and_keeps_the_press() {
    let mut test = test();
    let root = test.root;
    test.app.world_mut().spawn((
        PanelPart::LeftForOwner(root),
        laid_out(Vec2::ZERO, Vec2::new(800.0, 600.0), 5, 1.0),
    ));
    // On the button, and on the title bar, which would have started a drag.
    test.click(120.0, 160.0);
    assert!(test.still_pressed());
    assert!(test.presses().is_empty());
    let start = Vec2::new(120.0, 110.0);
    test.click(start.x, start.y);
    assert!(test.still_pressed());
    // A finger's drag is left too.
    let drag = Vec2::new(5.0, 5.0);
    touch_drag(&mut test, start, drag);
    assert_eq!(test.app.world().resource::<TouchGesture>().drag, drag);
    assert_eq!(test.panel_at(), Vec2::new(100.0, 100.0));
}

#[test]
fn clicks_elsewhere_are_left_alone() {
    let mut test = test();
    test.click(20.0, 20.0);
    assert!(test.still_pressed());
}

#[test]
fn a_click_on_the_panel_stops_there() {
    let mut test = test();
    test.click(280.0, 180.0);
    assert!(!test.still_pressed());
}

#[test]
fn the_close_button_despawns_the_panel() {
    let mut test = test();
    test.click(280.0, 110.0);
    test.app.update();
    assert!(test.app.world().get_entity(test.root).is_err());
}

#[test]
fn the_mouse_drags_by_the_title_bar_until_let_go() {
    let mut test = test();
    test.click(120.0, 110.0);
    test.app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(150.0, 130.0));
    test.step();
    assert_eq!(test.panel_at(), Vec2::new(130.0, 120.0));

    test.app
        .world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    test.app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(300.0, 300.0));
    test.step();
    assert_eq!(test.panel_at(), Vec2::new(130.0, 120.0));
}

#[test]
fn presses_land_where_the_interface_is_drawn_at_any_size() {
    for scale in [0.5, 2.0] {
        let mut test = test_at(scale);
        // In the interface's pixels, as `UiPointer` gives them.
        test.click(120.0, 160.0);
        assert_eq!(test.presses(), vec![test.button], "at {scale}");
        test.click(120.0, 110.0);
        test.app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(150.0, 130.0));
        test.step();
        assert_eq!(test.panel_at(), Vec2::new(130.0, 120.0), "at {scale}");
    }
}

#[test]
fn a_finger_drags_a_panel_as_far_as_it_went_on_the_screen() {
    for scale in [0.5, 2.0] {
        let mut test = test_at(scale);
        // In window pixels, as `TouchGesture` gives them: on the title bar.
        touch_drag(
            &mut test,
            Vec2::new(120.0, 110.0) * scale,
            Vec2::new(40.0, 20.0),
        );
        assert_eq!(
            test.panel_at(),
            Vec2::new(100.0, 100.0) + Vec2::new(40.0, 20.0) / scale,
            "at {scale}"
        );
    }
}

fn touch_drag(test: &mut Test, start: Vec2, drag: Vec2) {
    *test.app.world_mut().resource_mut::<TouchGesture>() = TouchGesture {
        drag,
        drag_start: start,
        ..default()
    };
    test.step();
}

#[test]
fn a_finger_that_landed_on_the_title_bar_drags_even_once_off_it() {
    let mut test = test();
    let start = Vec2::new(120.0, 110.0);
    touch_drag(&mut test, start, Vec2::new(5.0, 300.0));
    touch_drag(&mut test, start, Vec2::new(5.0, 0.0));
    assert_eq!(test.panel_at(), Vec2::new(110.0, 400.0));
    assert_eq!(test.app.world().resource::<TouchGesture>().drag, Vec2::ZERO);
}

#[test]
fn a_finger_on_the_panel_drags_nothing_and_elsewhere_is_left_alone() {
    let mut test = test();
    touch_drag(&mut test, Vec2::new(280.0, 180.0), Vec2::new(5.0, 5.0));
    assert_eq!(test.panel_at(), Vec2::new(100.0, 100.0));
    assert_eq!(test.app.world().resource::<TouchGesture>().drag, Vec2::ZERO);

    let drag = Vec2::new(5.0, 5.0);
    touch_drag(&mut test, Vec2::new(20.0, 20.0), drag);
    assert_eq!(test.app.world().resource::<TouchGesture>().drag, drag);
}
