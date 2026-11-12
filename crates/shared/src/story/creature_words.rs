//! Creature kinds as values for language files: `creature-<kind>`, rendered
//! with `$count` so each language picks its plural form.

use crate::creature::CreatureKind;
use crate::lang::Selector;

impl Selector for CreatureKind {
    fn fact(&self) -> &'static str {
        "creature"
    }

    fn key(&self) -> &'static str {
        use CreatureKind::*;
        match self {
            Beggar => "beggar",
            Birdfolk => "birdfolk",
            Cattle => "cattle",
            Centaur => "centaur",
            Chicken => "chicken",
            Drider => "drider",
            Dwarf => "dwarf",
            Elf => "elf",
            Giant => "giant",
            Goblin => "goblin",
            Halfling => "halfling",
            Horse => "horse",
            Human => "human",
            Kobold => "kobold",
            Lizardfolk => "lizardfolk",
            Minotaur => "minotaur",
            Orc => "orc",
            ScorpionMan => "scorpion-man",
            Wolf => "wolf",
        }
    }
}
