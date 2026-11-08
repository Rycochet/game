use super::*;
use crate::movement::gait::MOVEMENT_SKILLS;
use crate::skill::{SkillId, Skills};

/// How many hours of practising `amount` every 5 seconds it takes to reach each
/// of these percentages.
fn hours_to(id: SkillId, amount: u32, percents: &[u32]) -> Vec<f64> {
    let mut skills = Skills::default();
    let mut windows = 0u64;
    percents
        .iter()
        .map(|&percent| {
            while skills.value(id) < percent * 100 {
                skills.practise(id, amount);
                windows += 1;
            }
            windows as f64 * 5.0 / 3600.0
        })
        .collect()
}

#[test]
fn flat_ground_teaches_no_climbing() {
    assert_eq!(climb_practice(0), 0);
    assert_eq!(climb_practice(1), 0);
    assert_eq!(climb_practice(2), CLIMB_PRACTICE_PER_ELEVATION);
    assert_eq!(climb_practice(1000), MOST_CLIMB_PRACTICE);
}

/// Prints the curves for balancing (`--nocapture`), and checks they stay about
/// where the plan put them: 50% after roughly 10 hours.
#[test]
fn movement_practice_curve() {
    let percents = [25, 50, 100];
    let running = hours_to(MOVEMENT_SKILLS.running, RUN_PRACTICE, &percents);
    // A mountain step of about 4 elevations (100 m).
    let climbing = hours_to(MOVEMENT_SKILLS.climbing, climb_practice(4), &percents);
    for (name, hours) in [("Running", &running), ("Climbing", &climbing)] {
        let line: Vec<_> = percents
            .iter()
            .zip(hours.iter())
            .map(|(percent, hours)| format!("{percent}% after {hours:.1} h"))
            .collect();
        println!("{name}: {}", line.join(", "));
    }
    for hours in [&running, &climbing] {
        assert!((7.0..14.0).contains(&hours[1]), "50% after {} h", hours[1]);
    }
}
