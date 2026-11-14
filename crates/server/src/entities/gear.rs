//! What mobs wear and carry, giving them things, and killing them.

use shared::blade::limits_for_wielder;
use shared::creature::CreatureKind;
use shared::dungeon::line_is_clear;
use shared::id::Id;
use shared::item::{Inventory, ItemKind, MadeAt, Quality, StackLimits, coin_items};
use shared::item_form::give_blade_within;
use shared::mob::GIVE_REACH;
use shared::protocol::CharacterId;
use shared::sheet::Sheet;

use super::cells_apart;
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;
use crate::storage::{MobRecord, SiteId};

/// A new mob's belongings: it wears what it can, and keeps the rest in its
/// inventory while there is room, its money last as the fewest coins. Only what is
/// worn or a blade is made to its size.
pub(super) fn dressed(
    kind: CreatureKind,
    gear: &[(ItemKind, Option<Quality>)],
    coins: u64,
    made_at: MadeAt,
) -> Inventory {
    let mut carrying = Inventory {
        creature: kind,
        outfitted: true,
        size: kind.def().inventory,
        ..Inventory::default()
    };
    // Past half of what it can carry it is burdened: see `Sheet::load`. Its
    // kind's plain sheet is near enough for choosing what it carries.
    let lifting = Sheet::new(kind).capacity(kind.def().size) / 2;
    for &(item, quality) in gear {
        let quality = quality.unwrap_or_default();
        let mut new = crate::connections::made_for(&carrying, item, None, false, quality, made_at);
        if item.blade_family().is_some() {
            // A blade it can lift and that suits its size, or none at all.
            let spare = lifting.saturating_sub(carrying.total_weight());
            let mut limits = limits_for_wielder(kind.def().size, spare);
            let (width, height) = kind.def().inventory;
            let longest = limits.fits.map_or(u8::MAX, |(_, longest)| longest);
            limits.fits = Some((width.min(height), width.max(height).min(longest)));
            if !give_blade_within(&mut new, &limits) {
                continue;
            }
        }
        new.roll_secrets();
        // Clothes go straight on, as they may be bigger than its inventory.
        if carrying.check_wear_new(&new).is_ok() {
            let _ = carrying.wear_new(new);
        } else {
            // Their own things don't stack.
            let _ = carrying.give(new, StackLimits::NONE, &mut crate::ids::new_id);
        }
    }
    // Whole stacks, each put where it fits.
    let most = StackLimits::default().most;
    for coins in coin_items(coins, most, Some(made_at), &mut new_id) {
        let _ = carrying.give(coins, StackLimits::NONE, &mut crate::ids::new_id);
    }
    carrying
}

/// A player gives a carried item to a mob near enough and in sight. It remembers,
/// and keeps the item. What kind of creature it was.
pub(crate) fn give(
    client: &mut Client,
    world: &mut ServerWorld,
    item: Id,
    to: Id,
) -> Result<CreatureKind, &'static str> {
    let character = client.character_id.ok_or("nobody to remember")?;
    let visit = client.visit.as_mut().ok_or("not in a site")?;
    let (from, place) = (visit.position(), visit.population());
    let population = world
        .populations
        .0
        .get_mut(&place)
        .ok_or("nobody on this level")?;
    let mob = population
        .mobs
        .iter_mut()
        .find(|mob| mob.id == to)
        .ok_or("no such mob here")?;
    if cells_apart(from, mob.position) > GIVE_REACH
        || !line_is_clear(&population.level, from, mob.position)
    {
        return Err("too far away");
    }
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't hand that over")?;
    mob.memory.remember_gift(character);
    let kind = mob.kind;
    let (home_level, home, memory) = (mob.home_level, mob.home, mob.memory.clone());
    let site = mob.kept_with(place.0);
    let record = record_of(world, site, (to, home_level, home));
    record.memory = memory;
    record.carried.push(item);
    Ok(kind)
}

