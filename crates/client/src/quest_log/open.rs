//! Opening and closing the Quest Log: its key (L) or its button opens it or
//! closes it, and so does its close button; leaving the game closes it. Opening asks the
//! server which planets the character has quest history on. Asked to open on
//! a quest (`quests::OpenQuestLog`), it opens there.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, QuestAsk};

use super::button::LogButton;
use super::markup::QUEST_LOG;
use super::state::LogPanel;
use crate::common::{Action, GameView, Preferences, Session};
use crate::connection::ToServer;
use crate::dom::{UiEvent, UiValue, document, top_strip};
use crate::quests::OpenQuestLog;
use crate::widgets::{ButtonPressed, UiId};

#[allow(clippy::too_many_arguments)]
pub(super) fn toggle_log(
    mut pressed: MessageReader<ButtonPressed>,
    button: Query<(), With<LogButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    mut panel: ResMut<LogPanel>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    let by_button = pressed.read().any(|press| button.contains(press.0));
    let by_key = preferences.controls.just_pressed(&keys, Action::QuestLog);
    if *view.get() == GameView::Connecting || !(by_button || by_key) {
        return;
    }
    if let Some(document) = panel.document.take() {
        commands.entity(document).despawn();
        return;
    }
    open_log(&mut panel, &mut to_server, &mut commands);
}

/// Opens the log on a quest of this planet when asked (the tracked quest
/// under the minimap, clicked), or shows it there if open.
pub(super) fn open_on_quest(
    mut asked: MessageReader<OpenQuestLog>,
    view: Res<State<GameView>>,
    mut panel: ResMut<LogPanel>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    let Some(&OpenQuestLog(id)) = asked.read().last() else {
        return;
    };
    if *view.get() == GameView::Connecting {
        return;
    }
    if !panel.is_open() {
        open_log(&mut panel, &mut to_server, &mut commands);
    }
    if panel.planet.take().is_some() {
        // The chooser back on "Here".
        panel.planets_dirty = true;
    }
    panel.chosen = Some(id);
    panel.abandoning = false;
    panel.dirty = true;
}

/// Opens the log, asking the server which planets it has quests on.
fn open_log(
    panel: &mut LogPanel,
    to_server: &mut MessageWriter<ToServer>,
    commands: &mut Commands,
) {
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
