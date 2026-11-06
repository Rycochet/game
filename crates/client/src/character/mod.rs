//! What the character is like: the left of the Character panel, which `inventory`
//! opens, and how fast they go. See `docs/context/client/character.md`.

use bevy::prelude::*;
use shared::energy::NORMAL_SPEED;
use shared::item::Inventory;
use shared::protocol::ServerMessage;
use shared::sheet::{Row, Sheet};
use shared::vitals::VitalNow;

use crate::common::{CarryLimit, OwnVitals, PlayerSpeed, SheetText};
use crate::connection::FromServer;
use crate::inventory::Carried;

mod skill_lines;
mod vitals_lines;

use skill_lines::skill_lines;
use vitals_lines::vitals_lines;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSheet>().add_systems(
            Update,
            (read_server, publish_speed, publish_limit, show_words).chain(),
        );
    }
}

/// What the character is like, as the server says.
#[derive(Resource, Default)]
pub struct CharacterSheet(pub Sheet);

fn read_server(mut from_server: MessageReader<FromServer>, mut sheet: ResMut<CharacterSheet>) {
    for message in from_server.read() {
        if let ServerMessage::Sheet(new) = &message.0 {
            sheet.0 = new.clone();
        }
    }
}

/// What the player's speed comes to, for moving them.
fn publish_speed(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut speed: ResMut<PlayerSpeed>,
) {
    speed.set_if_neq(PlayerSpeed(sheet.0.speed(&carried.0)));
}

/// The most the player can carry, for the weight under the inventory.
fn publish_limit(sheet: Res<CharacterSheet>, carried: Res<Carried>, mut limit: ResMut<CarryLimit>) {
    let size = carried.0.creature.def().size;
    limit.set_if_neq(CarryLimit(sheet.0.capacity(size)));
}

/// What the panel says.
fn describe(sheet: &Sheet, carried: &Inventory, most: Option<(VitalNow, VitalNow)>) -> String {
    let def = carried.creature.def();
    let mut lines = vec![format!("{}, {:?}", def.name, def.size), String::new()];
    // The nine attributes, a row to a line.
    for row in Row::ALL {
        let values: Vec<_> = row
            .attributes()
            .iter()
            .map(|&which| format!("{} {}", which.name(), sheet.attributes.get(which)))
            .collect();
        lines.push(format!("{}: {}", row.name(), values.join(", ")));
    }
    lines.push(String::new());
    lines.extend(vitals_lines(sheet, most));
    lines.extend([
        // How much is carried now is shown under the inventory instead.
        format!(
            "Can carry {:.2} kg",
            sheet.capacity(def.size) as f32 / 1000.0
        ),
        format!("Speed {}%", sheet.speed(carried) * 100 / NORMAL_SPEED),
    ]);
    if !sheet.skills.is_empty() {
        lines.push(String::new());
        lines.extend(skill_lines(&sheet.skills));
    }
    if !def.abilities.is_empty() {
        lines.push(String::new());
        let abilities: Vec<_> = def.abilities.iter().map(|a| format!("{a:?}")).collect();
        lines.push(abilities.join(", "));
    }
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

fn show_words(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    own: Res<OwnVitals>,
    mut texts: Query<&mut Text, With<SheetText>>,
) {
    let most = own.0.map(|told| (told.health, told.mana));
    for mut text in &mut texts {
        let words = describe(&sheet.0, &carried.0, most);
        if text.0 != words {
            text.0 = words;
        }
    }
}

#[cfg(test)]
mod tests;
