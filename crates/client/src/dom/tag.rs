//! The elements markup may use. Anything else is an error until it is needed:
//! see `docs/context/html-interface.md` for the whole planned set.

/// What an element is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tag {
    /// A block: what is in it stacks from the top.
    Div,
    /// Inline: what is in it flows from the left, wrapping.
    Span,
    Table,
    Tr,
    Td,
    Th,
    /// Draws nothing; names what is in it (`form/name`).
    Form,
    /// Never shown: a piece code copies, to fill lists.
    Template,
    Br,
    P,
    H1,
    H2,
    Small,
    Label,
    Img,
    /// A bar with a `value`, a spinner without.
    Progress,
    Hr,
    /// A box its owner's code draws in.
    Canvas,
}

/// What may be inside an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Holds {
    Nothing,
    /// Only words.
    Words,
    /// Only these elements, and no words.
    Only(&'static [Tag]),
    Anything,
}

const ALL: [(&str, Tag); 18] = [
    ("div", Tag::Div),
    ("span", Tag::Span),
    ("table", Tag::Table),
    ("tr", Tag::Tr),
    ("td", Tag::Td),
    ("th", Tag::Th),
    ("form", Tag::Form),
    ("template", Tag::Template),
    ("br", Tag::Br),
    ("p", Tag::P),
    ("h1", Tag::H1),
    ("h2", Tag::H2),
    ("small", Tag::Small),
    ("label", Tag::Label),
    ("img", Tag::Img),
    ("progress", Tag::Progress),
    ("hr", Tag::Hr),
    ("canvas", Tag::Canvas),
];

impl Tag {
    pub fn named(name: &str) -> Option<Self> {
        ALL.iter()
            .find(|(known, _)| *known == name)
            .map(|(_, tag)| *tag)
    }

    pub fn name(self) -> &'static str {
        let found = ALL.iter().find(|(_, tag)| *tag == self);
        found.map_or("", |(name, _)| name)
    }

    pub(super) fn holds(self) -> Holds {
        match self {
            Self::Br | Self::Hr | Self::Img | Self::Canvas => Holds::Nothing,
            Self::Progress => Holds::Words,
            Self::Table => Holds::Only(&[Self::Tr]),
            Self::Tr => Holds::Only(&[Self::Td, Self::Th]),
            _ => Holds::Anything,
        }
    }

    /// The only element this one may be straight inside, if there is one.
    pub(super) fn belongs_in(self) -> Option<Self> {
        match self {
            Self::Tr => Some(Self::Table),
            Self::Td | Self::Th => Some(Self::Tr),
            _ => None,
        }
    }

    /// Whether it is an error to leave its `id` out: code has to find it.
    pub(super) fn needs_id(self) -> bool {
        matches!(self, Self::Form | Self::Template | Self::Canvas)
    }

    /// Whether what is in it flows from the left, as words do.
    pub(super) fn is_inline(self) -> bool {
        matches!(
            self,
            Self::Span
                | Self::P
                | Self::H1
                | Self::H2
                | Self::Small
                | Self::Label
                | Self::Td
                | Self::Th
        )
    }

    /// The attributes only this element takes, beside those all do.
    pub(super) fn own_attrs(self) -> &'static [&'static str] {
        match self {
            Self::Img => &["src", "alt"],
            Self::Progress => &["value", "max"],
            Self::Td | Self::Th => &["colspan"],
            Self::Table => &["cellspacing", "cellpadding"],
            Self::Label => &["for"],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests;
