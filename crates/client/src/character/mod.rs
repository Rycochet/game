//! What the character is like: the left of the Character panel, which `inventory`
//! opens, and how fast they go. See `docs/context/client/character.md`.

use bevy::prelude::*;
use shared::energy::NORMAL_SPEED;
use shared::item::Inventory;
use shared::protocol::ServerMessage;
use shared::sheet::Sheet;

use crate::common::{PlayerSpeed, SheetText};
use crate::connection::FromServer;
use crate::inventory::Carried;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSheet>()
            .add_systems(Update, (read_server, publish_speed, show_words).chain());
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

/// What the panel says.
fn describe(sheet: &Sheet, carried: &Inventory) -> String {
    let def = carried.creature.def();
    let stats = sheet.stats;
    let mut lines = vec![
        format!("{}, {:?}", def.name, def.size),
        String::new(),
        format!("Strength {}", stats.strength),
        format!("Dexterity {}", stats.dexterity),
        format!("Intelligence {}", stats.intelligence),
        format!("Constitution {}", stats.constitution),
        String::new(),
        format!(
            "Carrying {:.2} of {:.2} kg: {:?}",
            carried.total_weight() as f32 / 1000.0,
            sheet.capacity(def.size) as f32 / 1000.0,
            sheet.load(carried),
        ),
        format!("Speed {}%", sheet.speed(carried) * 100 / NORMAL_SPEED),
    ];
    if !sheet.skills.is_empty() {
        lines.push(String::new());
        for (skill, level) in &sheet.skills {
            lines.push(format!("{skill:?} {level}"));
        }
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
            sheet.appearance.says(which)
        ));
    }
    lines.join("\n")
}

fn show_words(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut texts: Query<&mut Text, With<SheetText>>,
) {
    for mut text in &mut texts {
        let words = describe(&sheet.0, &carried.0);
        if text.0 != words {
            text.0 = words;
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::creature::CreatureKind;

    use super::*;

    #[test]
    fn it_says_what_the_kind_is_and_how_loaded() {
        let carried = Inventory {
            creature: CreatureKind::Kobold,
            ..Inventory::default()
        };
        let words = describe(&Sheet::new(CreatureKind::Kobold), &carried);
        assert!(words.starts_with("Kobold, Small"), "{words}");
        assert!(words.contains("Strength 7"));
        assert!(words.contains("Stealth 3"));
        assert!(words.contains("NightVision"));
        assert!(words.contains("Light"));
        assert!(words.contains("Speed 100%"));
        assert!(words.contains("Height: Average"));
    }
}
