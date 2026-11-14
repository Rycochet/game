//! The Character panel's lines on Health and Mana: how much there can be of
//! each, and where it comes from.

use shared::item::Item;
use shared::sheet::Sheet;
use shared::vitals::{Vital, VitalNow};

/// A line for each, once the server has said how much there can be, with what
/// is `worn`. The client isn't told the playtime, so its part is what the rest
/// leaves (and anything about worn items it doesn't know).
pub(super) fn vitals_lines(
    sheet: &Sheet,
    worn: &[Item],
    most: Option<(VitalNow, VitalNow)>,
) -> Vec<String> {
    let Some((health, mana)) = most else {
        return Vec::new();
    };
    [(Vital::Health, health), (Vital::Mana, mana)]
        .into_iter()
        .map(|(vital, told)| {
            // Effects the server still keeps all count: it drops ended ones.
            let totals = sheet.totals(worn, 0);
            let attributes = totals.attributes(&sheet.attributes);
            let parts = sheet.vitals.most(vital, &attributes, &totals, 0);
            let rest =
                i64::from(parts.attribute) + i64::from(parts.practice) + i64::from(parts.bonus);
            let played = (i64::from(told.most) - rest).max(0);
            let mut sources = vec![format!("{} {}", vital.attribute().name(), parts.attribute)];
            for (amount, from) in [
                (played, "play"),
                (i64::from(parts.practice), "practice"),
                (i64::from(parts.bonus), "bonuses"),
            ] {
                if amount != 0 {
                    sources.push(format!("{amount:+} {from}"));
                }
            }
            format!("{} {} ({})", vital.name(), told.most, sources.join(", "))
        })
        .collect()
}
