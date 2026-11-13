use super::*;

#[test]
fn random_dots_add_up_and_spread_over_the_whole_triangle() {
    let mut rng = Seed::new(7).rng();
    let dots: Vec<Dot> = (0..2000).map(|_| random_dot(&mut rng)).collect();
    assert!(dots.iter().all(|dot| dot.is_valid()));
    // Each corner's weight sometimes near all and sometimes near none.
    for corner in 0..3 {
        assert!(dots.iter().any(|dot| dot.0[corner] > 900), "{corner}");
        assert!(dots.iter().any(|dot| dot.0[corner] < 100), "{corner}");
    }
    // Even on average: a third each, give or take.
    for corner in 0..3 {
        let mean = dots.iter().map(|dot| dot.0[corner] as u32).sum::<u32>() / 2000;
        assert!((300..=367).contains(&mean), "{corner}: {mean}");
    }
}
