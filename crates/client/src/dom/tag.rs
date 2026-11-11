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
    Button,
    /// By its `type`: `text`, `checkbox` (a switch with `switch`) or `range`.
    Input,
    /// A dropdown of its `option`s.
    Select,
    /// A choice in a `select`, `toggle-row`, `tab-bar` or `colour-picker`.
    Option,
    /// Joined buttons, one chosen, all on show.
    ToggleRow,
    /// Folder tabs, to switch between views of one thing.
    TabBar,
    /// Waits for a key, and shows the one chosen.
    KeyBox,
    /// Swatches, one chosen.
    ColourPicker,
    /// A parchment panel; `moveable` by its title, or by a `grip`.
    Panel,
    /// A panel in front of everything, darkening the rest; `modal` for the
    /// full-screen kind.
    Dialog,
    /// A dialog's row of buttons, at the right; a container's last row.
    Footer,
    /// The frame round a grid of things: its `title`, a `header` of
    /// controls, what it holds, and a `footer`.
    Container,
    /// A container's controls, at its top right.
    Header,
    /// What is in it scrolls when it is too tall.
    Scroll,
    /// One thing shown, by its `type` and `for`: an item (`objects`).
    Object,
    /// A grid of things, and a place holding one. Their owner draws what is
    /// in them and says what picking up and letting go there do, for now.
    ObjectGrid,
    ObjectSlot,
}

/// What may be inside an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Holds {
    Nothing,
    /// Only words.
    Words,
    /// Only words, or only one picture: what a button shows.
    WordsOrPicture,
    /// Only these elements, and no words.
    Only(&'static [Tag]),
    Anything,
}

const ALL: [(&str, Tag); 35] = [
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
    ("button", Tag::Button),
    ("input", Tag::Input),
    ("select", Tag::Select),
    ("option", Tag::Option),
    ("toggle-row", Tag::ToggleRow),
    ("tab-bar", Tag::TabBar),
    ("key-box", Tag::KeyBox),
    ("colour-picker", Tag::ColourPicker),
    ("panel", Tag::Panel),
    ("dialog", Tag::Dialog),
    ("footer", Tag::Footer),
    ("container", Tag::Container),
    ("header", Tag::Header),
    ("scroll", Tag::Scroll),
    ("object", Tag::Object),
    ("object-grid", Tag::ObjectGrid),
    ("object-slot", Tag::ObjectSlot),
];

/// The elements that hold `option`s.
const CHOOSERS: [Tag; 4] = [Tag::Select, Tag::ToggleRow, Tag::TabBar, Tag::ColourPicker];

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
            Self::Br
            | Self::Hr
            | Self::Img
            | Self::Canvas
            | Self::Input
            | Self::KeyBox
            | Self::Object
            | Self::ObjectGrid
            | Self::ObjectSlot => Holds::Nothing,
            Self::Progress | Self::Option => Holds::Words,
            Self::Button => Holds::WordsOrPicture,
            Self::Table => Holds::Only(&[Self::Tr]),
            Self::Tr => Holds::Only(&[Self::Td, Self::Th]),
            _ if CHOOSERS.contains(&self) => Holds::Only(&[Self::Option]),
            _ => Holds::Anything,
        }
    }

    /// The only elements this one may be straight inside, if it is so bound.
    pub(super) fn belongs_in(self) -> &'static [Self] {
        match self {
            // In a template, rows are copied into a table.
            Self::Tr => &[Self::Table, Self::Template],
            Self::Td | Self::Th => &[Self::Tr],
            Self::Option => &CHOOSERS,
            Self::Footer => &[Self::Dialog, Self::Container],
            Self::Header => &[Self::Container],
            _ => &[],
        }
    }

    /// Something the player uses, built from one of the widgets: it has no
    /// elements of its own inside, and what it does comes back by its `id`.
    pub fn is_control(self) -> bool {
        matches!(self, Self::Button | Self::Input | Self::KeyBox) || CHOOSERS.contains(&self)
    }

    /// Whether it is an error to leave its `id` out: code has to find it.
    pub(super) fn needs_id(self) -> bool {
        matches!(
            self,
            Self::Form
                | Self::Template
                | Self::Canvas
                | Self::Dialog
                | Self::ObjectGrid
                | Self::ObjectSlot
        ) || self.is_control()
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
            Self::Button => &["selected", "hold"],
            Self::Input => &[
                "type",
                "value",
                "checked",
                "readonly",
                "label",
                "switch",
                "min",
                "max",
                "step",
                "inputmode",
                "prefix",
                "suffix",
                "accepts-items",
                "src",
            ],
            Self::Select => &["value", "searchable"],
            Self::Option => &["value", "selected", "label", "src"],
            Self::ToggleRow | Self::TabBar | Self::ColourPicker | Self::KeyBox => &["value"],
            Self::Panel => &["moveable", "grip"],
            Self::Dialog => &["modal", "depth", "closable"],
            Self::Scroll => &["max-height", "auto", "scrollbars"],
            Self::Object => &["type", "for"],
            Self::ObjectGrid => &[
                "type", "for", "columns", "rows", "grows", "places", "stacks", "compact",
            ],
            Self::ObjectSlot => &["accepts", "for", "key"],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests;
