//! The character panel: what the character is like, opened with `C` or the
//! button at the bottom left. See `docs/context/client/character.md`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::energy::NORMAL_SPEED;
use shared::item::Inventory;
use shared::protocol::ServerMessage;
use shared::sheet::Sheet;

use crate::common::{GameView, PlayerSpeed};
use crate::connection::FromServer;
use crate::inventory::Carried;
use crate::widgets::{
    ButtonPressed, ButtonState, INK, Pressable, spawn_icon_button, spawn_moveable_panel,
};

const BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 10.0;
/// Beside the pockets' button.
const BUTTON_LEFT: f32 = CORNER_GAP + BUTTON + 6.0;
const FONT: f32 = 16.0;
const PANEL_AT: Vec2 = Vec2::new(620.0, 140.0);

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSheet>()
            .add_message::<OpenCharacterPanel>()
            .init_resource::<Panel>()
            .add_systems(Startup, spawn_button)
            .add_systems(
                Update,
                (
                    read_server,
                    publish_speed,
                    toggle_panel,
                    show_words,
                    show_button,
                )
                    .chain(),
            );
    }
}

/// Opens the panel, for looking at it (`dev_tools`).
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct OpenCharacterPanel;

/// What the character is like, as the server says.
#[derive(Resource, Default)]
pub struct CharacterSheet(pub Sheet);

/// The panel, if open.
#[derive(Resource, Default)]
struct Panel(Option<Entity>);

#[derive(Component)]
struct PanelButton;

/// The panel's words.
#[derive(Component)]
struct SheetText;

fn read_server(mut from_server: MessageReader<FromServer>, mut sheet: ResMut<CharacterSheet>) {
    for message in from_server.read() {
        if let ServerMessage::Sheet(new) = &message.0 {
            sheet.0 = new.clone();
        }
    }
}

/// What the player's speed comes to, for moving them.
fn publish_speed(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut speed: ResMut<PlayerSpeed>,
) {
    speed.set_if_neq(PlayerSpeed(sheet.0.speed(&carried.0)));
}

/// Someone's head and shoulders, drawn here so there is no file to ship for it.
fn person_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 - middle, y as f32);
            let head = across.hypot(down - 10.0) < 6.5;
            let shoulders = across.hypot(down - 30.0) < 12.5 && down < 28.0;
            if head || shoulders {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn spawn_button(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let icon = images.add(person_icon());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(CORNER_GAP),
                left: px(BUTTON_LEFT),
                ..default()
            },
            // Above the maps, as the warning button is.
            GlobalZIndex(15),
        ))
        .with_children(|holder| {
            let node = Node {
                width: px(BUTTON),
                height: px(BUTTON),
                ..default()
            };
            spawn_icon_button(holder, ButtonState::Normal, icon, node)
                .insert((PanelButton, Pressable));
        });
}

#[allow(clippy::too_many_arguments)]
fn toggle_panel(
    mut pressed: MessageReader<ButtonPressed>,
    mut opened: MessageReader<OpenCharacterPanel>,
    button: Query<(), With<PanelButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<GameView>>,
    nodes: Query<(), With<Node>>,
    mut panel: ResMut<Panel>,
    mut commands: Commands,
) {
    let asked = pressed.read().any(|press| button.contains(press.0))
        || keys.just_pressed(KeyCode::KeyC)
        || opened.read().next().is_some();
    if !asked || *view.get() == GameView::Connecting {
        return;
    }
    // Its close button may have closed it already.
    match panel.0.take().filter(|&open| nodes.contains(open)) {
        Some(open) => commands.entity(open).despawn(),
        None => {
            let (root, content) = spawn_moveable_panel(&mut commands, "Character", PANEL_AT);
            commands.entity(content).with_child((
                SheetText,
                Text::default(),
                TextFont::from_font_size(FONT),
                TextColor(INK),
            ));
            panel.0 = Some(root);
        }
    }
}

/// What the panel says.
fn describe(sheet: &Sheet, carried: &Inventory) -> String {
    let def = carried.creature.def();
    let stats = sheet.stats;
    let mut lines = vec![
        format!("{}, {:?}", def.name, def.size),
        String::new(),
        format!("Strength {}", stats.strength),
        format!("Dexterity {}", stats.dexterity),
        format!("Intelligence {}", stats.intelligence),
        format!("Constitution {}", stats.constitution),
        String::new(),
        format!(
            "Carrying {:.2} of {:.2} kg: {:?}",
            carried.total_weight() as f32 / 1000.0,
            sheet.capacity(def.size) as f32 / 1000.0,
            sheet.load(carried),
        ),
        format!("Speed {}%", sheet.speed(carried) * 100 / NORMAL_SPEED),
    ];
    if !sheet.skills.is_empty() {
        lines.push(String::new());
        for (skill, level) in &sheet.skills {
            lines.push(format!("{skill:?} {level}"));
        }
    }
    if !def.abilities.is_empty() {
        lines.push(String::new());
        let abilities: Vec<_> = def.abilities.iter().map(|a| format!("{a:?}")).collect();
        lines.push(abilities.join(", "));
    }
    lines.push(String::new());
    for &which in def.looks {
        lines.push(format!(
            "{}: {}",
            which.name(),
            sheet.appearance.says(which)
        ));
    }
    lines.join("\n")
}

fn show_words(
    sheet: Res<CharacterSheet>,
    carried: Res<Carried>,
    mut texts: Query<&mut Text, With<SheetText>>,
) {
    for mut text in &mut texts {
        let words = describe(&sheet.0, &carried.0);
        if text.0 != words {
            text.0 = words;
        }
    }
}

/// The button is hidden while connecting, as the warning button is.
fn show_button(view: Res<State<GameView>>, mut button: Single<&mut Visibility, With<PanelButton>>) {
    button.set_if_neq(if *view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}

#[cfg(test)]
mod tests {
    use shared::creature::CreatureKind;

    use super::*;

    #[test]
    fn it_says_what_the_kind_is_and_how_loaded() {
        let carried = Inventory {
            creature: CreatureKind::Kobold,
            ..Inventory::default()
        };
        let words = describe(&Sheet::new(CreatureKind::Kobold), &carried);
        assert!(words.starts_with("Kobold, Small"), "{words}");
        assert!(words.contains("Strength 7"));
        assert!(words.contains("Stealth 3"));
        assert!(words.contains("NightVision"));
        assert!(words.contains("Light"));
        assert!(words.contains("Speed 100%"));
        assert!(words.contains("Height: Average"));
    }
}
