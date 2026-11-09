//! What bags are kept and drawn as: the list, their meshes, parts and sparkles.

use bevy::prelude::*;
use shared::ground::Load;
use std::collections::BTreeMap;

use crate::view3d::XrayMaterial;

/// The bags known of on this level, by cell, as last seen.
#[derive(Resource, Default)]
pub(super) struct Bags(pub(super) BTreeMap<(i64, i64), Bag>);

pub(super) struct Bag {
    pub(super) load: Load,
    pub(super) root: Entity,
}

/// The bag the player is walking to, to open on arrival.
#[derive(Resource, Default)]
pub(super) struct Heading(pub(super) Option<(i64, i64)>);

/// What a bag is drawn with.
#[derive(Resource)]
pub(super) struct BagAssets {
    /// By look and variety: the bag in colour, in grey and its outlines.
    pub(super) bags: BTreeMap<(Load, u8), BagMeshes>,
    pub(super) star: Handle<Mesh>,
    pub(super) lit: Handle<StandardMaterial>,
    pub(super) remembered: Handle<StandardMaterial>,
    pub(super) sparkle: Handle<StandardMaterial>,
    pub(super) outline: Handle<StandardMaterial>,
    pub(super) star_edge: Handle<StandardMaterial>,
    /// The bag drawn through whatever hides it (`view3d::XrayMaterial`), white:
    /// its body's faint fill and its outline.
    pub(super) xray_fill: Handle<XrayMaterial>,
    pub(super) xray_outline: Handle<XrayMaterial>,
}

pub(super) struct BagMeshes {
    pub(super) lit: Handle<Mesh>,
    pub(super) remembered: Handle<Mesh>,
    pub(super) outline: Handle<Mesh>,
    pub(super) hover_outline: Handle<Mesh>,
    pub(super) star_outline: Handle<Mesh>,
    pub(super) star_hover_outline: Handle<Mesh>,
}

/// What depends on the state of the bag at this cell.
#[derive(Component, Clone, Copy)]
pub(super) enum BagPart {
    /// Shows while it is in sight.
    Lit,
    /// Shows while it is out of sight.
    Remembered,
    /// Shows while it is hovered: wider than the outline it always has.
    HoverOutline,
    /// Shows while it is hovered and in sight: the same for its sparkle.
    SparkleHoverOutline,
}

/// On each of a bag's parts: whose it is.
#[derive(Component, Clone, Copy)]
pub(super) struct OfBag(pub(super) (i64, i64));

/// A sparkle, standing behind its bag and facing the camera.
#[derive(Component, Clone, Copy)]
pub(super) struct Sparkle {
    pub(super) cell: (i64, i64),
    /// How far up it stands, how far behind the middle, and how big it is.
    pub(super) up: f32,
    pub(super) behind: f32,
    pub(super) scale: f32,
    /// So that bags don't twinkle in step.
    pub(super) phase: f32,
}

/// One star of a sparkle: how big it is, and how far out of step with the others.
#[derive(Component, Clone, Copy)]
pub(super) struct Star {
    pub(super) scale: f32,
    pub(super) phase: f32,
}
