//! The moveable panels the menu opens: Previous and Next for what a place draws,
//! and one showing every widget.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::rng::Seed;
use shared::site::{Generator, MAX_RUIN, StairsStyle};

use shared::protocol::ClientMessage;
use shared::world::MINUTES_PER_HOUR;

use super::actions::{DebugAction, kind_name};
use super::bodies::BodiesToShow;
use crate::common::{CurrentDungeon, GameView, HerbGrowthShift, ServerClock};
use crate::connection::ToServer;
use crate::dungeon_map::{CROP_STAGES, HERB_STAGES};
use crate::widgets::{
    BUTTON_PADDING, ButtonPressed, ButtonState, Dropdown, INK, Pressable, RaisePanel, TextBox,
    raised_box, spawn_button, spawn_check_button, spawn_close_button, spawn_dropdown,
    spawn_icon_button, spawn_moveable_panel, spawn_slider, spawn_tabs, spawn_text_box,
    spawn_tooltip,
};

const FONT: f32 = 16.0;
const CHECK_ROW: f32 = 34.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(60.0, 110.0);
const NEXT_AT: Vec2 = Vec2::new(30.0, 30.0);

/// The panels open, by what opened them.
#[derive(Resource, Default)]
pub(super) struct OpenPanels {
    panels: HashMap<DebugAction, Entity>,
    opened: u32,
}

/// A button that moves what `action`'s panel is for on or back a step.
#[derive(Component, Clone, Copy)]
pub(super) struct StepButton {
    action: DebugAction,
    forward: bool,
}

impl OpenPanels {
    pub(super) fn is_open(&self, action: DebugAction) -> bool {
        self.panels.contains_key(&action)
    }

    /// Closes this action's panel if it is open, otherwise opens it.
    pub(super) fn toggle(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        match self.panels.remove(&action) {
            Some(panel) if nodes.contains(panel) => commands.entity(panel).despawn(),
            _ => self.open(action, commands, raise, nodes),
        }
    }

    /// Opens this action's panel, or if it is open already, brings it to the
    /// front. `nodes` says which panels are still there: closing despawns them.
    pub(super) fn open(
        &mut self,
        action: DebugAction,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        if let Some(&panel) = self.panels.get(&action)
            && nodes.contains(panel)
        {
            raise.write(RaisePanel(panel));
            return;
        }
        let at = FIRST_AT + NEXT_AT * (self.opened % 8) as f32;
        self.opened += 1;
        let (panel, content) = spawn_moveable_panel(commands, action.name(), at);
        commands
            .entity(content)
            .with_children(|content| match action {
                DebugAction::ShowWidgets => spawn_every_widget(content),
                DebugAction::SunsAndMoons => {}
                _ => spawn_step_buttons(content, action),
            });
        if action == DebugAction::SunsAndMoons {
            commands.entity(content).insert(BodiesToShow);
        }
        self.panels.insert(action, panel);
    }
}

fn spawn_step_buttons(content: &mut ChildSpawnerCommands, action: DebugAction) {
    content
        .spawn(Node {
            column_gap: px(4),
            ..default()
        })
        .with_children(|row| {
            let [back, on] = if action == DebugAction::WorldTime {
                ["-1 hour", "+1 hour"]
            } else {
                ["Previous", "Next"]
            };
            for (label, forward) in [(back, false), (on, true)] {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, FONT, node)
                    .insert((StepButton { action, forward }, Pressable));
            }
        });
}

