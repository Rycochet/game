//! Connect's popups: the box to paste a server's certificate into, and the
//! requester asking whether to forget a server.

use bevy::prelude::*;

use super::super::{Action, BUTTON_ROW, FONT, Layer, SystemMenu};
use crate::widgets::{
    ButtonState, ButtonStyle, INK, Pressable, TextBox, TextSize, spawn_button, spawn_dialog,
    spawn_text_box,
};

/// Above Connect.
const POPUP_LAYER: i32 = super::super::ABOVE_MENU_LAYER + 1;
const POPUP_WIDTH: f32 = 480.0;
const BUTTON_WIDTH: f32 = 120.0;
const EXPLAINED: &str = "Paste in the server's certificate, from its Server Options or its \
     /certificate page, or its key's fingerprint from its log. Only a server with this \
     certificate will be accepted. Without one, any certificate is.";

/// The box the certificate is pasted into.
#[derive(Component)]
pub(in crate::system_menu) struct CertificateBox;

/// Says whether what is in the box is a certificate.
#[derive(Component)]
pub(in crate::system_menu) struct CertificateStatus;

/// The certificate popup's Keep, only enabled for a certificate.
#[derive(Component)]
pub(in crate::system_menu) struct KeepButton;

/// What the forget requester asks.
#[derive(Component)]
pub(in crate::system_menu) struct ForgetWords;

pub(in crate::system_menu) fn spawn_popups(commands: &mut Commands) {
    spawn_certificate(commands);
    spawn_forget(commands);
}

fn button_node() -> Node {
    Node {
        width: px(BUTTON_WIDTH),
        height: px(BUTTON_ROW),
        ..default()
    }
}

fn spawn_certificate(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, POPUP_LAYER, "Certificate");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::Certificate]), Action::Back));
    commands
        .entity(dialog.content)
        .insert(Node {
            width: px(POPUP_WIDTH),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                Text::new(EXPLAINED),
                TextSize::Medium.font(),
                TextColor(INK),
            ));
            let node = Node {
                width: percent(100),
                height: px(BUTTON_ROW),
                overflow: Overflow::clip(),
                ..default()
            };
            let text_box = TextBox {
                takes_keys: true,
                ..default()
            };
            spawn_text_box(content, text_box, FONT, node).insert(CertificateBox);
            content.spawn((
                CertificateStatus,
                Text::new(""),
                TextSize::Small.font(),
                TextColor(INK),
            ));
        });
    commands.entity(dialog.buttons).with_children(|row| {
        for (label, action, style) in [
            ("Keep", Action::KeepCertificate, ButtonStyle::Main),
            ("Clear", Action::ClearCertificate, ButtonStyle::Plain),
            ("Cancel", Action::Back, ButtonStyle::Plain),
        ] {
            let mut button = spawn_button(row, ButtonState::Normal, label, FONT, button_node());
            button.insert((action, style, Pressable));
            if action == Action::KeepCertificate {
                button.insert(KeepButton);
            }
        }
    });
}

fn spawn_forget(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, POPUP_LAYER, "Forget server");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::Forgetting]), Action::Back));
    commands.entity(dialog.content).with_child((
        ForgetWords,
        Text::new(""),
        FONT.font(),
        TextColor(INK),
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        for (label, action, style) in [
            ("Yes, forget it", Action::ReallyForget, ButtonStyle::Main),
            ("No", Action::Back, ButtonStyle::Plain),
        ] {
            let node = Node {
                width: px(BUTTON_WIDTH + 30.0),
                ..button_node()
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node)
                .insert((action, style, Pressable));
        }
    });
}
