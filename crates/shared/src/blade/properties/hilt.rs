//! Rolling the hilt and what else is on the haft: hands, guard, pommel, ricasso, langets, handle and the parts at the side, top and back.

use super::Holding;
use crate::blade::table::Row;
use crate::blade::{
    AXE_LONG_HAFT, Back, Guard, HAFT, HandleShape, Hands, Pommel, Ricasso, SidePart, Tips, Top,
    pick,
};
use crate::rng::Rng;

/// What is rolled for the hilt and the rest of the haft.
pub(super) struct HiltRolls {
    pub(super) hands: Hands,
    pub(super) guard: Guard,
    pub(super) pommel: Pommel,
    pub(super) ricasso: Ricasso,
    pub(super) langets: bool,
    pub(super) handle_shape: HandleShape,
    pub(super) sides: [SidePart; 2],
    pub(super) top: Top,
    pub(super) back: Back,
    pub(super) tips: Tips,
}

pub(super) fn roll_hilt(rng: &mut Rng, row: &Row, holding: Holding, handle: u16) -> HiltRolls {
    let Holding {
        is_side,
        held,
        on_a_grip,
    } = holding;
    let hands = pick(
        rng,
        row.hands,
        &if on_a_grip || (is_side && handle < AXE_LONG_HAFT.0) {
            [(Hands::One, 100), (Hands::OneOrTwo, 0), (Hands::Two, 0)]
        } else if handle < HAFT.1 {
            [(Hands::One, 0), (Hands::OneOrTwo, 50), (Hands::Two, 50)]
        } else {
            [(Hands::One, 0), (Hands::OneOrTwo, 0), (Hands::Two, 100)]
        },
    );
    let guard = pick(
        rng,
        row.guard,
        &if on_a_grip {
            [
                (Guard::None, 15),
                (Guard::Cross, 50),
                (Guard::Disc, 15),
                (Guard::Cup, 8),
                (Guard::KnuckleBow, 8),
                (Guard::Basket, 4),
            ]
        } else if held {
            [
                (Guard::None, 0),
                (Guard::Cross, 80),
                (Guard::Disc, 20),
                (Guard::Cup, 0),
                (Guard::KnuckleBow, 0),
                (Guard::Basket, 0),
            ]
        } else {
            [
                (Guard::None, 100),
                (Guard::Cross, 0),
                (Guard::Disc, 0),
                (Guard::Cup, 0),
                (Guard::KnuckleBow, 0),
                (Guard::Basket, 0),
            ]
        },
    );
    let pommel = pick(
        rng,
        row.pommel,
        &if held {
            [
                (Pommel::None, 5),
                (Pommel::Plain, 55),
                (Pommel::Disc, 15),
                (Pommel::Wheel, 15),
                (Pommel::Ring, 5),
                (Pommel::Hooked, 5),
            ]
        } else {
            [
                (Pommel::None, 100),
                (Pommel::Plain, 0),
                (Pommel::Disc, 0),
                (Pommel::Wheel, 0),
                (Pommel::Ring, 0),
                (Pommel::Hooked, 0),
            ]
        },
    );
    let ricasso = pick(
        rng,
        row.ricasso,
        &[
            (Ricasso::None, 90),
            (Ricasso::Plain, u64::from(held) * 10),
            (Ricasso::WithHooks, 0),
        ],
    );
    let langets = pick(
        rng,
        row.langets,
        &[(true, u64::from(!held) * 25), (false, 75)],
    );
    let handle_shape = pick(
        rng,
        row.handle_shape,
        &if held {
            [
                (HandleShape::Straight, 80),
                (HandleShape::Curved, 15),
                (HandleShape::SCurved, 3),
                (HandleShape::Angled, 2),
            ]
        } else {
            [
                (HandleShape::Straight, 85),
                (HandleShape::Curved, 5),
                (HandleShape::SCurved, 10),
                (HandleShape::Angled, 0),
            ]
        },
    );
    let side_parts = [
        pick(
            rng,
            row.side_one,
            &[
                (SidePart::AxeBlade, u64::from(is_side)),
                (SidePart::None, u64::from(!is_side)),
            ],
        ),
        pick(rng, row.side_two, &[(SidePart::None, 1)]),
    ];
    let top = pick(
        rng,
        row.top,
        &[
            (Top::None, u64::from(is_side)),
            (Top::Blade, u64::from(!is_side)),
        ],
    );
    let back = pick(
        rng,
        row.back,
        &[
            (Back::None, 85),
            (Back::Spike, u64::from(is_side) * 8),
            (Back::Hammer, u64::from(is_side) * 7),
        ],
    );
    let tips = pick(rng, row.tips, &[(Tips::Single, 1)]);
    HiltRolls {
        hands,
        guard,
        pommel,
        ricasso,
        langets,
        handle_shape,
        sides: side_parts,
        top,
        back,
        tips,
    }
}
