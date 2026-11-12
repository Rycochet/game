//! The loading screen: the game's name and a bar saying what is happening,
//! covering everything from the first frame until the game can take input, and
//! again while the connection is lost. See `docs/context/client/loading_screen.md`.

use bevy::diagnostic::FrameCount;
use bevy::input::InputSystems;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{
    CharacterSelect, GameView, LoadingStep, LoggingIn, MakingCharacter, PointerSystems,
    ServerStatus, Session, TouchGesture, interface_area,
};

mod frames;
mod screen;
mod takes_input;
use crate::dom::DomInput;
use crate::dom::{Dom, DrawObjects};
use frames::keep_frame_rate;
use screen::{bind_screen, spawn_screen, title_node};

/// Built into the program, rather than loaded, to be there on the first frame.
/// Cinzel Decorative Bold cut down to the letters of [`TITLE`] (see `ATTRIBUTION.md`).
const TITLE_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/HallOfWorldsTitle.ttf");

const TITLE: &str = "Hall of\nWorlds";
/// The share of the window the title may fill.
const TITLE_HEIGHT_SHARE: f32 = 0.5;
/// At the startup menu, the title keeps to the top of the window, above the menu.
const START_TITLE_HEIGHT_SHARE: f32 = 0.28;
const START_TITLE_TOP: f32 = 6.0;
const TITLE_WIDTH_SHARE: f32 = 0.8;
/// How wide the title's longest line is ("Worlds" in this font), and how tall
/// its two lines are, in font sizes.
const TITLE_EMS_WIDE: f32 = 4.61;
const TITLE_EMS_HIGH: f32 = 2.4;

const BACKGROUND: Color = Color::srgb(0.04, 0.03, 0.07);
const GOLD: Color = Color::srgb(0.93, 0.75, 0.38);
const TITLE_SHADOW: Color = Color::srgb(0.35, 0.12, 0.05);
const WRITING: Color = Color::srgb(0.95, 0.92, 0.85);
const REFUSED: Color = Color::srgb(1.0, 0.35, 0.3);

/// Frames before the screen has certainly been drawn once, so "Starting..." is
/// seen before the work starts.
const FRAMES_TO_DRAW: u32 = 3;
/// Frames after being welcomed, for the view to be built and drawn and the
/// messages following the welcome (such as being in a site) to arrive.
const FRAMES_TO_SETTLE: u32 = 3;

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoldLoadingScreen>()
            .add_systems(Startup, spawn_screen)
            // Straight after the screen is built, so the title is there on
            // the first frame.
            .add_systems(
                PostUpdate,
                bind_screen
                    .after(DrawObjects)
                    .before(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                PreUpdate,
                // At the startup menu, the menu over the screen takes what it
                // wants, as making a character does.
                take_all_input
                    .run_if(takes_input::takes_input)
                    .after(InputSystems)
                    .after(PointerSystems)
                    .before(DomInput::Presses),
            )
            .add_systems(
                Update,
                (update_step, show_screen, fit_title, keep_frame_rate).chain(),
            );

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, remove_page_screen.after(update_step));
    }
}

/// Keeps the screen up whatever happens (dev tools), to look at it.
#[derive(Resource, Default)]
pub struct HoldLoadingScreen(pub bool);

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct Title;

#[derive(Component)]
struct BarText;

/// The next step: `frames` is how many frames have been run, `settled` how many
/// of them in a row the client has spent welcomed.
fn next_step(step: LoadingStep, frames: u32, settled: u32) -> LoadingStep {
    if step == LoadingStep::Starting && frames < FRAMES_TO_DRAW {
        LoadingStep::Starting
    } else if settled > FRAMES_TO_SETTLE {
        LoadingStep::Ready
    } else {
        LoadingStep::Connecting
    }
}

/// The biggest font size that keeps the title in its share of the window, which
/// is less at the startup menu.
fn title_font_size(window: Vec2, at_start: bool) -> f32 {
    let height_share = if at_start {
        START_TITLE_HEIGHT_SHARE
    } else {
        TITLE_HEIGHT_SHARE
    };
    (window.x * TITLE_WIDTH_SHARE / TITLE_EMS_WIDE).min(window.y * height_share / TITLE_EMS_HIGH)
}