/// One of each thing `widgets` makes, to see how they look together.
fn spawn_every_widget(content: &mut ChildSpawnerCommands) {
    for (state, label) in [
        (ButtonState::Normal, "Normal"),
        (ButtonState::Selected, "Selected"),
        (ButtonState::Active, "Active"),
        (ButtonState::Disabled, "Disabled"),
        (ButtonState::DisabledSelected, "Disabled, selected"),
    ] {
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, state, label, FONT, node).insert(Pressable);
    }
    for (on, label) in [
        (Some(true), "Check on"),
        (Some(false), "Check off"),
        (None, "No check"),
    ] {
        let node = Node {
            height: px(CHECK_ROW),
            ..default()
        };
        spawn_check_button(content, ButtonState::Normal, on, label, FONT, node).insert(Pressable);
    }
    spawn_tabs(content, &["Tab", "Other tab", "Third tab"], 0, FONT);
    spawn_close_button(content, default());
    spawn_slider(content, 0.5, None, 160.0);
    let kinds = || PoiKind::ALL.map(kind_name).to_vec();
    for searchable in [false, true] {
        let dropdown = Dropdown {
            options: kinds(),
            selected: 0,
            searchable,
        };
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_dropdown(content, dropdown, FONT, node);
    }
    let text_box = TextBox {
        text: "Text box".into(),
        takes_keys: true,
        ..default()
    };
    let node = Node {
        height: px(CHECK_ROW),
        ..default()
    };
    spawn_text_box(content, text_box, FONT, node);
    content.spawn(raised_box()).with_child((
        Text::new("Raised box"),
        TextFont::from_font_size(FONT),
        TextColor(crate::widgets::INK),
    ));
    content
        .spawn(Node::default())
        .with_children(|holder| spawn_tooltip(holder, SampleTooltip, "Tooltip"));
}

/// The sample tooltip's text.
#[derive(Component)]
pub(super) struct SampleTooltip;

/// Open the Generate POI panel for this kind (the submenu chose it).
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct GenerateKind(pub PoiKind);

/// What the Generate POI panel is set to, besides its seed box.
#[derive(Resource, Default)]
pub(super) struct GenerateForm {
    kind: Option<PoiKind>,
    ruin: u8,
    seeds_made: i64,
}

#[derive(Component, Clone, Copy)]
pub(super) struct RuinButton(u8);

#[derive(Component)]
pub(super) struct SeedBox;

#[derive(Component)]
pub(super) struct NewSeedButton;

#[derive(Component)]
pub(super) struct GenerateButton;

/// The most digits of a seed the new seed button makes: enough for many, and
/// short enough to write down and type in again.
const NEW_SEEDS: u64 = 1_000_000_000;
const SEED_WIDTH: f32 = 240.0;

/// A seed to try. `now` is any number that keeps changing, `made` how many so far.
fn new_seed(now: u64, made: i64) -> u64 {
    Seed::new(now).at(&[made]) % NEW_SEEDS
}

