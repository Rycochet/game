//! What the Character tab would say of a character offered to choose from, as far
//! as their save tells (`ListedCharacter`): no Health and Mana, which the server
//! works out on joining, nor speed, which goes by what is carried.

use shared::protocol::ListedCharacter;
use shared::sheet::Row;

/// The kind, the nine attributes a row to a line, what they can carry, their
/// looks and difficulty.
pub fn describe_listed(listed: &ListedCharacter) -> String {
    let (sheet, kind) = (&listed.sheet, listed.kind);
    let def = kind.def();
    let mut lines = vec![
        format!("{} {}, {:?}", sheet.gender.name(), def.name, def.size),
        String::new(),
    ];
    for row in Row::ALL {
        let values: Vec<_> = row
            .attributes()
            .iter()
            .map(|&which| format!("{} {}", which.name(), sheet.attributes.get(which)))
            .collect();
        lines.push(format!("{}: {}", row.name(), values.join(", ")));
    }
    lines.push(String::new());
    lines.push(format!(
        "Can carry {:.2} kg",
        sheet.capacity(def.size) as f32 / 1000.0
    ));
    if let Some(difficulty) = listed.difficulty {
        lines.push(format!("Difficulty: {}", difficulty.name()));
    }
    lines.push(String::new());
    for &which in def.looks {
        lines.push(format!(
            "{}: {}",
            which.name(),
            sheet.appearance.says(kind, which)
        ));
    }
    lines.join("\n")
}
