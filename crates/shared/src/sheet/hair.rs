//! Each kind's choices of hair style and beard, from the creature research
//! (`docs/context/research-creatures.md`): dwarves' long beards, halflings' curls,
//! elves' fine long hair. Saved by place in the list, so new ones go at the end; the
//! first is what a creature has if nothing was chosen.

use crate::body::{Beard, Hair, HairStyle};
use crate::creature::CreatureKind;

use super::looks::word;
use super::{Appearance, Choice, Trait};

const PEOPLE_STYLES: [HairStyle; 6] = {
    use HairStyle::*;
    [Short, Long, TiedBack, Braids, Topknot, Bald]
};
const DWARF_STYLES: [HairStyle; 5] = {
    use HairStyle::*;
    [Long, Braids, TiedBack, Short, Bald]
};
const HALFLING_STYLES: [HairStyle; 4] = {
    use HairStyle::*;
    [Curly, Short, Long, TiedBack]
};
const ELF_STYLES: [HairStyle; 4] = {
    use HairStyle::*;
    [Long, Braids, TiedBack, Short]
};
const ORC_STYLES: [HairStyle; 5] = {
    use HairStyle::*;
    [Short, Crest, Topknot, Long, Bald]
};

const PEOPLE_BEARDS: [Beard; 4] = [Beard::None, Beard::Short, Beard::Full, Beard::Long];
const DWARF_BEARDS: [Beard; 5] = {
    use Beard::*;
    [Long, Braided, Forked, Full, Short]
};
const ORC_BEARDS: [Beard; 3] = [Beard::None, Beard::Short, Beard::Full];

/// Each style as a choice in words.
const fn style_words<const N: usize>(styles: [HairStyle; N]) -> [Choice; N] {
    let mut choices = [word(""); N];
    let mut at = 0;
    while at < N {
        choices[at] = word(styles[at].name());
        at += 1;
    }
    choices
}

const fn beard_words<const N: usize>(beards: [Beard; N]) -> [Choice; N] {
    let mut choices = [word(""); N];
    let mut at = 0;
    while at < N {
        choices[at] = word(beards[at].name());
        at += 1;
    }
    choices
}

const PEOPLE_STYLE_CHOICES: [Choice; 6] = style_words(PEOPLE_STYLES);
const DWARF_STYLE_CHOICES: [Choice; 5] = style_words(DWARF_STYLES);
const HALFLING_STYLE_CHOICES: [Choice; 4] = style_words(HALFLING_STYLES);
const ELF_STYLE_CHOICES: [Choice; 4] = style_words(ELF_STYLES);
const ORC_STYLE_CHOICES: [Choice; 5] = style_words(ORC_STYLES);
const PEOPLE_BEARD_CHOICES: [Choice; 4] = beard_words(PEOPLE_BEARDS);
const DWARF_BEARD_CHOICES: [Choice; 5] = beard_words(DWARF_BEARDS);
const ORC_BEARD_CHOICES: [Choice; 3] = beard_words(ORC_BEARDS);

/// The hair styles a creature of this kind may have, in order.
fn styles_for(kind: CreatureKind) -> &'static [HairStyle] {
    use CreatureKind::{Drider, Dwarf, Elf, Halfling, Orc};
    match kind {
        Dwarf => &DWARF_STYLES,
        Halfling => &HALFLING_STYLES,
        Elf | Drider => &ELF_STYLES,
        Orc => &ORC_STYLES,
        _ => &PEOPLE_STYLES,
    }
}

/// The beards a creature of this kind may have, in order.
fn beards_for(kind: CreatureKind) -> &'static [Beard] {
    match kind {
        CreatureKind::Dwarf => &DWARF_BEARDS,
        CreatureKind::Orc => &ORC_BEARDS,
        _ => &PEOPLE_BEARDS,
    }
}

pub(super) fn style_choices(kind: CreatureKind) -> &'static [Choice] {
    use CreatureKind::{Drider, Dwarf, Elf, Halfling, Orc};
    match kind {
        Dwarf => &DWARF_STYLE_CHOICES,
        Halfling => &HALFLING_STYLE_CHOICES,
        Elf | Drider => &ELF_STYLE_CHOICES,
        Orc => &ORC_STYLE_CHOICES,
        _ => &PEOPLE_STYLE_CHOICES,
    }
}

pub(super) fn beard_choices(kind: CreatureKind) -> &'static [Choice] {
    match kind {
        CreatureKind::Dwarf => &DWARF_BEARD_CHOICES,
        CreatureKind::Orc => &ORC_BEARD_CHOICES,
        _ => &PEOPLE_BEARD_CHOICES,
    }
}

impl Appearance {
    /// The hair it has: bald and beardless where its kind has no such choice.
    pub fn hair(&self, kind: CreatureKind) -> Hair {
        let chosen = |which: Trait| {
            let has = kind.def().looks.contains(&which);
            has.then(|| self.0.get(&which).copied().unwrap_or(0) as usize)
        };
        let style = chosen(Trait::HairStyle).and_then(|at| styles_for(kind).get(at).copied());
        let beard = chosen(Trait::Beard).and_then(|at| beards_for(kind).get(at).copied());
        Hair {
            style: style.unwrap_or(HairStyle::Bald),
            beard: beard.unwrap_or(Beard::None),
        }
    }

    /// The colour of its hair, if it has a choice of one.
    pub fn hair_colour(&self, kind: CreatureKind) -> Option<[u8; 3]> {
        self.choice(kind, Trait::HairColour)?.colour
    }
}

#[cfg(test)]
mod tests;