/// A round arrow, drawn here as there is no such letter in the font.
fn new_seed_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [64, 43, 26, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (dx, dy) = (x as f32 - middle, y as f32 - middle);
            let reach = dx.hypot(dy);
            // Clockwise from the top, leaving a gap for the arrow's head.
            let round = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU);
            let ring = (8.0..12.0).contains(&reach) && round > 0.9;
            // The head, pointing round the ring at the top right.
            let head_along = dx + 2.0;
            let head =
                (0.0..8.0).contains(&head_along) && (dy + 10.0).abs() < (8.0 - head_along) * 0.9;
            if ring || head {
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

fn row() -> Node {
    Node {
        column_gap: px(4),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn words(text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(FONT),
        TextColor(INK),
    )
}

/// Opens the Generate POI panel for the kind chosen, in place of any open.
#[allow(clippy::too_many_arguments)]
pub(super) fn open_generate_panel(
    mut kinds: MessageReader<GenerateKind>,
    mut commands: Commands,
    mut open: ResMut<OpenPanels>,
    mut form: ResMut<GenerateForm>,
    mut images: ResMut<Assets<Image>>,
    nodes: Query<(), With<Node>>,
    time: Res<Time<Real>>,
) {
    let Some(&GenerateKind(kind)) = kinds.read().last() else {
        return;
    };
    let action = DebugAction::GeneratePoi;
    if let Some(panel) = open.panels.remove(&action)
        && nodes.contains(panel)
    {
        commands.entity(panel).despawn();
    }
    let at = FIRST_AT + NEXT_AT * (open.opened % 8) as f32;
    open.opened += 1;
    let (panel, content) = spawn_moveable_panel(&mut commands, action.name(), at);
    open.panels.insert(action, panel);
    form.kind = Some(kind);
    form.ruin = 0;
    form.seeds_made += 1;
    let seed = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
    let icon = images.add(new_seed_icon());
    commands.entity(content).with_children(|content| {
        content.spawn(words(kind_name(kind)));
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Ruin"));
            for ruin in 0..=MAX_RUIN {
                let node = Node {
                    padding: BUTTON_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, ruin.to_string(), FONT, node)
                    .insert((RuinButton(ruin), Pressable));
            }
        });
        content.spawn(row()).with_children(|row| {
            row.spawn(words("Seed"));
            let text_box = TextBox {
                text: seed.to_string(),
                digits_only: true,
                takes_keys: true,
                ..default()
            };
            let node = Node {
                width: px(SEED_WIDTH),
                height: px(CHECK_ROW),
                ..default()
            };
            spawn_text_box(row, text_box, FONT, node).insert(SeedBox);
            let node = Node {
                width: px(CHECK_ROW),
                height: px(CHECK_ROW),
                ..default()
            };
            spawn_icon_button(row, ButtonState::Normal, icon, node)
                .insert((NewSeedButton, Pressable));
        });
        let node = Node {
            padding: BUTTON_PADDING,
            ..default()
        };
        spawn_button(content, ButtonState::Normal, "Generate", FONT, node)
            .insert((GenerateButton, Pressable));
    });
}

/// What the Generate POI panel's buttons do.
#[allow(clippy::too_many_arguments)]
pub(super) fn press_generate_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    ruins: Query<&RuinButton>,
    new_seed_button: Query<(), With<NewSeedButton>>,
    generate: Query<(), With<GenerateButton>>,
    mut seed_box: Query<&mut TextBox, With<SeedBox>>,
    mut form: ResMut<GenerateForm>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(ruin) = ruins.get(press.0) {
            form.ruin = ruin.0;
        }
        let Ok(mut seed) = seed_box.single_mut() else {
            continue;
        };
        if new_seed_button.contains(press.0) {
            form.seeds_made += 1;
            let new = new_seed(time.elapsed().as_nanos() as u64, form.seeds_made);
            seed.text = new.to_string();
        }
        if generate.contains(press.0)
            && let (Some(kind), Ok(seed)) = (form.kind, seed.text.parse())
        {
            to_server.write(ToServer(ClientMessage::DevGenerateSite {
                kind,
                seed,
                ruin: form.ruin,
            }));
        }
    }
}

