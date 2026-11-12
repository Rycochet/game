//! Developer tools' Inspect Mob: a mob as the server knows it, in lines of words.

use shared::creature::CreatureKind;
use shared::protocol::CharacterId;

use super::Mob;

/// What the server knows of `mob`, as seen by a player of `kind` playing
/// `character`, at action tick `tick`.
pub(super) fn describe(
    mob: &Mob,
    kind: CreatureKind,
    character: Option<&CharacterId>,
    tick: u64,
    frozen: bool,
) -> Vec<String> {
    let mut lines = vec![
        format!("{} ({:?})", mob.kind.def().name, mob.id),
        format!(
            "At {}, {}; home {}, {} on level {}",
            mob.position.0, mob.position.1, mob.home.0, mob.home.1, mob.home_level
        ),
        format!("Doing: {:?}", mob.doing),
        format!(
            "Target: {}",
            mob.target.map_or("none".into(), |id| format!("{id:?}"))
        ),
        format!("Health: {:.1}", mob.health(tick).as_f32()),
        format!("Attributes: {:?}", mob.attributes),
        format!("Feels: {:?} towards you", mob.feeling(kind, character)),
    ];
    if let Some(character) = character {
        let opinion = mob.memory.opinion(character) as f64 / 1000.0;
        lines.push(format!("Opinion of you: {opinion:.1} Health"));
        if mob.memory.provoked(character) {
            lines.push("Killing it would be self-defence".into());
        }
    }
    lines.push(match mob.alignment {
        Some(alignment) => format!(
            "Alignment: {} ({})",
            alignment.value(),
            alignment.band().name()
        ),
        None => "Alignment: none".into(),
    });
    lines.push(format!("Speed: {}; scent {}%", mob.speed, mob.scent));
    lines.push(format!("Wields: {:?}", mob.wielded));
    lines.push(format!("Wears: {} things", mob.shown.len()));
    let mut notes = Vec::new();
    if mob.still {
        notes.push("Still");
    }
    if mob.is_dev_made() {
        notes.push("Made by dev tools");
    }
    if frozen {
        notes.push("Frozen");
    }
    if !notes.is_empty() {
        lines.push(notes.join(", "));
    }
    lines
}
