//! The kinds of value an attribute can have, and reading them.

/// A size or a distance: interface pixels, or a share of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Pixels(f32),
    Percent(f32),
}

/// Where what is in an element goes, from side to side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// The same, from top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
    /// In a row: everything as tall as the row (columns side by side).
    Stretch,
}

/// A number that isn't negative.
pub(super) fn number(name: &str, value: &str) -> Result<f32, String> {
    match value.parse::<f32>() {
        Ok(number) if number >= 0.0 && number.is_finite() => Ok(number),
        _ => Err(format!("`{name}` must be a number, not `{value}`")),
    }
}

/// `12` (pixels) or `50%`.
pub(super) fn length(name: &str, value: &str) -> Result<Length, String> {
    match value.strip_suffix('%') {
        Some(share) => number(name, share).map(Length::Percent),
        None => number(name, value).map(Length::Pixels),
    }
}

/// Spacing is in steps of 4 pixels, so things line up; 0 is fine.
pub(super) fn spacing(name: &str, value: &str) -> Result<f32, String> {
    let pixels = number(name, value)?;
    if pixels % 4.0 == 0.0 {
        Ok(pixels)
    } else {
        Err(format!("`{name}` must be a multiple of 4, not `{value}`"))
    }
}

pub(super) fn align(value: &str) -> Result<Align, String> {
    match value {
        "left" => Ok(Align::Left),
        "center" => Ok(Align::Center),
        "right" => Ok(Align::Right),
        _ => Err(format!(
            "`align` is `left`, `center` or `right`, not `{value}`"
        )),
    }
}

pub(super) fn valign(value: &str) -> Result<VAlign, String> {
    match value {
        "top" => Ok(VAlign::Top),
        "middle" => Ok(VAlign::Middle),
        "bottom" => Ok(VAlign::Bottom),
        "stretch" => Ok(VAlign::Stretch),
        _ => Err(format!(
            "`valign` is `top`, `middle`, `bottom` or `stretch`, not `{value}`"
        )),
    }
}

/// A whole number of at least 1.
pub(super) fn count(name: &str, value: &str) -> Result<u16, String> {
    match value.parse::<u16>() {
        Ok(count) if count >= 1 => Ok(count),
        _ => Err(format!(
            "`{name}` must be a whole number from 1, not `{value}`"
        )),
    }
}
