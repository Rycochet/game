//! Attributes written back as markup, for showing what a document holds now.

use super::values::{Align, Length, VAlign};
use super::{Attrs, SIDES};

impl Attrs {
    /// These as markup would have them, each with a space before it:
    /// ` class="warning" hidden`.
    pub fn written(&self) -> String {
        let mut out = String::new();
        let mut write = |name: &str, value: Option<String>| match value.as_deref() {
            Some("") => out.push_str(&format!(" {name}")),
            Some(value) => {
                let value = value.replace('&', "&amp;").replace('"', "&quot;");
                out.push_str(&format!(" {name}=\"{value}\""));
            }
            None => {}
        };
        write("class", self.class.as_ref().map(|class| class.to_string()));
        write("title", self.title.as_ref().map(|title| title.to_string()));
        write("align", self.align.map(|align| align.name().to_owned()));
        write("valign", self.valign.map(|valign| valign.name().to_owned()));
        let lengths = [
            ("top", self.top),
            ("left", self.left),
            ("right", self.right),
            ("bottom", self.bottom),
            ("width", self.width),
            ("height", self.height),
        ];
        for (name, length) in lengths {
            write(name, length.map(Length::written));
        }
        for (side, margin) in SIDES.iter().zip(self.margin) {
            let margin = margin.map(|margin| margin.to_string());
            write(&format!("margin-{side}"), margin);
        }
        for (name, value) in &self.own {
            write(name, Some(value.to_string()));
        }
        let flags = [
            ("hidden", self.hidden),
            ("disabled", self.disabled),
            ("inert", self.inert),
            ("nowrap", self.nowrap),
        ];
        for (name, _) in flags.iter().filter(|(_, on)| *on) {
            write(name, Some(String::new()));
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
