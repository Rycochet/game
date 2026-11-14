//! A creature look's pieces in millimetres, at its real size, standing on the ground.

use bevy::prelude::*;
use shared::body::{Hair, Layout};

use super::super::body::{Triangle, bare_pieces};
use super::super::colours::{Colours, material};
use super::super::morph::blend_colours;
use super::super::{Look, character};

/// What a model is made of.
pub(super) struct Pieces {
    /// Triangles in millimetres with what they are made of.
    pub(super) pieces: Vec<(Vec<Triangle>, StandardMaterial)>,
    /// The skin and any coat alone: the body's closed surface, without what is worn
    /// or sheets of hair (whose inner faces would look hidden by the body).
    pub(super) skin: Vec<Triangle>,
    /// The body as built, at its real size, for telling which part each point is on.
    pub(super) layout: Layout,
}

/// The pieces of a creature's look, `None` for a look that isn't a body.
pub(super) fn pieces(look: &Look) -> Option<Pieces> {
    match look {
        Look::Character {
            kind,
            worn,
            colours,
            hair,
            height,
        } => {
            let (pieces, layout) = character::dressed(*kind, worn, *colours, *hair)?;
            // Every frame is a person's height: shrunk or grown to the kind's own, as
            // `Layout::at_size` does, then to its own Height.
            let standing = Layout::at_size(*kind)?.scaled(*height);
            let share = standing.spine.base[1] as f32 / (layout.spine.base[1] as f32).max(1.0);
            let mut pieces: Vec<_> = pieces
                .into_iter()
                .map(|(triangles, material)| (scaled(triangles, share), material))
                .collect();
            let skin = pieces[..3].iter().flat_map(|(t, _)| t).copied().collect();
            pieces.retain(|(triangles, _)| !triangles.is_empty());
            Some(Pieces {
                pieces,
                skin,
                layout: standing,
            })
        }
        Look::Body {
            layout,
            height,
            colours,
            hair,
        } => {
            // A body standing on two legs is drawn as tall as `height`; an animal's
            // layout is its own size already.
            let share = match layout.frame() {
                Some(frame) => *height as f32 / frame.height as f32,
                None => 1.0,
            };
            let layout = layout.scaled((share * WHOLE as f32) as i32);
            Some(bare(&layout, *colours, *hair))
        }
        Look::Morph(morph) => {
            let t = morph.along();
            let layout = Layout::blend(&morph.from, &morph.to, t);
            let colours = blend_colours(&morph.colours[0], &morph.colours[1], t);
            let hair = morph.hair[usize::from(t >= shared::body::BLEND_WHOLE / 2)];
            Some(bare(&layout, colours, hair))
        }
        Look::Portrait(inner) => pieces(inner),
        _ => None,
    }
}

/// Thousandths, as `Layout::scaled` takes.
const WHOLE: i32 = 1000;

/// A bare body: skin, its markings, a taur's coat, feathers, hair and an animal's long hair.
fn bare(layout: &Layout, colours: Colours, hair: Hair) -> Pieces {
    let [skin, marked, coat, feathers, hair, mane] = bare_pieces(layout, hair, colours.marks);
    let surface = skin.iter().chain(&marked).chain(&coat).copied().collect();
    let pieces = [
        (skin, colours.skin),
        (
            marked,
            colours.marks.map_or(colours.skin, |marks| marks.colour),
        ),
        (coat, colours.coat),
        (feathers, colours.feathers),
        (hair, colours.hair),
        (mane, colours.mane),
    ]
    .into_iter()
    .filter(|(triangles, _)| !triangles.is_empty())
    .map(|(triangles, colour)| (triangles, material(colour)))
    .collect();
    Pieces {
        pieces,
        skin: surface,
        layout: *layout,
    }
}

/// Triangles made `share` times bigger about the ground under the middle.
fn scaled(triangles: Vec<Triangle>, share: f32) -> Vec<Triangle> {
    triangles
        .into_iter()
        .map(|triangle| triangle.map(|corner| corner * share))
        .collect()
}