/// What is kept of a mob (`id`, `home_level`, `home`) on `site`: saved with the
/// level it belongs on, wherever it has got to. In the wilderness its block is
/// marked, to be kept until it resets.
pub(super) fn record_of(
    world: &mut ServerWorld,
    site: SiteId,
    (id, home_level, home): (Id, u32, (i64, i64)),
) -> &mut MobRecord {
    if site == SiteId::Wilderness {
        super::wild::touched(world, home);
    }
    let saved = world.site_mut(site).levels.entry(home_level).or_default();
    saved.records.entry(id).or_default()
}

/// A mob in the player's sight dies (dev tools). All it had falls where it stood, and
/// it is gone for good: nothing of it is kept but that its spawn is dead.
pub(crate) fn kill(
    client: &mut Client,
    world: &mut ServerWorld,
    mob: Id,
) -> Result<(), &'static str> {
    let visit = client.visit.as_mut().ok_or("not in a site")?;
    let place = visit.population();
    // Where things fall: in the wilderness the area's number isn't a level.
    let here_index = visit.index();
    let population = world
        .populations
        .0
        .get_mut(&place)
        .ok_or("nobody on this level")?;
    let index = population
        .mobs
        .iter()
        .position(|one| one.id == mob)
        .ok_or("no such mob here")?;
    let at = population.mobs[index].position;
    if !visit.can_see(at.0, at.1) {
        return Err("out of sight");
    }
    bury(world, place, here_index, mob, true);
    Ok(())
}

/// A mob on this level (`place`, whose own level is `here_index`) has died: it
/// goes, never to be made again, and if it `leaves` anything (it wasn't
/// deleted by developer tools) what it had falls where it stood and its killers
/// count it.
pub(super) fn bury(
    world: &mut ServerWorld,
    place: (SiteId, u32),
    here_index: u32,
    mob: Id,
    leaves: bool,
) {
    let Some(population) = world.populations.0.get_mut(&place) else {
        return;
    };
    let Some(index) = population.mobs.iter().position(|one| one.id == mob) else {
        return;
    };
    let dead = population.mobs.remove(index);
    let (at, size) = (dead.spot, dead.size());
    world.died.insert(mob);
    if leaves {
        let provoked = dead.harmed_by.credited(world.action_clock.ticks());
        let provoked = provoked.filter(|&who| dead.memory.provoked(&CharacterId(who)));
        world.kills.push(super::Kill {
            place,
            kind: dead.kind,
            by: dead.harmed_by.clone(),
            alignment: dead.alignment,
            provoked: provoked.collect(),
            boss: dead.is_boss(),
            escort: (dead.escort.as_ref()).map(|escort| (escort.quester, escort.quest)),
        });
    }
    population.due.retain(|&(_, id)| id != mob);
    // Its home level was waiting for it to come back.
    if !dead.way_back.is_empty()
        && let Some(home) = world.populations.0.get_mut(&(place.0, dead.home_level))
    {
        home.away = home.away.saturating_sub(1);
    }
    // A site's mob about in the wilderness is kept with its site.
    let kept_with = dead.kept_with(place.0);
    if kept_with == SiteId::Wilderness {
        super::wild::touched(world, dead.home);
    }
    // What it had is saved with the level it belongs on, wherever it has got to.
    // One not kept (developer tools', a quest's extras) is never marked dead.
    let kept = dead.is_kept();
    let site = world.site_mut(kept_with);
    let saved = site.levels.entry(dead.home_level).or_default();
    if kept {
        saved.dead.insert(dead.number);
    }
    saved.mobs.remove(&dead.number);
    saved.unsettled.remove(&dead.number);
    let record = saved.records.remove(&mob).unwrap_or_default();
    if !leaves {
        return;
    }
    let mut left = record.gear.map_or_else(Vec::new, Inventory::spilled);
    left.extend(record.carried);
    if !left.is_empty() {
        let here = world
            .site_mut(place.0)
            .levels
            .entry(here_index)
            .or_default();
        // Where it fell, in a bag already within its reach or a new one.
        crate::ground::drop_at(&mut here.ground, size, at, left);
    }
}
