use super::*;

// Golden values: if these change, every generated world changes.
#[test]
fn golden_values() {
    let seed = Seed::new(12345);
    let mut rng = seed.rng();
    let values = [
        seed.derive("land").value(),
        seed.at(&[1, -2, 3]),
        rng.next_u64(),
        rng.next_u64(),
    ];
    assert_eq!(values, GOLDEN_VALUES);
}

const GOLDEN_VALUES: [u64; 4] = [
    15857797406316004683,
    8104792498113331671,
    2454886589211414944,
    3778200017661327597,
];

#[test]
fn purposes_are_independent() {
    let seed = Seed::new(1);
    assert_ne!(seed.derive("land"), seed.derive("weather"));
    assert_ne!(seed.at(&[1, 2]), seed.at(&[2, 1]));
}

#[test]
fn ranges_stay_in_bounds() {
    let mut rng = Seed::new(7).rng();
    for _ in 0..1000 {
        let n = rng.range_u64(10, 20);
        assert!((10..20).contains(&n));
        let f = rng.range_f64(-1.0, 1.0);
        assert!((-1.0..1.0).contains(&f));
    }
}
