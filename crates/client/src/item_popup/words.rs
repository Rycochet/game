//! An item's lines in its popup: what it gives while worn or when eaten
//! (`shared::bonus`), each worded through the language files (`item.ftl`).
//! Only what the client's copy of the item knows goes in.

use shared::bonus::{Amount, Bonus, Effect, Stat, food_effect, item_bonuses};
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::item::Item;
use shared::lang::{Args, Lang, RenderError};
use shared::vitals::Vital;

/// One line for each bonus the item gives: worn ("+40 Toughness") or eaten
/// ("Restores 1 Health a second for 30 s; stops on moving or being hurt").
/// A stack's are its top's.
pub(super) fn bonus_lines(item: &Item, lang: &Lang) -> Vec<String> {
    let item = item.top();
    let worn = item_bonuses(item).into_iter();
    let worn = worn.filter(|bonus| amount_of(bonus) != 0);
    let mut lines: Vec<String> = worn.map(|bonus| bonus_line(&bonus, lang)).collect();
    if let Some(effect) = food_effect(item.kind, 0) {
        lines.extend(effect_lines(&effect, lang));
    }
    lines
}

fn amount_of(bonus: &Bonus) -> i32 {
    match bonus.amount {
        Amount::Flat(milli) | Amount::Percent(milli) => milli,
    }
}

/// An eaten thing's bonuses, each with how long it lasts and what ends it.
fn effect_lines(effect: &Effect, lang: &Lang) -> Vec<String> {
    let ends = match (effect.ends_on.moving, effect.ends_on.hurt) {
        (true, true) => "both",
        (true, false) => "moving",
        (false, true) => "hurt",
        (false, false) => "nothing",
    };
    let ticks = effect.end.map(|end| end.saturating_sub(effect.start));
    let bonuses = effect.bonuses.iter().filter(|bonus| amount_of(bonus) != 0);
    bonuses
        .map(|bonus| {
            let mut line = bonus_line(bonus, lang);
            if let Some(ticks) = ticks {
                let seconds = ticks as f64 / ACTION_TICKS_PER_SECOND;
                let args = Args::new()
                    .text("line", line)
                    .text("seconds", number(seconds));
                line = written(lang.render("item-lasts", &args));
            }
            let args = Args::new().text("line", line).text("ends", ends);
            written(lang.render("item-ends", &args))
        })
        .collect()
}

/// "+40 Toughness", "−8 Agility", "+10% Strength", "Restores 1 Health a second".
fn bonus_line(bonus: &Bonus, lang: &Lang) -> String {
    let stat = stat_words(bonus.stat, lang);
    let (id, milli) = match (bonus.stat, bonus.amount) {
        (Stat::Restore(_), Amount::Flat(milli)) => {
            let amount = number(f64::from(milli) / 1000.0);
            let args = Args::new().text("amount", amount).text("stat", stat);
            return written(lang.render("item-restores", &args));
        }
        (_, Amount::Flat(milli)) => ("item-bonus", milli),
        (_, Amount::Percent(milli)) => ("item-bonus-percent", milli),
    };
    let args = Args::new().text("amount", signed(milli)).text("stat", stat);
    written(lang.render(id, &args))
}

/// What a stat is called: "Toughness", "most Mana".
fn stat_words(stat: Stat, lang: &Lang) -> String {
    let vital = |vital: Vital| match vital {
        Vital::Health => "health",
        Vital::Mana => "mana",
    };
    let id = match stat {
        Stat::Attribute(attribute) => format!("stat-{}", attribute.name().to_lowercase()),
        Stat::Most(of) => format!("stat-most-{}", vital(of)),
        Stat::Restore(of) => format!("stat-{}", vital(of)),
        Stat::Skill(skill) => {
            let args = Args::new().text("skill", skill.name());
            return written(lang.render("stat-skill", &args));
        }
    };
    written(lang.render(&id, &Args::new()))
}

/// Thousandths as a signed number: "+40", "−0.6" (a true minus sign), to one
/// place at most.
pub(super) fn signed(milli: i32) -> String {
    let sign = if milli < 0 { "\u{2212}" } else { "+" };
    format!("{sign}{}", number(f64::from(milli.unsigned_abs()) / 1000.0))
}

/// A number to one place at most, without a needless ".0": "40", "12.3".
pub(super) fn number(value: f64) -> String {
    let tenths = (value * 10.0).round() as i64;
    match tenths % 10 {
        0 => (tenths / 10).to_string(),
        _ => format!("{:.1}", tenths as f64 / 10.0),
    }
}

/// What could be written, any part missing left in braces (and logged).
fn written(words: Result<String, RenderError>) -> String {
    words.unwrap_or_else(|error| {
        bevy::log::warn!("Words missing in {}: {:?}", error.id, error.problems);
        error.text
    })
}

#[cfg(test)]
mod tests;
