//! Time for doing things. See `docs/context/shared/energy.md`.
//!
//! A creature gains its speed in energy every tick. It can act with 0 or more,
//! and acting spends the action's cost, which can take it below 0: a dearer action
//! means a longer wait for the next, and a faster creature acts more often.
//!
//! Nothing touches a creature between actions. Only its energy and the tick that
//! was correct at are stored, and the formula gives it at any other tick, so a
//! thousand creatures with nothing to do cost nothing, and time away only refills
//! them.

use serde::{Deserialize, Serialize};

use crate::movement::step_cost;

/// How many ticks of action time pass in a second. The server's ticks, and what
/// the client counts for itself.
pub const ACTION_TICKS_PER_SECOND: f64 = 20.0;

/// What an ordinary creature gains per tick. With these costs, it walks about 12
/// cells a second in a site and 60 points a second on the world map.
pub const NORMAL_SPEED: u32 = 210;

/// A cell of a site takes this many times the energy of a point of the world map,
/// the scales being so different.
pub const SITE_SCALE: u32 = 5;

/// Taking stairs or going into or out of a site: two straight steps' worth.
pub const CHANGE_MAP_COST: u32 = 2 * 70 * SITE_SCALE;

/// How much a step in a site costs: its distance.
pub fn site_step_cost(from: (i64, i64), to: (i64, i64)) -> u32 {
    step_cost(from, to) * SITE_SCALE
}

/// How much putting on or taking off something this heavy costs: plate takes
/// longer than a hat.
pub fn dressing_cost(grams: u32) -> u32 {
    /// A hat's worth, whatever it weighs.
    const AT_LEAST: u32 = 70 * SITE_SCALE;
    AT_LEAST + grams / 10
}

/// How much energy something has, and the tick that was correct at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Energy {
    energy: i64,
    at: u64,
}

impl Energy {
    /// How much it has at `tick`, at this speed. Stops just under a tick's worth,
    /// so waiting saves nothing up, but fast creatures lose no fractions of a tick.
    pub fn at(&self, tick: u64, speed: u32) -> i64 {
        let gained = tick.saturating_sub(self.at).saturating_mul(speed as u64);
        let gained = i64::try_from(gained).unwrap_or(i64::MAX);
        self.energy
            .saturating_add(gained)
            .min(speed.max(1) as i64 - 1)
    }

    pub fn ready(&self, tick: u64, speed: u32) -> bool {
        self.at(tick, speed) >= 0
    }

    /// Spends this much at `tick`.
    pub fn spend(&mut self, tick: u64, speed: u32, cost: u32) {
        self.energy = self.at(tick, speed) - cost as i64;
        self.at = tick;
    }

    /// Records what it has now, for when the speed is about to change: the time
    /// before now went at this speed.
    pub fn settle(&mut self, tick: u64, speed: u32) {
        self.spend(tick, speed, 0);
    }

    /// The first tick at or after `tick` when it can act, at this speed.
    pub fn ready_at(&self, tick: u64, speed: u32) -> u64 {
        let now = self.at(tick, speed);
        if now >= 0 {
            return tick;
        }
        tick + (-now as u64).div_ceil(speed.max(1) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Seed;

    /// The same thing done the slow way, tick by tick, to check the formula against.
    struct Looped {
        energy: i64,
        tick: u64,
    }

    impl Looped {
        fn at(&mut self, tick: u64, speed: u32) -> i64 {
            while self.tick < tick {
                self.energy = (self.energy + speed as i64).min(speed as i64 - 1);
                self.tick += 1;
            }
            self.energy
        }
    }

    #[test]
    fn the_formula_gives_the_same_as_looping_over_every_tick() {
        for run in 0..50 {
            let seed = Seed::new(run);
            let mut stored = Energy::default();
            let mut looped = Looped { energy: 0, tick: 0 };
            let mut tick = 0;
            let mut speed = 210;
            for step in 0..300 {
                let roll = |purpose: i64| seed.at(&[step, purpose]);
                tick += roll(0) % 40;
                assert_eq!(
                    stored.at(tick, speed),
                    looped.at(tick, speed),
                    "{run} {step}"
                );
                match roll(1) % 3 {
                    0 => {
                        let cost = (roll(2) % 2000) as u32;
                        stored.spend(tick, speed, cost);
                        looped.energy -= cost as i64;
                    }
                    1 => {
                        let new_speed = 1 + (roll(3) % 400) as u32;
                        stored.settle(tick, speed);
                        looped.energy = looped.energy.min(new_speed as i64 - 1);
                        stored.energy = stored.energy.min(new_speed as i64 - 1);
                        speed = new_speed;
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn waiting_saves_nothing_up() {
        let mut energy = Energy::default();
        assert!(energy.ready(100_000, NORMAL_SPEED));
        energy.spend(100_000, NORMAL_SPEED, 350);
        assert!(!energy.ready(100_000, NORMAL_SPEED));
        // Hours later, one action is all it has.
        assert_eq!(energy.at(9_000_000, NORMAL_SPEED), NORMAL_SPEED as i64 - 1);
    }

    #[test]
    fn dearer_actions_wait_longer_and_faster_creatures_act_more_often() {
        let mut energy = Energy::default();
        energy.spend(0, 100, 250);
        assert_eq!(energy.ready_at(0, 100), 3);
        assert!(!energy.ready(2, 100));
        assert!(energy.ready(3, 100));
        // Twice as fast: half the wait, rounded up to a tick.
        assert_eq!(energy.ready_at(0, 200), 2);
    }

    #[test]
    fn fractions_of_a_tick_carry_over_so_walking_keeps_its_pace() {
        // Costing 1⅔ ticks each, 12 steps take a second's 20 ticks.
        let cost = site_step_cost((0, 0), (1, 0));
        let mut energy = Energy::default();
        let mut tick = 0;
        for _ in 0..12 {
            energy.spend(tick, NORMAL_SPEED, cost);
            tick = energy.ready_at(tick, NORMAL_SPEED);
        }
        assert_eq!(tick, 20);
    }

    #[test]
    fn plain_walking_matches_the_old_speeds() {
        let per_second = NORMAL_SPEED as f64 * ACTION_TICKS_PER_SECOND;
        let site = per_second / site_step_cost((0, 0), (1, 0)) as f64;
        assert_eq!(site, 12.0);
        assert_eq!(per_second / step_cost((0, 0), (1, 0)) as f64, 60.0);
    }

    #[test]
    fn the_heavier_it_is_the_longer_it_takes_to_put_on() {
        assert!(dressing_cost(20_000) > dressing_cost(150));
        assert_eq!(dressing_cost(0), 350);
    }
}
