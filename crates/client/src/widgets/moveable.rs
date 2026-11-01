//! Panels that fit their contents and can be moved by their title bars, and
//! the taking of clicks and touches for widgets.

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use super::{
    ButtonPressed, ButtonState, INK, Pressable, RaisePanel, panel, panel_title, raised_border,
    spawn_close_button,
};
use crate::common::{PointerPosition, TouchGesture};

/// Above the full map (20 to 25); a panel clicked on goes above the others.
const FIRST_PANEL_Z: i32 = 30;

const EDGE: f32 = 3.0;
const PADDING: f32 = 6.0;
const CONTENT_GAP: f32 = 4.0;
const TITLE_GAP: f32 = 8.0;
/// The stripes either side of the title: how many, how thick, the gap between,
/// and the least each side may shrink to.
const STRIPES: usize = 3;
const STRIPE_THICKNESS: f32 = 3.0;
const STRIPE_GAP: f32 = 3.0;
const MIN_STRIPES_WIDTH: f32 = 16.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;

/// A panel that can be moved. `at` is where its top-left corner is wanted, in
/// logical pixels; it is kept on the screen.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct MoveablePanel {
    at: Vec2,
}

/// What a click on this node does.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PanelPart {
    /// Only stops the click going through the panel.
    Body(Entity),
    TitleBar(Entity),
    Close(Entity),
}

