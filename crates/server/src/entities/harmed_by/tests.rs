use super::*;

fn id(n: u128) -> Id {
    Id(uuid::Uuid::from_u128(n))
}

#[test]
fn only_recent_blows_share_in_a_kill() {
    let mut by = HarmedBy::default();
    by.harmed(id(1), 0);
    by.harmed(id(2), 100);
    by.harmed(id(1), 200);
    let now = 100 + KILL_CREDIT_TICKS - 1;
    let mut credited: Vec<_> = by.credited(now).collect();
    credited.sort();
    assert_eq!(credited, [id(1), id(2)], "each once, at their last blow");
    // Two minutes after their last blow, someone no longer counts.
    let now = 100 + KILL_CREDIT_TICKS;
    assert_eq!(by.credited(now).collect::<Vec<_>>(), [id(1)]);
    by.clear();
    assert_eq!(by.credited(0).count(), 0);
}
