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
fn fractions_of_a_tick_carry_over_so_actions_keep_their_pace() {
    // Costing 1⅔ ticks each (a straight step's distance at `SITE_SCALE`), 12 take
    // a second's 20 ticks.
    let cost = crate::movement::STRAIGHT_COST * SITE_SCALE;
    let mut energy = Energy::default();
    let mut tick = 0;
    for _ in 0..12 {
        energy.spend(tick, NORMAL_SPEED, cost);
        tick = energy.ready_at(tick, NORMAL_SPEED);
    }
    assert_eq!(tick, 20);
}

#[test]
fn plain_walking_keeps_its_pace() {
    let per_second = NORMAL_SPEED as f64 * ACTION_TICKS_PER_SECOND;
    let site = per_second / site_step_cost((0, 0), (1, 0)) as f64;
    assert_eq!(site, 4.0);
    assert_eq!(site, SITE_CELLS_PER_SECOND);
    assert_eq!(per_second / step_cost((0, 0), (1, 0)) as f64, 60.0);
}

#[test]
fn the_heavier_it_is_the_longer_it_takes_to_put_on() {
    assert!(dressing_cost(20_000) > dressing_cost(150));
    assert_eq!(dressing_cost(0), 350);
}
