//! Opening and closing the Quest Log: its key (L) opens it or closes it, and
//! so does its close button; leaving the game closes it. Opening asks the
//! server which planets the character has quest history on.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, QuestAsk};

use super::markup::QUEST_LOG;
use super::state::LogPanel;
use crate::common::{Action, GameView, Preferences, Session};
use crate::connection::ToServer;
use crate::dom::{UiEvent, UiValue, document, top_strip};
use crate::widgets::UiId;

pub(super) fn toggle_log(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    mut panel: ResMut<LogPanel>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    if *view.get() == GameView::Connecting
        || !preferences.controls.just_pressed(&keys, Action::QuestLog)
    {
        return;
    }
    if let Some(document) = panel.document.take() {
        commands.entity(document).despawn();
        return;
    }
    let document = commands
        .spawn(document(&QUEST_LOG))
        .insert(top_strip())
        .id();
    to_server.write(ToServer(ClientMessage::Quest(QuestAsk::Planets)));
    *panel = LogPanel {
        document: Some(document),
        // What other planets had may have changed since.
        runes: std::mem::take(&mut panel.runes),
        planets_dirty: true,
        dirty: true,
        ..default()
    };
}

/// Forgets the log once its close button has closed it, and closes it on
/// leaving the game.
pub(super) fn notice_closed(
    mut events: MessageReader<UiEvent>,
    mut commands: Commands,
    mut panel: ResMut<LogPanel>,
    session: Res<Session>,
) {
    let closed = |event: &UiEvent| {
        event.id == UiId::new("quest_log", "panel") && event.value == UiValue::Closed
    };
    let closed = events.read().filter(|event| closed(event)).count() > 0;
    let Some(document) = panel.document else {
        return;
    };
    if closed || !session.is_playing() {
        // The rest of the document goes with its panel.
        commands.entity(document).despawn();
        panel.document = None;
    }
}
