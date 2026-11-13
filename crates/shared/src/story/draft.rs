//! A story being made: the request, its reason, a candidate for every role
//! and colour fact, the beats chosen so far and what they use. `plot` fills
//! it; `finish` keeps only what was used.

use std::collections::BTreeSet;

use super::cast::{Cast, CastPet, Member, Role};
use super::facts::{Errand, Occasion, Passerby, Reason, Stake, TimeAgo};
use super::request::Request;
use super::told::{Facts, Lie};
use super::{Beat, Boss, Pet, roll};

/// Colour facts and the unnamed witness: whether a chosen beat uses each.
#[derive(Default)]
pub(super) struct Used {
    pub roles: BTreeSet<Role>,
    pub pet: bool,
    pub boss: bool,
    pub passerby: bool,
    pub errand: bool,
    pub time_ago: bool,
    pub occasion: bool,
    pub stake: bool,
}

pub(super) struct Draft<'a> {
    pub request: &'a Request,
    pub reason: Reason,
    /// Candidates for each role, carried or rolled.
    pub members: Vec<Member>,
    /// Whether the witness is named (a member) or a passerby.
    pub witness_named: bool,
    pub passerby: Passerby,
    pub pet: CastPet,
    pub boss: Option<Boss>,
    pub errand: Option<Errand>,
    pub time_ago: TimeAgo,
    pub occasion: Occasion,
    pub stake: Option<Stake>,
    pub lies: Vec<Lie>,
    pub beats: Vec<Beat>,
    pub used: Used,
}

impl<'a> Draft<'a> {
    pub fn new(request: &'a Request, reason: Reason) -> Self {
        let seed = request.text_seed.derive("cast");
        let carried = &request.carried;
        let kind = request.household.kind;
        let rolled = |role: Role| -> Member {
            match role {
                Role::Giver => roll::giver(&request.household, request.giver_index, seed),
                Role::Relative => roll::relative(&request.household, request.giver_index, seed),
                Role::Rival => {
                    roll::outsider(kind, role, Some(super::facts::Relation::Neighbour), seed)
                }
                Role::Recipient => {
                    let kind = request.destination.as_ref().map_or(kind, |d| d.kind);
                    roll::outsider(kind, role, Some(super::facts::Relation::Partner), seed)
                }
                Role::Witness => roll::outsider(kind, role, None, seed),
            }
        };
        let members = (Role::ALL.iter())
            .map(|&role| {
                carried
                    .member(role)
                    .cloned()
                    .unwrap_or_else(|| rolled(role))
            })
            .collect();
        let pet = (carried.pet.clone()).unwrap_or_else(|| {
            let pet = request
                .pet
                .clone()
                .unwrap_or_else(|| Pet::new(seed.derive("pet")));
            roll::pet(pet, seed)
        });
        let colour = request.text_seed.derive("facts");
        let relative = carried.member(Role::Relative);
        let elder = relative
            .and_then(|m| m.relation)
            .is_some_and(|r| r.is_elder());
        let mut rng = colour.derive("witness").rng();
        Self {
            request,
            reason,
            members,
            witness_named: carried.member(Role::Witness).is_some() || rng.range_u64(0, 2) == 0,
            passerby: roll::passerby(colour),
            pet,
            boss: (request.boss.clone()).or_else(|| carried.boss.clone()),
            errand: (carried.errand)
                .filter(|e| {
                    (request.place.as_ref()).is_some_and(|p| e.allowed_at(p.description.kind))
                })
                .or_else(|| {
                    (request.place.as_ref()).map(|p| roll::errand(p.description.kind, colour))
                }),
            time_ago: (carried.time_ago).unwrap_or_else(|| roll::time_ago(elder, colour)),
            occasion: (carried.occasion)
                .unwrap_or_else(|| roll::pick(&mut colour.derive("occasion").rng(), Occasion::ALL)),
            stake: None,
            lies: Vec::new(),
            beats: Vec::new(),
            used: Used::default(),
        }
    }

    pub fn member(&self, role: Role) -> &Member {
        (self.members.iter())
            .find(|m| m.role == role)
            .expect("every role has a candidate")
    }

    pub fn member_mut(&mut self, role: Role) -> &mut Member {
        (self.members.iter_mut())
            .find(|m| m.role == role)
            .expect("every role has a candidate")
    }

    pub fn has(&self, beat: Beat) -> bool {
        self.beats.contains(&beat)
    }

    /// The cast and facts, keeping only what the beats used.
    pub fn finish(self) -> (Cast, Facts, Vec<Lie>) {
        let used = &self.used;
        let request = self.request;
        let cast = Cast {
            members: (self.members.iter())
                .filter(|m| m.role == Role::Giver || used.roles.contains(&m.role))
                .cloned()
                .collect(),
            pet: used.pet.then(|| self.pet.clone()),
            boss: self.boss.clone().filter(|_| used.boss),
            passerby: used.passerby.then_some(self.passerby),
        };
        let facts = Facts {
            home: request.home.clone(),
            asks: request.asks.clone(),
            place: request.place.clone(),
            creatures: request.creatures,
            item: request.item,
            remains: request.remains,
            destination: request.destination.clone(),
            reward: request.reward.clone(),
            errand: self.errand.filter(|_| used.errand),
            time_ago: used.time_ago.then_some(self.time_ago),
            occasion: used.occasion.then_some(self.occasion),
            stake: self.stake.filter(|_| used.stake),
            season: request.season.filter(|_| request.higher_reward),
            weather: request.weather.filter(|_| request.higher_reward),
        };
        (cast, facts, self.lies)
    }
}
