//! Give Item's picture of what it will give, rolled here from the same seed as the
//! server will roll it; how many it may give; and whether there is room for them.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::{Inventory, Item, StackLimits};
use shared::item_form::{ItemOrder, batch_seed, make_item};
use shared::protocol::LevelKind;

use super::{GiveButton, GiveChoice, GiveCount, GiveForm, GiveNoSpace, GivePreview};
use crate::common::{CurrentDungeon, GameView};
use crate::inventory::{Carried, Stacks};
use crate::widgets::{ButtonState, CELL, ItemLook, ItemName, TextBox, spawn_item_cells};

/// Draws what is to be given, keeps the count to the most allowed, and greys what
/// can't be used: Give where it can't all be given, Different / Identical for one.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::debug_menu) fn show_give_preview(
    mut commands: Commands,
    mut form: ResMut<GiveForm>,
    carried: Res<Carried>,
    stacks: Res<Stacks>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut previews: Query<(Entity, &mut GivePreview, &mut Node)>,
    mut counts: Query<&mut TextBox, With<GiveCount>>,
    mut no_space: Query<&mut Visibility, With<GiveNoSpace>>,
    mut buttons: Query<
        (&mut ButtonState, Option<&GiveChoice>),
        Or<(With<GiveButton>, With<GiveChoice>)>,
    >,
) {
    let Ok((entity, mut preview, mut node)) = previews.single_mut() else {
        return;
    };
    let fresh = preview.drawn.is_none();
    let moved = view.is_changed() || dungeon.is_changed();
    if !(fresh || moved || form.is_changed() || carried.is_changed() || stacks.is_changed()) {
        return;
    }
    let (order, seed) = (form.order(), form.seed.unwrap_or_default());
    let wearer = carried.0.creature;
    let item = one_of(&order, wearer, seed, 0, form.identical);
    let most = match form.identical {
        true => stacks.0.most_of(&item),
        false => stacks.0.most,
    };
    if form.count > most.max(1) {
        form.count = most.max(1);
        for mut count in &mut counts {
            count.text = form.count.to_string();
        }
    }
    if preview.drawn != Some((order, seed)) {
        preview.drawn = Some((order, seed));
        draw(&mut commands, entity, &mut node, &item);
    }
    let has_ground = *view.get() == GameView::Dungeon
        && (dungeon.info.as_ref()).is_some_and(|info| !matches!(info.kind, LevelKind::Hall { .. }));
    let room = has_ground
        || all_fit(
            &carried.0,
            stacks.0,
            &order,
            seed,
            form.count,
            form.identical,
        );
    for mut shown in &mut no_space {
        shown.set_if_neq(if room {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for (mut state, choice) in &mut buttons {
        let usable = match choice {
            Some(GiveChoice::Identical) => form.count > 1,
            Some(_) => continue,
            None => room && form.seed.is_some(),
        };
        state.set_if_neq(if usable {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}

/// The `n`th of a batch, as the server will make it; its ids only stand in.
fn one_of(order: &ItemOrder, wearer: CreatureKind, seed: u64, n: u32, identical: bool) -> Item {
    let mut next = (n as u128 + 1) << 32;
    let mut new_id = || {
        next += 1;
        Id::for_test(next)
    };
    let id = new_id();
    let seed = batch_seed(seed, if identical { 0 } else { n });
    make_item(order, wearer, id, seed, None, &mut new_id)
}

/// Whether all of them would fit in what is carried, as the server will try it.
fn all_fit(
    carried: &Inventory,
    limits: StackLimits,
    order: &ItemOrder,
    seed: u64,
    count: u32,
    identical: bool,
) -> bool {
    let mut inventory = carried.clone();
    (0..count).all(|n| {
        let item = one_of(order, carried.creature, seed, n, identical);
        inventory.give(item, limits).is_ok()
    })
}

fn draw(commands: &mut Commands, entity: Entity, node: &mut Node, item: &Item) {
    let shape = item.shape();
    node.width = px(shape.width as f32 * CELL);
    node.height = px(shape.height as f32 * CELL);
    let name = ItemName {
        words: item.name(),
        quality: item.quality,
        set: item.set,
    };
    commands.entity(entity).despawn_related::<Children>();
    commands.entity(entity).with_children(|parent| {
        let look = ItemLook::of(item);
        spawn_item_cells(parent, look, shape, (0.0, 0.0), false, |_, _| name.clone());
    });
}