/// Shows the ruin chosen, and greys what can't be used: ruin for kinds that have
/// none, and Generate without a seed.
pub(super) fn update_generate_buttons(
    form: Res<GenerateForm>,
    mut ruins: Query<(&RuinButton, &mut ButtonState), Without<GenerateButton>>,
    mut generate: Query<&mut ButtonState, With<GenerateButton>>,
    seed_box: Query<&TextBox, With<SeedBox>>,
) {
    let can_ruin = form.kind.is_some_and(Generator::can_be_ruined);
    for (button, mut state) in &mut ruins {
        let chosen = button.0 == form.ruin;
        state.set_if_neq(match (can_ruin, chosen) {
            (true, true) => ButtonState::Selected,
            (true, false) => ButtonState::Normal,
            (false, true) => ButtonState::DisabledSelected,
            (false, false) => ButtonState::Disabled,
        });
    }
    let has_seed = seed_box
        .single()
        .is_ok_and(|seed| seed.text.parse::<u64>().is_ok());
    for mut state in &mut generate {
        state.set_if_neq(if has_seed {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}

/// Whether a panel's buttons can do anything here.
fn can_step(action: DebugAction, view: GameView, dungeon: &CurrentDungeon) -> bool {
    action == DebugAction::WorldTime
        || view == GameView::Dungeon
            && match action {
                DebugAction::StairsStyle => !dungeon.stairs.is_empty(),
                DebugAction::FieldsState => !dungeon.fields.is_empty(),
                DebugAction::HerbsState => dungeon.level_has(Cell::HerbBed),
                _ => true,
            }
}

/// Forgets panels closed by their own close buttons.
pub(super) fn forget_closed(mut open: ResMut<OpenPanels>, nodes: Query<(), With<Node>>) {
    let closed = open.panels.values().any(|&panel| !nodes.contains(panel));
    if closed {
        open.panels.retain(|_, panel| nodes.contains(*panel));
    }
}

/// The style before this one, going round.
fn previous_style(style: StairsStyle) -> StairsStyle {
    let all = StairsStyle::ALL;
    let at = all.iter().position(|&other| other == style).unwrap_or(0);
    all[(at + all.len() - 1) % all.len()]
}

/// Moves what the pressed button's panel is for on or back a step: only what is
/// drawn, and the next level goes back to its own.
pub(super) fn press_panel_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&StepButton>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut shift: ResMut<HerbGrowthShift>,
    clock: Res<ServerClock>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(&StepButton { action, forward }) = buttons.get(press.0) else {
            continue;
        };
        match action {
            DebugAction::StairsStyle => {
                for stairs in &mut dungeon.stairs {
                    stairs.style = if forward {
                        stairs.style.next()
                    } else {
                        previous_style(stairs.style)
                    };
                }
            }
            DebugAction::FieldsState => {
                for field in &mut dungeon.fields {
                    // As crops, which alone go through every stage.
                    field.livestock = false;
                    let by = if forward { 1 } else { CROP_STAGES as u32 - 1 };
                    field.variety = field.variety.wrapping_add(by);
                }
            }
            DebugAction::HerbsState => {
                // Back is nearly all the way round, as the shift can't go below 0.
                let by = if forward { 1 } else { HERB_STAGES - 1 };
                shift.0 = (shift.0 + by) % HERB_STAGES;
            }
            DebugAction::WorldTime => {
                let by = if forward {
                    MINUTES_PER_HOUR
                } else {
                    -MINUTES_PER_HOUR
                };
                to_server.write(ToServer(ClientMessage::DevSetTime {
                    minutes: clock.minutes + by,
                }));
            }
            _ => {}
        }
    }
}

/// Greys the buttons where they can do nothing; the panel stays open.
pub(super) fn update_panel_buttons(
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut buttons: Query<(&StepButton, &mut ButtonState)>,
) {
    for (button, mut state) in &mut buttons {
        state.set_if_neq(if can_step(button.action, *view.get(), &dungeon) {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_seeds_are_short_and_differ() {
        let seeds: Vec<u64> = (0..50).map(|made| new_seed(12345, made)).collect();
        assert!(seeds.iter().all(|&seed| seed < NEW_SEEDS));
        assert!(seeds.iter().any(|&seed| seed != seeds[0]));
    }

    #[test]
    fn previous_undoes_next_for_every_style() {
        for style in StairsStyle::ALL {
            assert_eq!(previous_style(style.next()), style);
        }
    }

    #[test]
    fn panels_need_a_site_with_what_they_change() {
        let dungeon = CurrentDungeon::default();
        let in_site = |action| can_step(action, GameView::Dungeon, &dungeon);
        assert!(!in_site(DebugAction::StairsStyle));
        assert!(!in_site(DebugAction::FieldsState));
        assert!(!in_site(DebugAction::HerbsState));
        assert!(!can_step(
            DebugAction::StairsStyle,
            GameView::WorldMap,
            &dungeon
        ));
    }
}
