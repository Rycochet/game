//! What choosing a beat changes in a story being made: which roles and facts
//! it uses (so they are kept), and the states it sets. States only move
//! forward (`State::can_become`); `fits` has already checked that.

use super::Beat;
use super::cast::Role;
use super::draft::Draft;
use super::facts::{Stake, State, TimeAgo, Trait};

impl Draft<'_> {
    fn use_role(&mut self, role: Role) {
        self.used.roles.insert(role);
    }

    fn set_relative(&mut self, state: State) {
        self.use_role(Role::Relative);
        let relative = self.member_mut(Role::Relative);
        if relative.state.can_become(state) {
            relative.state = state;
        }
    }

    /// A witness: named, or a passerby.
    fn use_witness(&mut self) {
        if self.witness_named {
            self.use_role(Role::Witness);
        } else {
            self.used.passerby = true;
        }
    }

    fn set_stake(&mut self, stake: Stake) {
        self.stake = Some(stake);
        self.used.stake = true;
    }

    /// Something recent: days or weeks ago.
    fn recently(&mut self) {
        if self.time_ago > TimeAgo::Weeks {
            self.time_ago = TimeAgo::Days;
        }
        self.used.time_ago = true;
    }

    pub(super) fn apply(&mut self, beat: Beat) {
        use Beat::*;
        self.beats.push(beat);
        match beat {
            RelativeWent => {
                let state = self.went_state();
                self.set_relative(state);
                self.used.errand = true;
                self.used.time_ago = true;
            }
            HeirloomLost => {
                self.use_role(Role::Relative);
                self.used.time_ago = true;
            }
            DrivenOff | GiverWasThere => self.used.time_ago = true,
            PetStrayed => {
                self.used.pet = true;
                self.pet.state = State::Missing;
                self.recently();
            }
            RumourHeard => self.use_witness(),
            NeighbourWrong | Accused | RivalSeeksIt | NotGuilty | WhoToHelp | Honour => {
                self.use_role(Role::Rival)
            }
            DebtOwed => self.use_role(Role::Recipient),
            OrderPending if self.request.destination.is_some() => self.use_role(Role::Recipient),
            ThreatSeen => self.recently(),
            DiedThere => {
                self.set_relative(State::Dead);
                self.used.errand = true;
                self.used.time_ago = true;
            }
            DyingWish => self.set_relative(State::Dead),
            Murdered => {
                self.set_relative(State::Dead);
                self.use_role(Role::Rival);
            }
            BossNamed | BossWasOneOfUs => self.used.boss = true,
            SickAtHome => self.set_relative(State::Sick),
            Eloped => self.set_relative(State::Gone),
            FirstTryFailed => self.member_mut(Role::Giver).state = State::Hurt,
            GiverCannot => {
                let giver = self.member_mut(Role::Giver);
                if !matches!(giver.nature, Trait::Old | Trait::Fearful)
                    && giver.state != State::Hurt
                {
                    giver.nature = Trait::Fearful;
                }
            }
            TimePressure => self.used.occasion = true,
            ForOccasion => {
                self.used.occasion = true;
                self.set_stake(Stake::Occasion);
            }
            LeftOnPurpose => self.set_relative(State::Gone),
            SpareOrSlay => self.used.boss = true,
            Memory => self.set_stake(Stake::Memory),
            Safety => self.set_stake(Stake::Safety),
            Livelihood => self.set_stake(Stake::Livelihood),
            ChildMisses => self.set_stake(Stake::Child),
            Revenge => {
                self.use_role(Role::Rival);
                self.set_stake(Stake::Honour);
            }
            Silence => self.use_witness(),
            StealFrom | Frame => self.use_role(Role::Rival),
            LearnFate | Bring | Promise | LayToRest => self.use_role(Role::Relative),
            BringPet => self.used.pet = true,
            Take => self.use_role(Role::Recipient),
            _ => {}
        }
        match beat {
            Honour => self.set_stake(Stake::Honour),
            Promise => self.set_stake(Stake::Promise),
            _ => {}
        }
    }
}
