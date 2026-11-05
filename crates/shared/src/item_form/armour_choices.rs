//! What can be asked of a kind of armour or footwear: each value of each option that
//! a roll asked for it, with the rest of the request, makes from a fixed seed or two,
//! so the same everywhere and never one that can't be made.

use super::armour::Piece;
use super::armour_ask::armour_form_to;
use super::choices::Choices;
use super::request::{Field, RollRequest, fields};
use crate::armour::{Construction, Dye, Finish, Style};
use crate::item::{ItemKind, Quality};
use crate::material::Material;
use crate::rng::Seed;

/// How many seeds a value is tried with. Each roll already tries many times over.
const SEEDS: u64 = 2;

/// Each option's values for this piece.
pub(super) fn armour_choices(
    kind: ItemKind,
    piece: Piece,
    quality: Quality,
    request: &RollRequest,
) -> Choices {
    let options = Options {
        kind,
        piece,
        quality,
        request,
    };
    Choices {
        styles: options.values(Field::Style, Style::ALL, |request, style| RollRequest {
            style: Some(style),
            ..request
        }),
        constructions: options.values(Field::Construction, Construction::ALL, |request, made| {
            RollRequest {
                construction: Some(made),
                ..request
            }
        }),
        materials: options.values(Field::Material, Material::ALL, |request, material| {
            RollRequest {
                material: Some(material),
                ..request
            }
        }),
        finishes: options.values(Field::Finish, Finish::all(), |request, finish| {
            RollRequest {
                finish: Some(finish),
                ..request
            }
        }),
        dyes: options.values(Field::Dye, Dye::ALL, |request, dye| RollRequest {
            dye: Some(dye),
            ..request
        }),
        ..Choices::default()
    }
}

/// What is being asked about.
struct Options<'a> {
    kind: ItemKind,
    piece: Piece,
    quality: Quality,
    request: &'a RollRequest,
}

impl Options<'_> {
    /// The values of one option that can be made with the rest of the request, in
    /// order: none if the piece doesn't take it.
    fn values<T: Copy>(
        &self,
        field: Field,
        all: impl IntoIterator<Item = T>,
        ask: impl Fn(RollRequest, T) -> RollRequest,
    ) -> Vec<T> {
        if !fields(self.kind).contains(&field) {
            return Vec::new();
        }
        let rest = self.request.without(field);
        let made = |request: &RollRequest| {
            (0..SEEDS).any(|n| {
                let mut rng = Seed::new(n).derive("roll choices").rng();
                armour_form_to(self.piece, &mut rng, self.quality, request).is_ok()
            })
        };
        all.into_iter()
            .filter(|&value| made(&ask(rest, value)))
            .collect()
    }
}
