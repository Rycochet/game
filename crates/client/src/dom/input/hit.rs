//! What the pointer is on. One search a frame for the whole interface, which
//! everything reads, so two things can never both think they are in front.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::dispatch::Buttons;
use crate::common::UiPointer;
use crate::dom::parts::{Element, Inert, Solid};

/// What the pointer is on this frame (`PreUpdate`, `DomInput::Find`).
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct Hit {
    /// The thing in front at the pointer: an element, or a `Solid` node.
    pub target: Option<Entity>,
    /// The target, then everything it is in, out to the root.
    pub chain: Vec<Entity>,
    /// Where the pointer is, in interface pixels.
    pub at: Option<Vec2>,
}

impl Hit {
    /// The first of the target and what holds it, going outwards, that
    /// `pick` takes: how a press on a label in a button reaches the button.
    pub fn first<T>(&self, pick: impl FnMut(Entity) -> Option<T>) -> Option<T> {
        self.chain.iter().copied().find_map(pick)
    }
}

/// The element that has taken the pointer until the button is let go (a
/// slider's handle, a panel's title): while set, it is the target wherever
/// the pointer goes, so nothing else is pointed at on the way.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerCapture(pub Option<Entity>);

type Laid = (
    &'static ComputedNode,
    &'static UiGlobalTransform,
    &'static InheritedVisibility,
    &'static ComputedStackIndex,
    Option<&'static CalculatedClip>,
    Option<&'static Children>,
    Has<Solid>,
    Has<Inert>,
    Has<Element>,
);

/// Finds what is in front at a point.
#[derive(SystemParam)]
pub struct Finder<'w, 's> {
    nodes: Query<'w, 's, Laid>,
    roots: Query<'w, 's, Entity, (With<ComputedNode>, Without<ChildOf>)>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl Finder<'_, '_> {
    /// The thing in front at `at` (interface pixels), if the pointer is on
    /// the interface at all.
    pub fn target_at(&self, at: Vec2) -> Option<Entity> {
        let found = self.roots.iter().filter_map(|root| self.find(root, at));
        found
            .max_by_key(|(stack, _)| *stack)
            .map(|(_, entity)| entity)
    }

    /// `target`, then everything it is in, out to the root.
    pub fn chain_of(&self, target: Entity) -> Vec<Entity> {
        let mut chain = vec![target];
        chain.extend(self.parents.iter_ancestors(target));
        chain
    }

    /// The thing in front at `at` in `entity` and what it holds, with how
    /// far in front it is.
    fn find(&self, entity: Entity, at: Vec2) -> Option<(u32, Entity)> {
        let (node, transform, visible, stack, clip, children, solid, inert, element) =
            self.nodes.get(entity).ok()?;
        if !visible.get() || inert {
            return None;
        }
        let physical = at / node.inverse_scale_factor;
        let on = node.contains_point(*transform, physical)
            && clip.is_none_or(|clip| clip.contains_point(physical));
        // Nothing in a document reaches outside what holds it, so a point
        // that isn't on an element isn't on anything in it: the search
        // drills down, never looking into what the pointer is off.
        if element && !on {
            return None;
        }
        let own = (on && (solid || element)).then_some((stack.0, entity));
        let mut inside: Vec<Entity> = children.map(|held| held.to_vec()).unwrap_or_default();
        if !element {
            // Other nodes may hold things placed anywhere, so all are tried.
            let found = inside.iter().filter_map(|child| self.find(*child, at));
            return found.chain(own).max_by_key(|(stack, _)| *stack);
        }
        // In front first, so the first found is the one in front.
        let depth = |child: &Entity| self.nodes.get(*child).map_or(0, |laid| laid.3.0);
        inside.sort_by_key(|child| std::cmp::Reverse(depth(child)));
        let found = inside.iter().find_map(|child| self.find(*child, at));
        found.or(own)
    }
}

/// Works out `Hit`, and notes the mouse's buttons before anything takes them.
pub(super) fn find_target(
    pointer: Res<UiPointer>,
    finder: Finder,
    mouse: Res<ButtonInput<MouseButton>>,
    capture: Res<PointerCapture>,
    mut buttons: ResMut<Buttons>,
    mut hit: ResMut<Hit>,
) {
    *buttons = Buttons::of(&mouse);
    let captured = capture.0.filter(|taken| finder.nodes.contains(*taken));
    let target = captured.or_else(|| pointer.0.and_then(|at| finder.target_at(at)));
    let found = Hit {
        target,
        chain: target
            .map(|target| finder.chain_of(target))
            .unwrap_or_default(),
        at: pointer.0,
    };
    hit.set_if_neq(found);
}

/// For tests of what reads `Hit`: works it out now, as `find_target` would.
#[cfg(test)]
pub fn find_now(world: &mut World) {
    use bevy::ecs::system::RunSystemOnce;
    world.init_resource::<Hit>();
    world.init_resource::<PointerCapture>();
    world.init_resource::<Buttons>();
    world.run_system_once(find_target).expect("the search runs");
}

#[cfg(test)]
pub(in crate::dom) mod tests;
