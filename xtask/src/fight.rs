//! `cargo xtask fight`: thousands of fights between two equal median creatures,
//! by `shared::combat`'s rules, printing how long they last. For balancing combat
//! before playing it.

use std::process::ExitCode;

use shared::blade::Family;
use shared::combat::{Blow, Weapon, attack_cost, off_hand_share, roll_blow, wind_up};
use shared::item::Quality;
use shared::rng::{Rng, Seed};

/// Energy gained a second at normal speed: 210 a tick, 20 ticks a second.
const ENERGY_A_SECOND: f64 = 4200.0;
/// A median creature's Health, in thousandths.
const HEALTH: u64 = 20_000;

/// What one side fights with: a weapon in one hand or one in each.
struct Arms {
    name: &'static str,
    hands: Vec<Weapon>,
    /// Dual wielding skill as a percentage, when there are two.
    dual: u64,
}

fn blade(family: Family, grams: u32, length_mm: u32) -> Weapon {
    Weapon {
        family: Some(family),
        grams,
        length_mm,
        quality: Quality::Common,
        metal_tier: 2,
        skill: None,
    }
}

fn arms() -> Vec<Arms> {
    let dagger = blade(Family::Dagger, 300, 350);
    let sword = blade(Family::Sword, 1200, 1000);
    let polearm = blade(Family::Polearm, 2600, 2200);
    let one = |name, weapon| Arms {
        name,
        hands: vec![weapon],
        dual: 0,
    };
    let two = |name, dual| Arms {
        name,
        hands: vec![sword, dagger],
        dual,
    };
    vec![
        one("Bare hands", Weapon::UNARMED),
        one("Dagger (300 g)", dagger),
        one("Sword (1.2 kg)", sword),
        one("Polearm (2.6 kg)", polearm),
        two("Sword + dagger, untrained", 0),
        two("Sword + dagger, mastered", 100),
    ]
}

/// The seconds until each blow of one side lands, as long as `until`, and what
/// each does to the other.
fn blows(arms: &Arms, armour: u64, until: f64, rng: &mut Rng) -> Vec<(f64, u64)> {
    let dual = (arms.hands.len() > 1).then_some(arms.dual);
    let mut out = Vec::new();
    let mut ready = 0.0;
    for (n, weapon) in arms.hands.iter().cycle().enumerate() {
        let cost = attack_cost(weapon, 100, 0, dual);
        let lands = ready + f64::from(wind_up(cost)) / ENERGY_A_SECOND;
        if lands > until {
            break;
        }
        let hand = if n % arms.hands.len() == 0 {
            1000
        } else {
            off_hand_share(arms.dual)
        };
        let blow = Blow {
            weapon: *weapon,
            strength: 100,
            luck: roll_blow(rng),
            hand,
            attack: 1,
            defence: 1,
            armour,
        };
        out.push((lands, u64::from(blow.harm().milli())));
        ready += f64::from(cost) / ENERGY_A_SECOND;
    }
    out
}

/// How long one side takes to bring the other down.
fn time_to_kill(arms: &Arms, armour: u64, rng: &mut Rng) -> f64 {
    let mut left = HEALTH;
    for (at, harm) in blows(arms, armour, 600.0, rng) {
        left = left.saturating_sub(harm);
        if left == 0 {
            return at;
        }
    }
    f64::INFINITY
}

pub fn run(args: &[String]) -> ExitCode {
    let mut fights = 2000;
    let mut armour = 0;
    for arg in args {
        match arg.split_once('=') {
            Some(("fights", n)) => fights = n.parse().unwrap_or(fights),
            Some(("armour", n)) => armour = n.parse().unwrap_or(armour),
            _ => {
                eprintln!("Usage: cargo xtask fight [fights=2000] [armour=0]");
                return ExitCode::FAILURE;
            }
        }
    }
    println!(
        "{fights} fights a side between equal median creatures (20 Health, untrained, \
         armour {armour}): seconds until one falls"
    );
    println!(
        "{:<28} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "Both using", "least", "10%", "half", "90%", "most"
    );
    for (n, arms) in arms().iter().enumerate() {
        let mut rng = Seed::new(n as u64).derive("fight").rng();
        let mut times: Vec<f64> = (0..fights)
            .map(|_| {
                // A fight ends when the quicker of the two lands the last blow.
                let one = time_to_kill(arms, armour, &mut rng);
                let other = time_to_kill(arms, armour, &mut rng);
                one.min(other)
            })
            .collect();
        times.sort_by(f64::total_cmp);
        let at = |share: usize| times[(times.len() - 1) * share / 100];
        println!(
            "{:<28} {:>6.1} {:>6.1} {:>6.1} {:>6.1} {:>6.1}",
            arms.name,
            at(0),
            at(10),
            at(50),
            at(90),
            at(100)
        );
    }
    ExitCode::SUCCESS
}
