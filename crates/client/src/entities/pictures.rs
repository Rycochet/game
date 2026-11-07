//! Which picture stands for each kind of creature on its token, and how big.

use shared::creature::CreatureKind;

use crate::common::TokenSize;

/// The picture for a kind of creature.
pub(super) fn picture(kind: CreatureKind) -> &'static str {
    match kind {
        // (No centaur, dwarf, light elf, goblin, minotaur or birdfolk among the pictures
        // yet.)
        CreatureKind::Centaur
        | CreatureKind::Human
        | CreatureKind::Dwarf
        | CreatureKind::Elf
        | CreatureKind::Goblin
        | CreatureKind::Minotaur
        | CreatureKind::Birdfolk => "monsters/humanfighter.png",
        CreatureKind::Beggar => "monsters/humandruid01.png",
        CreatureKind::Cattle => "monsters/cowright.png",
        CreatureKind::Chicken => "monsters/chicken1.png",
        CreatureKind::Drider => "monsters/spiderblack.png",
        CreatureKind::Giant => "monsters/gianthill.png",
        CreatureKind::Horse => "monsters/horse01.png",
        CreatureKind::Kobold => "monsters/koboldfighter.png",
        CreatureKind::Halfling => "monsters/halflingfighter.png",
        CreatureKind::Lizardfolk => "monsters/lizardgreen.png",
        CreatureKind::Orc => "monsters/orcfighter.png",
        CreatureKind::ScorpionMan => "monsters/scorpionbrown.png",
        CreatureKind::Wolf => "monsters/doggrey.png",
    }
}

/// How big a kind of creature's picture is drawn.
pub(super) fn picture_size(kind: CreatureKind) -> TokenSize {
    match kind {
        CreatureKind::Chicken => TokenSize::Small,
        CreatureKind::Giant | CreatureKind::Minotaur => TokenSize::Large,
        _ => TokenSize::Default,
    }
}
