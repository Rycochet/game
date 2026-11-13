//! The panel's words from the language files (`shared::lang`): who someone
//! is, what a giver says when a quest can't be taken yet (never numbers), and
//! when something asked can't be done.

use shared::lang::{Args, Lang};
use shared::protocol::{Ahead, Hint, QuestRefusal};
use shared::quest::Standing;
use shared::site::BuildingKind;
use shared::story::Person;

pub(super) use crate::quests::{or_partly, plain};

/// The message naming what the giver of a building of `kind` does, if it has
/// one ("occupation-potion-shop"): a house's has none.
pub(super) fn occupation_id(kind: BuildingKind) -> String {
    let words = kind.name().to_lowercase().replace('\'', "");
    format!("occupation-{}", words.replace(' ', "-"))
}

/// "Mara Tull, baker", or just the name for a house.
pub(super) fn giver_title(lang: &Lang, giver: &Person, kind: BuildingKind) -> String {
    let name = or_partly(giver.full_name(lang));
    let id = occupation_id(kind);
    if !lang.has(&id) {
        return name;
    }
    let occupation = or_partly(lang.render(&id, &Args::new()));
    let args = Args::new()
        .name("name", &name)
        .text("occupation", occupation);
    or_partly(lang.render("giver-title", &args))
}

/// What a giver says of why a quest can't be taken yet.
pub(super) fn hint_text(lang: &Lang, hint: &Hint) -> String {
    let (id, args) = match hint {
        Hint::Standing { least } => {
            let id = match least {
                Standing::Stranger | Standing::Known => "hint-standing-known",
                Standing::Trusted => "hint-standing-trusted",
                Standing::Honoured => "hint-standing-honoured",
                Standing::Champion => "hint-standing-champion",
            };
            (id, Args::new())
        }
        Hint::Earlier => ("hint-earlier", Args::new()),
        Hint::Elsewhere {
            name: Some(name), ..
        } => ("hint-elsewhere-named", Args::new().name("town", name)),
        Hint::Elsewhere { name: None, .. } => ("hint-elsewhere", Args::new()),
        Hint::Skill(skill) => (
            "hint-skill",
            Args::new().text("skill", skill.name().to_lowercase()),
        ),
        Hint::Ahead(Ahead::Slain) => ("hint-dealt-with", Args::new()),
        Hint::Ahead(Ahead::Found) => ("hint-found", Args::new()),
    };
    or_partly(lang.render(id, &args))
}

/// What a giver says to something asked that couldn't be done.
pub(super) fn refusal(lang: &Lang, why: QuestRefusal) -> String {
    let id = match why {
        QuestRefusal::TooMany => "refused-too-many",
        QuestRefusal::NoRoom => "refused-no-room",
        QuestRefusal::NotOffered => "refused-not-offered",
        QuestRefusal::NotReady => "refused-not-ready",
    };
    or_partly(lang.render(id, &Args::new()))
}
