//! Dev Area's and Create Mob's choices kept between runs (`Preferences`), so
//! their panels open as last left, and Regenerate Dev Area, which makes the last
//! area again without opening its panel.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, DevMob};
use shared::site::{DEV_AREA_SIZES, DevLayout};

use super::DevAreaForm;
use super::create_mob::ATTITUDES;
use crate::common::{CreateMobOrder, CreateMobPrefs, DevAreaPrefs, Preferences};
use crate::connection::ToServer;
use crate::debug_menu::DebugAction;

/// Adds the systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_kept_choices(app: &mut App) {
    app.add_systems(Update, (load_choices, save_choices, regenerate).chain());
}

/// The first frame: the panels' choices as last kept.
fn load_choices(
    preferences: Res<Preferences>,
    mut form: ResMut<DevAreaForm>,
    mut order: ResMut<CreateMobOrder>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    *loaded = true;
    if let Some(kept) = &preferences.dev_area {
        *form = dev_area_form(kept, form.seeds_made);
    }
    if let Some(kept) = &preferences.create_mob {
        order.0 = mob_order(kept, order.0);
    }
}

/// Keeps the choices whenever they change.
fn save_choices(
    form: Res<DevAreaForm>,
    order: Res<CreateMobOrder>,
    mut preferences: ResMut<Preferences>,
) {
    if form.is_changed() {
        let kept = Some(dev_area_prefs(&form));
        if preferences.dev_area != kept {
            preferences.dev_area = kept;
        }
    }
    if order.is_changed() {
        let kept = Some(mob_prefs(&order.0));
        if preferences.create_mob != kept {
            preferences.create_mob = kept;
        }
    }
}

/// Regenerate Dev Area: the panel's area, made again as new.
fn regenerate(
    mut actions: MessageReader<DebugAction>,
    form: Res<DevAreaForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &action in actions.read() {
        if action == DebugAction::RegenerateDevArea {
            to_server.write(ToServer(ClientMessage::DevGenerateArea(form.area())));
        }
    }
}

/// A kind of creature by its name.
fn kind_named(name: &str) -> Option<CreatureKind> {
    CreatureKind::ALL
        .into_iter()
        .find(|kind| kind.def().name.eq_ignore_ascii_case(name))
}

pub(super) fn dev_area_prefs(form: &DevAreaForm) -> DevAreaPrefs {
    let mobs = match form.mobs {
        None => "Default".to_owned(),
        Some(None) => "None".to_owned(),
        Some(Some(kind)) => kind.def().name.to_owned(),
    };
    DevAreaPrefs {
        size: form.size,
        generator: form.layout.name(),
        mobs,
        count: form.count,
        seed: form.seed.clone(),
    }
}

/// The form as kept; anything no longer known as the panel starts.
pub(super) fn dev_area_form(kept: &DevAreaPrefs, seeds_made: i64) -> DevAreaForm {
    let start = DevAreaForm::default();
    let mobs = match kept.mobs.as_str() {
        "None" => Some(None),
        name => kind_named(name).map(Some),
    };
    DevAreaForm {
        size: if DEV_AREA_SIZES.contains(&kept.size) {
            kept.size
        } else {
            start.size
        },
        layout: DevLayout::named(&kept.generator).unwrap_or(start.layout),
        mobs,
        count: kept.count.max(1),
        seed: kept.seed.clone(),
        seeds_made,
    }
}

pub(super) fn mob_prefs(order: &DevMob) -> CreateMobPrefs {
    let attitude = ATTITUDES
        .iter()
        .find(|&&(_, attitude)| attitude == order.attitude)
        .map_or("Default", |&(name, _)| name);
    CreateMobPrefs {
        kind: order.kind.def().name.to_owned(),
        attitude: attitude.to_owned(),
        gear: order.gear,
        still: order.still,
        seed: order.seed,
    }
}

/// The order as kept; anything no longer known as it was (`start`).
pub(super) fn mob_order(kept: &CreateMobPrefs, start: DevMob) -> DevMob {
    let attitude = ATTITUDES
        .iter()
        .find(|(name, _)| *name == kept.attitude)
        .map_or(start.attitude, |&(_, attitude)| attitude);
    DevMob {
        kind: kind_named(&kept.kind).unwrap_or(start.kind),
        attitude,
        gear: kept.gear,
        still: kept.still,
        seed: kept.seed,
        ..start
    }
}

#[cfg(test)]
mod tests;
