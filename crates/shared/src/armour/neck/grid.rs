//! Every neck piece that may be made: the grid rolls keep to, checked by the tests.

use super::materials::materials;
use super::roll::metal;
use super::tables::{
    capes, constructions, edgings, fronts, has, has_cloth, hems, highs, lames, may_spike, may_stud,
    plates, points,
};
use super::{NeckForm, NeckKind};
use crate::armour::{Construction, Finish, Style};
use crate::material::Material;

impl NeckForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (kind, construction, style) = (self.kind, self.construction, self.style);
        let (least, most) = lames(kind, construction, self.plates, self.high);
        has(constructions(kind, style), construction)
            && has(plates(kind, construction, style), self.plates)
            && has(highs(kind, construction, self.plates), self.high)
            && (least..=most).contains(&self.lames)
            && has(points(kind, construction, style), self.pointed)
            && has(fronts(kind, construction, style), self.front)
            && has(capes(kind, construction), self.cape)
            && has(hems(kind, construction, style), self.hem)
            && has(edgings(construction, style), self.edging)
            && (!self.spikes || may_spike(kind, construction, self.plates))
            && (!self.studded || may_stud(kind, construction))
            && has(materials(construction, style), self.material)
            && self.finish_fits()
            && self.cloth.is_some() == has_cloth(construction, style)
            && self.trim.is_none_or(|trim| {
                trim == Material::Brass
                    && matches!(kind, NeckKind::Gorget | NeckKind::Bevor)
                    && construction == Construction::Plate
                    && metal(self.material)
            })
    }

    fn finish_fits(&self) -> bool {
        use Construction as C;
        match self.construction {
            C::Mail | C::RingMail | C::MailAndPlate if metal(self.material) => matches!(
                self.finish,
                Finish::Bright | Finish::Blackened | Finish::Tinned | Finish::Gilt
            ),
            C::Laminar | C::Lamellar if self.style == Style::Japanese => {
                matches!(self.finish, Finish::Lacquered(_))
            }
            _ if metal(self.material) => true,
            _ if self.material == Material::HardenedLeather => {
                matches!(self.finish, Finish::Bright | Finish::Lacquered(_))
            }
            _ => self.finish == Finish::Bright,
        }
    }
}
