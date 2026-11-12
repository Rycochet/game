//! A story's values for its language files: every name and fact it has, by
//! the names wordings use (`$giver-name`, `$place-direction`). Proper names
//! go as they are; facts as their words in the language (`Lang::phrase`);
//! genders as keys for select expressions.

use super::cast::{Member, Role};
use super::facts::{CountBand, RewardPart};
use super::told::Story;
use crate::gender::Gender;
use crate::item::coin_stacks;
use crate::lang::{Args, Lang, RenderError, Selector};

/// The kebab-case name of an item kind's coin message (`coin-gold`).
fn coin_message(kind: crate::item::ItemKind) -> &'static str {
    use crate::item::ItemKind::*;
    match kind {
        GoldCoin => "coin-gold",
        SilverCoin => "coin-silver",
        _ => "coin-copper",
    }
}

fn member(args: Args, lang: &Lang, member: &Member) -> Result<Args, RenderError> {
    let role = member.role.key();
    let mut args = args
        .name(format!("{role}-name"), member.person.full_name(lang)?)
        .name(format!("{role}-given"), &member.person.given)
        .choice(format!("{role}-gender"), &member.person.gender)
        .text(format!("{role}-nature"), lang.phrase(&member.nature)?)
        .text(format!("{role}-state"), lang.phrase(&member.state)?);
    if let Some(relation) = member.relation {
        args = args.text(format!("{role}-relation"), lang.phrase(&relation)?);
    }
    Ok(args)
}

/// The reward in words: "3 gold coins, a ring and training".
pub(super) fn reward(lang: &Lang, parts: &[RewardPart]) -> Result<String, RenderError> {
    let mut words = Vec::new();
    for part in parts {
        match part {
            RewardPart::Coins(copper) => {
                for (kind, count) in coin_stacks(*copper, u32::MAX) {
                    let args = Args::new().count("count", i64::from(count));
                    words.push(lang.render(coin_message(kind), &args)?);
                }
            }
            RewardPart::Item(item) => words.push(lang.rules().indefinite(&lang.phrase(item)?)),
            RewardPart::Training => words.push(lang.render("reward-training", &Args::new())?),
            RewardPart::Favour => words.push(lang.render("reward-favour", &Args::new())?),
        }
    }
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    Ok(lang.rules().list(&words))
}

/// The creatures in words, in the plural their count band calls for.
pub(super) fn creatures(
    lang: &Lang,
    kind: &impl Selector,
    count: CountBand,
) -> Result<String, RenderError> {
    let id = format!("{}-{}", kind.fact(), kind.key());
    lang.render(&id, &Args::new().count("count", count.plural_count()))
}

impl Story {
    /// Every value this story has, for any of its messages.
    pub fn args(&self, lang: &Lang) -> Result<Args, RenderError> {
        let mut args = Args::new().name("home", &self.facts.home);
        for one in &self.cast.members {
            args = member(args, lang, one)?;
        }
        if let Some(passerby) = self.cast.passerby {
            let words = lang.rules().indefinite(&lang.phrase(&passerby)?);
            args = (args.text("witness-name", words)).choice("witness-gender", &Gender::Neutral);
        }
        if let Some(pet) = &self.cast.pet {
            args = args
                .name("pet-name", &pet.pet.name)
                .text("pet-kind", lang.phrase(&pet.pet.kind)?)
                .choice("pet-gender", &pet.pet.gender)
                .text("pet-nature", lang.phrase(&pet.nature)?);
        }
        if let Some(boss) = &self.cast.boss {
            args =
                (args.name("boss-name", boss.full_name(lang)?)).choice("boss-gender", &boss.gender);
        }
        let facts = &self.facts;
        if let Some(place) = &facts.place {
            let words = match &place.name {
                Some(name) => name.clone(),
                None => place.description.describe(lang)?,
            };
            args = args
                .text("place", words)
                .text(
                    "place-named",
                    if place.name.is_some() { "yes" } else { "no" },
                )
                .text("place-direction", lang.phrase(&place.direction)?)
                .text("place-distance", lang.phrase(&place.distance)?);
        }
        if let Some(found) = &facts.creatures {
            args = args
                .text("creatures", creatures(lang, &found.kind, found.count)?)
                .count("creature-count", found.count.plural_count())
                .text("creature-number", lang.phrase(&found.count)?);
        }
        if let Some(to) = &facts.destination {
            args = args
                .name("destination", &to.name)
                .text("destination-direction", lang.phrase(&to.direction)?)
                .text("destination-distance", lang.phrase(&to.distance)?);
        }
        if !facts.reward.is_empty() {
            args = args.text("reward", reward(lang, &facts.reward)?);
        }
        let colour: [(&str, Option<&dyn Phrase>); 7] = [
            ("item", facts.item.as_ref().map(|v| v as &dyn Phrase)),
            ("errand", facts.errand.as_ref().map(|v| v as &dyn Phrase)),
            (
                "time-ago",
                facts.time_ago.as_ref().map(|v| v as &dyn Phrase),
            ),
            (
                "occasion",
                facts.occasion.as_ref().map(|v| v as &dyn Phrase),
            ),
            ("stake", facts.stake.as_ref().map(|v| v as &dyn Phrase)),
            ("season", facts.season.as_ref().map(|v| v as &dyn Phrase)),
            ("weather", facts.weather.as_ref().map(|v| v as &dyn Phrase)),
        ];
        for (key, value) in colour {
            if let Some(value) = value {
                args = args.text(key, value.words(lang)?);
            }
        }
        Ok(args)
    }

    /// The member in `role`'s values only, for the ending's liar.
    pub(super) fn role_name(&self, lang: &Lang, role: Role) -> Result<String, RenderError> {
        match (self.cast.member(role), self.cast.passerby) {
            (Some(member), _) => member.person.full_name(lang),
            (None, Some(passerby)) => Ok(lang.rules().indefinite(&lang.phrase(&passerby)?)),
            (None, None) => self.cast.giver().person.full_name(lang),
        }
    }
}

/// A fact that can be put in words, whatever its type (Rust can't keep
/// `impl Selector` values of different types in one list, but can keep
/// `&dyn Phrase`).
pub(super) trait Phrase {
    fn words(&self, lang: &Lang) -> Result<String, RenderError>;
}

impl<T: Selector> Phrase for T {
    fn words(&self, lang: &Lang) -> Result<String, RenderError> {
        lang.phrase(self)
    }
}
