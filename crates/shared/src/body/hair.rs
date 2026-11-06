//! Hair on the head and beards: the shapes a creature's hair can take, for drawing.
//! Which a creature has is a look (`sheet::Trait::HairStyle`, `Trait::Beard`), chosen
//! from its kind's list; its colour is `Trait::HairColour`.

/// How the hair on the head is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HairStyle {
    Bald,
    /// Close to the head, above the ears and the nape.
    Short,
    /// Loose to the shoulders.
    Long,
    /// Pulled back into a tail behind the head.
    TiedBack,
    /// Two plaits hanging down the back.
    Braids,
    /// Gathered in a knot on top of the head.
    Topknot,
    /// A ridge from the forehead to the nape, the sides bare.
    Crest,
    /// A thick mop of curls.
    Curly,
}

impl HairStyle {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bald => "Bald",
            Self::Short => "Short",
            Self::Long => "Long",
            Self::TiedBack => "Tied back",
            Self::Braids => "Braids",
            Self::Topknot => "Topknot",
            Self::Crest => "Crest",
            Self::Curly => "Curly",
        }
    }
}

/// A beard, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Beard {
    None,
    /// Close along the jaw.
    Short,
    /// Thick round the jaw and chin, with a moustache.
    Full,
    /// Hanging down the chest.
    Long,
    /// Long, in one plait.
    Braided,
    /// Long, parted into two points.
    Forked,
}

impl Beard {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Short => "Short",
            Self::Full => "Full",
            Self::Long => "Long",
            Self::Braided => "Braided",
            Self::Forked => "Forked",
        }
    }
}

/// The hair a creature has: on its head and its chin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hair {
    pub style: HairStyle,
    pub beard: Beard,
}

impl Hair {
    /// No hair at all, as anything without a choice of hair.
    pub const NONE: Self = Self {
        style: HairStyle::Bald,
        beard: Beard::None,
    };
}
