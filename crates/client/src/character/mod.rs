//! What the character is like: the Character panel's Character tab, which
//! `inventory` opens, and how fast they go. See `docs/context/client/character.md`.

use bevy::prelude::*;
use shared::energy::NORMAL_SPEED;
use shared::item::Inventory;
use shared::movement::Gait;
use shared::protocol::ServerMessage;
use shared::sheet::{Attributes, Sheet};
use shared::vitals::VitalNow;

use crate::common::{CarryLimit, OwnVitals, PlayerGait, PlayerSize, PlayerSpeed};
use crate::connection::FromServer;
use crate::dom::Dom;
use crate::inventory::Carried;
use crate::widgets::UiId;

mod attribute_lines;
mod listed;
mod vitals_lines;

pub use listed::describe_listed;

use attribute_lines::attribute_lines;
use vitals_lines::vitals_lines;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSheet>()
            .init_resource::<WithBonuses>()
            .add_systems(
                Update,
                (
                    read_server,
                    publish_speed,
                    publish_field,
                    publish_limit,
                    show_words,
                )
                    .chain(),
            );
    }
}

/// What the character is like, as the server says.
#[derive(Resource, Default)]
pub struct CharacterSheet(pub Sheet);

/// The character's attributes with every bonus, as the server last said
/// (`ServerMessage::Attributes`): `None` until it has.
#[derive(Resource, Default)]
pub struct WithBonuses(pub Option<Attributes>);

fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut sheet: ResMut<CharacterSheet>,
    mut with: ResMut<WithBonuses>,
) {
    for message in from_server.read() {
        match &message.0 {
            ServerMessage::Sheet(new) => sheet.0 = new.clone(),
            ServerMessage::Attributes(new) => with.0 = Some(*new),
            _ => {}
        }
    }
}

/// What the player sees round them, by kind and Observation (`PlayerField`).
fn publish_field(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut field: ResMut<crate::common::PlayerField>,
) {
    use shared::facing::{Field, observation_skill};
    let observation = sheet.0.skills.value(observation_skill());
    let now = Field::of(carried.0.creature, observation);
    field.set_if_neq(crate::common::PlayerField(now));
}

/// What the player's speed and gait come to, for moving them.
fn publish_speed(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut speed: ResMut<PlayerSpeed>,
    mut gait: ResMut<PlayerGait>,
) {
    speed.set_if_neq(PlayerSpeed(sheet.0.speed(&carried.0)));
    gait.set_if_neq(PlayerGait(Gait::of(&sheet.0.skills)));
}

/// The most the player can carry, for the weight under the inventory.
/// And how big they are, for moving them and their token.
fn publish_limit(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut limit: ResMut<CarryLimit>,
    mut player_size: ResMut<PlayerSize>,
) {
    let size = carried.0.creature.def().size;
    limit.set_if_neq(CarryLimit(sheet.0.capacity(size)));
    player_size.set_if_neq(PlayerSize(size));
}

/// What the panel says.
fn describe(
    (sheet, with): (&Sheet, Option<&Attributes>),
    carried: &Inventory,
    most: Option<(VitalNow, VitalNow)>,
) -> String {
    let def = carried.creature.def();
    let mut lines = vec![
        format!("{} {}, {:?}", sheet.gender.name(), def.name, def.size),
        String::new(),
    ];
    lines.extend(attribute_lines(&sheet.attributes, with));
    lines.push(String::new());
    lines.extend(vitals_lines(sheet, &carried.worn, most));
    lines.extend([
        // How much is carried now is shown under the inventory instead.
        format!(
            "Can carry {:.2} kg",
            sheet.capacity(def.size) as f32 / 1000.0
        ),
        format!("Speed {}%", sheet.speed(carried) * 100 / NORMAL_SPEED),
    ]);
    lines.push(String::new());
    for &which in def.looks {
        lines.push(format!(
            "{}: {}",
            which.name(),
            sheet.appearance.says(carried.creature, which)
        ));
    }
    lines.join("\n")
}

/// Writes the sheet (`inventory/sheet`, in the Character panel's markup) when
/// what it says changes, or the panel opens.
fn show_words(
    (sheet, with): (Res<CharacterSheet>, Res<WithBonuses>),
    carried: Res<Carried>,
    own: Res<OwnVitals>,
    ids: Query<&UiId>,
    new: Query<&UiId, Added<UiId>>,
    mut dom: Dom,
    mut said: Local<String>,
) {
    let id = UiId::new("inventory", "sheet");
    let most = own.0.map(|told| (told.health, told.mana));
    let words = describe((&sheet.0, with.0.as_ref()), &carried.0, most);
    let opened = new.iter().any(|new| *new == id);
    if !opened && *said == words {
        return;
    }
    said.clone_from(&words);
    // (Closed: it is written when it opens.)
    if ids.iter().any(|built| *built == id) {
        dom.set_text("inventory/sheet", words);
    }
}

#[cfg(test)]
mod tests;