/// What the bar says, and how full it is.
fn bar_shows(step: LoadingStep, status: &ServerStatus) -> (String, f32) {
    if let ServerStatus::Refused(reason) = status {
        return (reason.clone(), 0.0);
    }
    match step {
        LoadingStep::Starting => ("Starting...".into(), 0.0),
        LoadingStep::Connecting | LoadingStep::Ready => ("Connecting...".into(), 0.5),
    }
}

/// Nothing behind the screen gets any input: no keys, clicks, wheel or fingers.
fn take_all_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
) {
    keys.reset_all();
    mouse.reset_all();
    *scroll = default();
    *gesture = default();
}

fn update_step(
    frames: Res<FrameCount>,
    status: Res<ServerStatus>,
    view: Res<State<GameView>>,
    hold: Res<HoldLoadingScreen>,
    mut step: ResMut<LoadingStep>,
    mut settled: Local<u32>,
) {
    let welcomed = status.is_connected() && *view.get() != GameView::Connecting && !hold.0;
    *settled = if welcomed { *settled + 1 } else { 0 };
    step.set_if_neq(next_step(*step, frames.0, *settled));
}

#[allow(clippy::too_many_arguments)]
fn show_screen(
    step: Res<LoadingStep>,
    status: Res<ServerStatus>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    select: Res<CharacterSelect>,
    logging_in: Res<LoggingIn>,
    mut screen: Single<&mut Visibility, With<LoadingScreen>>,
    mut text: Query<&mut TextColor, With<BarText>>,
    new_text: Query<(), Added<BarText>>,
    mut dom: Dom,
) {
    let changed = step.is_changed() || status.is_changed() || session.is_changed();
    let choosing = making.is_changed() || select.is_changed();
    let refused = matches!(*status, ServerStatus::Refused(_));
    if !new_text.is_empty() || changed {
        for mut colour in &mut text {
            colour.0 = if refused { REFUSED } else { WRITING };
        }
    }
    if !changed && !choosing && !logging_in.is_changed() {
        return;
    }
    let shown = match *step != LoadingStep::Ready {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    };
    screen.set_if_neq(shown);
    // Only the title behind the startup menu, or the screens choosing and making
    // a character and logging in.
    let bar = session.is_playing() && !making.0 && !select.open && !logging_in.shows();
    dom.set_flag("loading/bar", "hidden", !bar);
    let (words, full) = bar_shows(*step, &status);
    dom.set("loading/bar", "value", &full.to_string());
    dom.set_text("loading/bar", words);
}

fn fit_title(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    session: Res<Session>,
    mut title: Query<(&mut TextFont, &mut Node), With<Title>>,
    mut dom: Dom,
    mut last: Local<Option<(Vec2, bool)>>,
) {
    // The title keeps its share of the window, whatever size the interface is.
    let size = interface_area(&window, &ui_scale);
    let at_start = !session.is_playing();
    // Minimised: nothing to fit, and no size of writing would.
    if *last == Some((size, at_start)) || size.min_element() < 1.0 {
        return;
    }
    // (Not before it is built: it is made the right size then.)
    let Ok((mut font, mut node)) = title.single_mut() else {
        return;
    };
    *last = Some((size, at_start));
    font.font_size = title_font_size(size, at_start).into();
    *node = title_node(at_start);
    let valign = if at_start { "top" } else { "middle" };
    dom.set("loading/screen", "valign", valign);
}

/// Removes the screen in `index.html`, shown during the download, once this one
/// has certainly been drawn to take its place.
#[cfg(target_arch = "wasm32")]
fn remove_page_screen(step: Res<LoadingStep>, mut removed: Local<bool>) {
    if *removed || *step == LoadingStep::Starting {
        return;
    }
    *removed = true;
    let screen = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading"));
    if let Some(screen) = screen {
        screen.remove();
    }
}

#[cfg(test)]
mod tests;
