//! Attributes written back as markup, for showing what a document holds now.

use super::values::{Align, Length, VAlign};
use super::{Attrs, SIDES};

impl Attrs {
    /// These as markup would have them, each with a space before it:
    /// ` class="warning" hidden`.
    pub fn written(&self) -> String {
        let mut out = String::new();
        let mut value = |name: &str, value: Option<String>| {
            if let Some(value) = value {
                let value = value.replace('&', "&amp;").replace('"', "&quot;");
                out.push_str(&format!(" {name}=\"{value}\""));
            }
        };
        let text = |words: &Option<std::borrow::Cow<'static, str>>| {
            words.as_ref().map(|words| words.to_string())
        };
        let number = |number: Option<f32>| number.map(|number| number.to_string());
        value("class", text(&self.class));
        value("title", text(&self.title));
        value("for", text(&self.for_id));
        value("align", self.align.map(|align| align.name().to_owned()));
        value("valign", self.valign.map(|valign| valign.name().to_owned()));
        let lengths = [
            ("top", self.top),
            ("left", self.left),
            ("right", self.right),
            ("bottom", self.bottom),
            ("width", self.width),
            ("height", self.height),
        ];
        for (name, length) in lengths {
            value(name, length.map(Length::written));
        }
        for (side, margin) in SIDES.iter().zip(self.margin) {
            value(&format!("margin-{side}"), number(margin));
        }
        value("src", text(&self.src));
        value("alt", text(&self.alt));
        value("value", number(self.value));
        value("max", number(self.max));
        value("colspan", self.colspan.map(|span| span.to_string()));
        value("cellspacing", number(self.cellspacing));
        value("cellpadding", number(self.cellpadding));
        let flags = [
            ("hidden", self.hidden),
            ("disabled", self.disabled),
            ("inert", self.inert),
        ];
        for (name, _) in flags.iter().filter(|(_, on)| *on) {
            out.push_str(&format!(" {name}"));
        }
        out
    }
}

impl Length {
    fn written(self) -> String {
        match self {
            Self::Pixels(pixels) => pixels.to_string(),
            Self::Percent(share) => format!("{share}%"),
        }
    }
}

impl Align {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

impl VAlign {
    fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Middle => "middle",
            Self::Bottom => "bottom",
        }
    }
}
