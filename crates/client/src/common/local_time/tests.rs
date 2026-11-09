use super::*;

#[test]
fn a_moment_reads_as_a_date_and_time() {
    // Whatever the zone, it is a day, a month, a year and a time.
    let words = local_date_time(1_762_700_000);
    let [day, month, year, time] = words
        .split([' ', ','])
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()[..]
    else {
        panic!("{words}");
    };
    assert!(
        day.parse::<u8>().is_ok_and(|day| (1..=31).contains(&day)),
        "{words}"
    );
    assert!(MONTHS.contains(&month), "{words}");
    assert!(year == "2025" || year == "2026", "{words}");
    assert_eq!(time.len(), 5, "{words}");
}
