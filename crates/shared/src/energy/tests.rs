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
    assert_eq!(dressing_cost(0), 2100);
}

/// Seconds at `NORMAL_SPEED`.
fn seconds(cost: u32) -> f64 {
    cost as f64 / (NORMAL_SPEED as f64 * ACTION_TICKS_PER_SECOND)
}

#[test]
fn getting_at_a_ring_takes_longer_under_heavier_things() {
    let ring = dressing_cost(5);
    assert!((seconds(ring) - 0.5).abs() < 0.01);
    let under_gloves = seconds(ring + covered_cost(100));
    assert!((1.8..=2.2).contains(&under_gloves), "{under_gloves}");
    let under_gauntlets = seconds(ring + covered_cost(860));
    assert!((5.0..=6.0).contains(&under_gauntlets), "{under_gauntlets}");
}

#[test]
fn cancelling_gives_back_all_but_the_penalty() {
    let mut energy = Energy::default();
    energy.spend(0, 210, 4200);
    energy.cancel(2, 210, 420);
    assert_eq!(energy.at(2, 210), -420);
    assert_eq!(energy.ready_at(2, 210), 4);
    // Owing less than the penalty already, nothing changes.
    let mut nearly = Energy::default();
    nearly.spend(0, 210, 500);
    nearly.cancel(2, 210, 420);
    assert_eq!(nearly.at(2, 210), -80);
}

#[test]
fn sneaking_holds_steps_to_a_cell_every_two_seconds() {
    let step = site_step_cost((0, 0), (1, 0));
    let sneaking = sneak_step_energy(step, 1, NORMAL_SPEED);
    let seconds = f64::from(sneaking) / f64::from(NORMAL_SPEED) / ACTION_TICKS_PER_SECOND;
    assert_eq!(seconds, 2.0);
    // A jump covers two cells: four seconds.
    assert_eq!(sneak_step_energy(2 * step, 2, NORMAL_SPEED), 2 * sneaking);
    // Something already slower stays as it is.
    assert_eq!(
        sneak_step_energy(u32::MAX / 2, 1, NORMAL_SPEED),
        u32::MAX / 2
    );
}

#[test]
fn moves_between_middles_cost_what_steps_do() {
    use crate::spot::Spot;
    let from = Spot::middle_of((3, -2));
    for (dx, dy) in [(1, 0), (0, -1), (1, 1), (-1, 1), (2, 0), (2, -2), (0, 2)] {
        let to = Spot::middle_of((3 + dx, -2 + dy));
        let cells = site_step_cost((3, -2), (3 + dx, -2 + dy));
        assert_eq!(site_move_cost(from, to), cells, "{dx}, {dy}");
    }
    // Half a cell costs half a step, rounded.
    assert_eq!(
        site_move_cost(from, from.offset(120, 0)),
        35 * SITE_STEP_SCALE
    );
}
