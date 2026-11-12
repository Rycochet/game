use super::*;

#[test]
fn each_gender_has_its_own_row() {
    let rows: Vec<Row> = Gender::ALL.into_iter().map(Gender::row).collect();
    assert_eq!(rows, Row::ALL);
    assert_eq!(Gender::default(), Gender::Neutral);
}

#[test]
fn rolls_share_out_45_45_10() {
    let mut counts = [0; 3];
    for hash in 0..1000u64 {
        counts[Gender::rolled(hash) as usize] += 1;
    }
    assert_eq!(counts, [450, 450, 100]);
}

#[test]
fn a_gender_keeps_its_place_in_messages() {
    // Saves and messages carry one by its place, so new ones go at the end.
    for (place, gender) in Gender::ALL.into_iter().enumerate() {
        assert_eq!(postcard::to_allocvec(&gender).unwrap(), [place as u8]);
    }
}
