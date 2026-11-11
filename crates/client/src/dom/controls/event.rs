//! What a player did to a control, told by the control's id.

use bevy::prelude::*;

use crate::common::Binding;
use crate::widgets::UiId;

/// A control was used. Read these in `Update` and match on the id:
/// `event.id == UiId::new("connect", "go")`.
#[derive(Message, Clone, Debug, PartialEq)]
pub struct UiEvent {
    pub id: UiId,
    pub value: UiValue,
}

/// What a control now holds, by its kind.
#[derive(Clone, Debug, PartialEq)]
pub enum UiValue {
    /// A button was pressed.
    Pressed,
    /// A checkbox or switch: whether it is on.
    Flag(bool),
    /// A text box: what it says.
    Text(String),
    /// A slider: where it is, from its `min` to its `max`.
    Number(f32),
    /// A dropdown, toggle row, tab bar or colour picker: which option, and
    /// its `value` (its words, if it has none).
    Choice { index: usize, value: String },
    /// A key box: the key chosen, or none.
    Key(Option<Binding>),
}
