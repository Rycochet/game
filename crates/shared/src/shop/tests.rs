use super::*;
use crate::item::Quality;
use crate::rng::Seed;
use crate::site::BuildingKind;

const fn smith(quality: u16, of_kind: u32) -> Shop {
    Shop {
        kind: BuildingKind::Weaponsmith,
        quality,
        trades: 100,
        of_kind,
    }
}

/// The qualities of what a shop would have, new stock taken as is.
fn after(stock: &[Quality], restocked: &Restocked) -> Vec<Quality> {
    let kept = restocked.keep.iter().map(|&index| stock[index]);
    let added = restocked.new.iter().map(|new| new.quality.unwrap());
    kept.chain(added).collect()
}

#[test]
fn the_same_answers_give_the_same_stock() {
    let shop = smith(500, 1);
    let first = catch_up(Seed::new(1), &shop, &[], None, None, 1_000_000);
    assert_eq!(
        first,
        catch_up(Seed::new(1), &shop, &[], None, None, 1_000_000)
    );
    assert_ne!(
        first,
        catch_up(Seed::new(2), &shop, &[], None, None, 1_000_000)
    );
}

#[test]
fn a_shop_first_opened_has_about_its_count_and_never_an_artifact() {
    for seed in 0..200 {
        let quality = (seed * 5) as u16;
        let shop = smith(quality, 1);
        let stocked = catch_up(Seed::new(seed), &shop, &[], None, None, 500_000);
        let count = stocked.new.len() as u64;
        assert!(
            (shop.minimum()..=shop.target()).contains(&count),
            "{quality}: {count}"
        );
        assert!(
            stocked
                .new
                .iter()
                .all(|new| { new.quality != Some(Quality::Artifact) && new.minute <= 500_000 })
        );
    }
}

#[test]
fn better_and_busier_shops_make_better_things() {
    let share = |shop: Shop| {
        let mut rng = Seed::new(3).rng();
        let rolls = (0..50_000).map(|_| shop.roll_quality(&mut rng));
        rolls.filter(|&quality| quality >= Quality::Rare).count()
    };
    assert!(share(smith(1000, 1)) * 2 > share(smith(500, 1)) * 3);
    assert!(share(smith(500, 1)) > share(smith(0, 1)) * 2);
    let busy = Shop {
        trades: 1000,
        ..smith(500, 1)
    };
    assert!(share(busy) > share(smith(500, 1)));
}

#[test]
fn make_time_goes_by_quality_kind_shop_and_rivals() {
    let shop = smith(500, 1);
    assert_eq!(shop.make_minutes(Quality::Common, 100), 60);
    assert_eq!(shop.make_minutes(Quality::Rare, 100), 1440);
    assert_eq!(shop.make_minutes(Quality::Common, 200), 120);
    assert_eq!(smith(0, 1).make_minutes(Quality::Common, 100), 90);
    assert_eq!(smith(1000, 1).make_minutes(Quality::Common, 100), 30);
    assert_eq!(smith(500, 4).make_minutes(Quality::Common, 100), 30);
    assert_eq!(smith(500, 4).replaced_after(), REPLACED_AFTER / 2);
}

#[test]
fn rivals_turn_stock_over_faster_at_the_same_count() {
    let stock = [Quality::Common; 15];
    let shop = |of_kind| smith(400, of_kind);
    let kept = |of_kind| {
        let last = 1_000_000;
        let at = last + 3 * MINUTES_PER_DAY;
        (0..100)
            .map(|seed| {
                catch_up(
                    Seed::new(seed),
                    &shop(of_kind),
                    &stock,
                    Some(last),
                    Some(last),
                    at,
                )
                .keep
                .len()
            })
            .sum::<usize>()
    };
    assert!(kept(9) < kept(1), "{} {}", kept(9), kept(1));
}

#[test]
fn past_the_limit_everything_has_been_replaced() {
    let shop = smith(500, 1);
    let old = [Quality::Legendary; 3];
    let now = REPLACED_AFTER * 3;
    let stocked = catch_up(Seed::new(1), &shop, &old, Some(5), Some(5), now);
    assert!(stocked.keep.is_empty());
    assert!(
        stocked
            .new
            .iter()
            .all(|new| new.minute >= now - REPLACED_AFTER)
    );
}

#[test]
fn no_time_passed_changes_nothing_even_if_the_clock_went_back() {
    let shop = smith(500, 1);
    let old = [Quality::Common; 5];
    for now in [1000, 500] {
        let stocked = catch_up(Seed::new(1), &shop, &old, Some(1000), Some(now), now);
        assert_eq!(stocked.keep, [0, 1, 2, 3, 4]);
        assert!(stocked.new.is_empty());
        assert_eq!(stocked.last_made, Some(1000));
    }
}

#[test]
fn a_shop_is_only_topped_up_after_time_away() {
    let shop = smith(500, 1);
    let bare: [Quality; 0] = [];
    let now = 1_000_000;
    let straight_back = catch_up(Seed::new(1), &shop, &bare, Some(now), Some(now), now + 1);
    assert!(straight_back.new.is_empty());
    let away = now + TOP_UP_AFTER;
    let mut topped = 0;
    for seed in 0..200 {
        let stocked = catch_up(Seed::new(seed), &shop, &bare, Some(now), Some(now), away);
        let count = stocked.new.len() as u64;
        assert!(count >= shop.minimum() && count <= shop.target(), "{count}");
        topped += (count < shop.target()) as u32;
    }
    assert!(topped > 0, "not always to the exact count");
}

#[test]
fn something_part_made_is_still_being_made_when_looked_at_again() {
    // Looking in often gets the same as not, so visits can't change the roll.
    let shop = smith(500, 1);
    let start = 1_000_000;
    let mut stock: Vec<Quality> = vec![Quality::Common; shop.target() as usize];
    let mut last_made = Some(start);
    for hours in 1..=200 {
        let now = start + hours * 60;
        let restocked = catch_up(Seed::new(9), &shop, &stock, last_made, Some(now), now);
        stock = after(&stock, &restocked);
        last_made = restocked.last_made;
    }
    let once = catch_up(
        Seed::new(9),
        &shop,
        &vec![Quality::Common; shop.target() as usize],
        Some(start),
        Some(start + 200 * 60),
        start + 200 * 60,
    );
    assert_eq!(last_made, once.last_made);
}

/// Not the numbers themselves, but that they stay sensible: rare things are
/// kept longer than they are made, without taking over.
#[test]
fn the_stock_of_a_shop_looked_in_on_stays_mostly_ordinary() {
    let (mut with_legendary, mut samples, mut total) = (0, 0, [0u64; 6]);
    for seed in 0..40 {
        let shop = smith(500, 1);
        let mut stock = Vec::new();
        let mut last_made = None;
        let mut now = 500_000;
        for visit in 0..200 {
            let restocked = catch_up(Seed::new(seed), &shop, &stock, last_made, Some(now), now);
            stock = after(&stock, &restocked);
            last_made = restocked.last_made;
            now += 7 * MINUTES_PER_DAY;
            if visit > 40 {
                samples += 1;
                with_legendary += stock.contains(&Quality::Legendary) as u32;
                for &quality in &stock {
                    total[quality as usize] += 1;
                }
            }
        }
    }
    let share = with_legendary as f64 / samples as f64;
    assert!((0.0..0.5).contains(&share), "{share}");
    let ordinary = total[Quality::Poor as usize] + total[Quality::Common as usize];
    assert!(ordinary * 3 > total.iter().sum::<u64>(), "{total:?}");
}
