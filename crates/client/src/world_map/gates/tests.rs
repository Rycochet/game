use shared::hall::GateDesign;

use super::*;
use crate::view3d::TOKEN_DIAMETER;

/// Every design, scaled to the gate height, stands within its base as it
/// turns (the base is a token wide).
#[test]
fn every_gate_fits_its_base() {
    for design in GateDesign::ALL {
        let (width, height) = GateModel::size(design);
        let drawn = width * GATE_HEIGHT / height;
        assert!(
            drawn <= TOKEN_DIAMETER,
            "{design:?} is {drawn} wide, its base {TOKEN_DIAMETER}"
        );
    }
}