/// A moveable panel with this title, at this place. Returns the panel, and the
/// entity to put its contents in; the panel fits them. It is despawned when
/// closed.
pub fn spawn_moveable_panel(commands: &mut Commands, title: &str, at: Vec2) -> (Entity, Entity) {
    let root = commands
        .spawn((
            panel(),
            MoveablePanel { at },
            GlobalZIndex(FIRST_PANEL_Z),
            raised_border(),
        ))
        .id();
    commands.entity(root).insert((
        PanelPart::Body(root),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(EDGE)),
            padding: UiRect::all(px(PADDING)),
            row_gap: px(CONTENT_GAP),
            ..default()
        },
    ));
    let content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(CONTENT_GAP),
            ..default()
        })
        .id();
    let bar = commands
        .spawn((
            PanelPart::TitleBar(root),
            Node {
                align_items: AlignItems::Center,
                column_gap: px(TITLE_GAP),
                ..default()
            },
        ))
        .with_children(|bar| {
            spawn_stripes(bar);
            bar.spawn(panel_title(title));
            spawn_stripes(bar);
            bar.spawn((
                PanelPart::Close(root),
                Node {
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        })
        .id();
    commands.entity(root).add_children(&[bar, content]);
    (root, content)
}

/// Put this on a node for presses on it to go no further. `entity` is the node.
pub fn stops_presses(entity: Entity) -> impl Bundle {
    PanelPart::Body(entity)
}

/// Lines running from the title out to the edge of the panel.
fn spawn_stripes(bar: &mut ChildSpawnerCommands) {
    bar.spawn(Node {
        flex_grow: 1.0,
        min_width: px(MIN_STRIPES_WIDTH),
        flex_direction: FlexDirection::Column,
        row_gap: px(STRIPE_GAP),
        ..default()
    })
    .with_children(|stripes| {
        for _ in 0..STRIPES {
            stripes.spawn((
                Node {
                    height: px(STRIPE_THICKNESS),
                    ..default()
                },
                BackgroundColor(INK),
            ));
        }
    });
}

/// Where a panel of this size goes, wanted at `at`: wholly on the screen if it
/// fits, otherwise with its top left on the screen, for the title bar.
fn on_screen(at: Vec2, size: Vec2, window: Vec2) -> Vec2 {
    at.min(window - size).max(Vec2::ZERO)
}

/// Puts panels where they are wanted, as far as the window allows.
pub(super) fn place_panels(
    window: Single<&Window, With<PrimaryWindow>>,
    mut panels: Query<(&mut MoveablePanel, &ComputedNode, &mut Node)>,
) {
    let window = Vec2::new(window.width(), window.height());
    for (mut moveable, computed, mut node) in &mut panels {
        let size = computed.size() * computed.inverse_scale_factor;
        let at = on_screen(moveable.at, size, window);
        if moveable.at != at {
            moveable.at = at;
        }
        let (left, top) = (px(at.x), px(at.y));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
    }
}

/// What is being moved, between frames.
#[derive(Resource, Default)]
pub(super) struct Dragging {
    /// By the mouse: which panel.
    mouse: Option<Entity>,
    /// The last place the pointer was, while dragging by the mouse.
    from: Option<Vec2>,
    /// Where the finger now dragging landed, and the panel its title bar was
    /// there, if it was.
    touch: Option<(Vec2, Option<Entity>)>,
    /// The highest panel so far.
    top_z: i32,
}

/// What this takes from the mouse: a press on a widget, and it is gone, so
/// nothing under the widget sees it. A tap arrives as a press (see `touch`), so
/// buttons work by touch; moving a panel by touch comes from `TouchGesture`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn take_presses(
    mut commands: Commands,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pointer: Res<PointerPosition>,
    mut gesture: ResMut<TouchGesture>,
    parts: Query<(
        &ComputedStackIndex,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        &PanelPart,
    )>,
    buttons: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Entity,
            Option<&ButtonState>,
        ),
        With<Pressable>,
    >,
    mut raised: MessageReader<RaisePanel>,
    mut panels: Query<(&mut MoveablePanel, &mut GlobalZIndex)>,
    mut pressed: MessageWriter<ButtonPressed>,
    mut dragging: ResMut<Dragging>,
) {
    let dragging = &mut *dragging;
    // The part in front at a point, and how far in front it is.
    let stacked_part_at = |at: Vec2| {
        parts
            .iter()
            .filter(|(_, node, transform, visible, ..)| {
                visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
            })
            .max_by_key(|(stack, ..)| stack.0)
            .map(|(stack, .., part)| (stack.0, *part))
    };
    let part_at = |at: Vec2| stacked_part_at(at).map(|(_, part)| part);
    let mut raise = |panel: Entity, panels: &mut Query<(&mut MoveablePanel, &mut GlobalZIndex)>| {
        if let Ok((_, mut z)) = panels.get_mut(panel) {
            dragging.top_z = dragging.top_z.max(FIRST_PANEL_Z) + 1;
            z.0 = dragging.top_z;
        }
    };
    for panel in raised.read() {
        raise(panel.0, &mut panels);
    }

    if mouse.just_pressed(MouseButton::Left) {
        let button = pointer.0.and_then(|at| {
            buttons
                .iter()
                .filter(|(_, node, transform, visible, ..)| {
                    visible.get()
                        && node.contains_point(**transform, at / node.inverse_scale_factor)
                })
                .max_by_key(|(stack, ..)| stack.0)
        });
        let stacked = pointer.0.and_then(stacked_part_at);
        // Only what is in front: a button hidden by another panel gets nothing.
        let button = button.filter(|(stack, ..)| stacked.is_none_or(|(part, _)| stack.0 > part));
        let part = stacked.map(|(_, part)| part);
        if let Some((.., entity, state)) = button {
            mouse.clear_just_pressed(MouseButton::Left);
            if !state.is_some_and(|state| state.is_disabled()) {
                pressed.write(ButtonPressed(entity));
            }
        } else if part.is_some() {
            mouse.clear_just_pressed(MouseButton::Left);
        }
        // A press on a button in a panel is a press on the panel, but its
        // title bar and close button don't have buttons on them.
        match part {
            Some(PanelPart::Close(panel)) => {
                commands.entity(panel).despawn();
            }
            Some(PanelPart::TitleBar(panel)) => {
                raise(panel, &mut panels);
                dragging.mouse = Some(panel);
                dragging.from = pointer.0;
            }
            Some(PanelPart::Body(panel)) => raise(panel, &mut panels),
            None => {}
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        dragging.mouse = None;
    }
    if let (Some(panel), Some(from), Some(to)) = (dragging.mouse, dragging.from, pointer.0) {
        dragging.from = Some(to);
        if let Ok((mut moveable, _)) = panels.get_mut(panel) {
            moveable.at += to - from;
        }
    }

    if gesture.drag != Vec2::ZERO {
        let start = gesture.drag_start;
        if dragging.touch.is_none_or(|(known, _)| known != start) {
            dragging.touch = Some((
                start,
                match part_at(start) {
                    Some(PanelPart::TitleBar(panel)) => Some(panel),
                    Some(_) => Some(Entity::PLACEHOLDER),
                    None => None,
                },
            ));
        }
        if let Some((_, Some(panel))) = dragging.touch {
            if let Ok((mut moveable, _)) = panels.get_mut(panel) {
                moveable.at += gesture.drag;
            }
            gesture.drag = Vec2::ZERO;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::math::Affine2;

    use super::*;

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

    /// A node on the screen, as the layout would leave it.
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
        root: Entity,
        button: Entity,
    }

    /// A panel at (100, 100) with a title bar along its top, a close button
    /// at the bar's right and a button below.
    fn test() -> Test {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<PointerPosition>()
            .init_resource::<TouchGesture>()
            .init_resource::<Dragging>()
            .add_message::<ButtonPressed>()
            .add_message::<RaisePanel>();
        let world = app.world_mut();
        let root = world
            .spawn((
                MoveablePanel {
                    at: Vec2::new(100.0, 100.0),
                },
                GlobalZIndex(FIRST_PANEL_Z),
            ))
            .id();
        world.entity_mut(root).insert((
            PanelPart::Body(root),
            laid_out(Vec2::new(100.0, 100.0), Vec2::new(200.0, 100.0), 0),
        ));
        world.spawn((
            PanelPart::TitleBar(root),
            laid_out(Vec2::new(100.0, 100.0), Vec2::new(160.0, 30.0), 1),
        ));
        world.spawn((
            PanelPart::Close(root),
            laid_out(Vec2::new(260.0, 100.0), Vec2::new(40.0, 30.0), 1),
        ));
        let button = world
            .spawn((
                Pressable,
                ButtonState::Normal,
                laid_out(Vec2::new(110.0, 150.0), Vec2::new(80.0, 30.0), 1),
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
            self.app.world_mut().resource_mut::<PointerPosition>().0 = Some(Vec2::new(x, y));
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
            laid_out(Vec2::new(110.0, 150.0), Vec2::new(80.0, 30.0), 5),
        ));
        test.click(120.0, 160.0);
        assert!(!test.still_pressed());
        assert!(test.presses().is_empty());
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
        test.app.world_mut().resource_mut::<PointerPosition>().0 = Some(Vec2::new(150.0, 130.0));
        test.step();
        assert_eq!(test.panel_at(), Vec2::new(130.0, 120.0));

        test.app
            .world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        test.app.world_mut().resource_mut::<PointerPosition>().0 = Some(Vec2::new(300.0, 300.0));
        test.step();
        assert_eq!(test.panel_at(), Vec2::new(130.0, 120.0));
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
}
