use super::*;

fn world() -> WorldParams {
    WorldParams::from_seed(Seed::new(12345))
}

fn all_rivers(rivers: &Rivers) -> Vec<River> {
    let mut found = Vec::new();
    for y in -40..40 {
        for x in 0..80 {
            found.extend(rivers.in_chunk(ChunkPos { x, y }));
        }
    }
    found
}

#[test]
fn about_the_right_share_of_chunks_have_rivers() {
    let rivers = Rivers::new(&world());
    let count = all_rivers(&rivers).len();
    // 6400 chunks at 3-30%, less those with no land.
    assert!((300..=1900).contains(&count), "{count}");
}

#[test]
fn rivers_run_downhill_one_step_at_a_time() {
    let world = world();
    let (rivers, surface) = (Rivers::new(&world), Surface::new(&world));
    let all = all_rivers(&rivers);
    for river in &all {
        for pair in river.points.windows(2) {
            let ((ax, ay), (bx, by)) = (pair[0], pair[1]);
            assert!((ax - bx).abs() <= 1 && (ay - by).abs() <= 1);
            assert!(surface.broad_height(bx, by) < surface.broad_height(ax, ay));
        }
    }
    assert!(all.iter().any(|river| river.reaches_sea));
    assert!(all.iter().any(|river| !river.reaches_sea));
}

#[test]
fn rivers_stop_at_the_sea() {
    let world = world();
    let (rivers, surface) = (Rivers::new(&world), Surface::new(&world));
    let low_tide = 2.0 * world.sea_level - world.high_water();
    for river in all_rivers(&rivers) {
        let (_, before_last) = river.points.split_last().unwrap();
        for &(x, y) in before_last {
            assert!(surface.sample(x, y).height > low_tide);
        }
    }
}

#[test]
fn wet_biomes_have_more_rivers_than_dry_ones() {
    assert!(chance_percent(Biome::Desert) < chance_percent(Biome::TropicalRainforest));
    assert!(
        Biome::ALL
            .iter()
            .all(|&b| chance_percent(b) <= MOST_LIKELY_PERCENT)
    );
}

#[test]
fn the_same_chunk_gives_the_same_river_after_a_wrap() {
    let rivers = Rivers::new(&world());
    let chunk = (0..80)
        .flat_map(|x| (-40..40).map(move |y| ChunkPos { x, y }))
        .find(|&chunk| rivers.in_chunk(chunk).is_some())
        .unwrap();
    let wrapped = ChunkPos {
        x: chunk.x + rivers.columns(),
        ..chunk
    };
    assert_eq!(rivers.in_chunk(chunk), rivers.in_chunk(wrapped));
}
