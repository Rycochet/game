//! The login screen: how to log in to this server (a button for each
//! provider, Play as Guest if it takes guests, Cancel), then while the player
//! logs in on the provider's page in their browser (Open Again, Back).

use bevy::prelude::*;

use crate::common::{LoggingIn, LoginChoice, open_in_browser};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, INK, Pressable, REFUSED, TextSize, UiId, spawn_button,
    spawn_dialog,
};

/// Its widgets' form, for their ids.
const FORM: &str = "login";

/// Over the loading screen, as making a character is.
const LAYER: i32 = 0;

const WIDTH: f32 = 420.0;
const BUTTON_HEIGHT: f32 = 44.0;

#[derive(Component)]
pub(super) struct LoginLayer;

/// What the screen says.
#[derive(Component)]
pub(super) struct Words;

/// Where the buttons go, one under another.
#[derive(Component)]
pub(super) struct Buttons;

/// What a button does.
#[derive(Component, Clone, Debug)]
pub(super) enum Press {
    Choose(LoginChoice),
    OpenAgain(String),
}

pub(super) fn spawn_screen(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, LAYER, "Log in");
    commands.entity(dialog.layer).insert(LoginLayer);
    commands.entity(dialog.content).with_children(|content| {
        content.spawn((
            Words,
            Text::new(""),
            TextSize::Medium.font(),
            TextColor(INK),
            Node {
                width: px(WIDTH),
                ..default()
            },
        ));
        content.spawn((
            Buttons,
            Node {
                flex_direction: FlexDirection::Column,
                width: px(WIDTH),
                row_gap: px(8),
                // Clear of the words above.
                margin: UiRect::top(px(16)),
                ..default()
            },
        ));
    });
}

/// Shows the screen while there is something to ask, built again for it.
pub(super) fn show_screen(
    mut commands: Commands,
    logging_in: Res<LoggingIn>,
    mut layer: Single<&mut Visibility, With<LoginLayer>>,
    mut words: Single<(&mut Text, &mut TextColor), With<Words>>,
    buttons: Single<Entity, With<Buttons>>,
) {
    if !logging_in.is_changed() {
        return;
    }
    layer.set_if_neq(if logging_in.shows() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let (text, colour, presses) = match &*logging_in {
        LoggingIn::Not => return,
        LoggingIn::Choosing {
            providers,
            guests,
            failed,
        } => {
            let mut presses: Vec<(String, Press, &str)> = providers
                .iter()
                .map(|provider| {
                    let choice = LoginChoice::Provider(provider.name.clone());
                    (
                        format!("Log in with {}", provider.label),
                        Press::Choose(choice),
                        "provider",
                    )
                })
                .collect();
            if *guests {
                presses.push((
                    "Play as Guest".into(),
                    Press::Choose(LoginChoice::Guest),
                    "guest",
                ));
            }
            presses.push((
                "Cancel".into(),
                Press::Choose(LoginChoice::Cancel),
                "cancel",
            ));
            let (text, colour) = match failed {
                Some(reason) => (format!("Logging in didn't work: {reason}."), REFUSED),
                None => (choosing_words(*guests), INK),
            };
            (text, colour, presses)
        }
        LoggingIn::InBrowser { provider, url } => {
            let mut presses = Vec::new();
            if let Some(url) = url {
                presses.push((
                    "Open Again".into(),
                    Press::OpenAgain(url.clone()),
                    "open_again",
                ));
            }
            presses.push(("Back".into(), Press::Choose(LoginChoice::Back), "back"));
            let text = match url {
                Some(_) => format!(
                    "Log in with {} in your browser, then come back here. \
                     If no page opened, press Open Again.",
                    provider.label
                ),
                None => format!("Opening {}'s page in your browser...", provider.label),
            };
            (text, INK, presses)
        }
    };
    words.0.0 = text;
    words.1.0 = colour;
    commands.entity(*buttons).despawn_children();
    commands.entity(*buttons).with_children(|column| {
        for (index, (label, press, name)) in presses.into_iter().enumerate() {
            let node = Node {
                height: px(BUTTON_HEIGHT),
                ..default()
            };
            let style = if index == 0 {
                ButtonStyle::Main
            } else {
                ButtonStyle::Plain
            };
            let id = match name {
                "provider" => UiId::new(FORM, name).at(index),
                _ => UiId::new(FORM, name),
            };
            spawn_button(column, ButtonState::Normal, label, TextSize::Large, node)
                .insert((press, id, style, Pressable));
        }
    });
}

fn choosing_words(guests: bool) -> String {
    if guests {
        "Log in to keep your characters with your account, or play as a guest on this \
         computer."
            .into()
    } else {
        "This server needs you to log in.".into()
    }
}

/// Does what the button pressed says.
pub(super) fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&Press>,
    mut choices: MessageWriter<LoginChoice>,
) {
    for press in pressed.read() {
        match buttons.get(press.0) {
            Ok(Press::Choose(choice)) => {
                choices.write(choice.clone());
            }
            Ok(Press::OpenAgain(url)) => {
                if let Err(reason) = open_in_browser(url) {
                    warn!("Can't open the login page: {reason}");
                }
            }
            Err(_) => {}
        }
    }
}
