//! Generator versions: saves of sites made by older ones are forgotten.

/// Development tool: bump this to start clean. Every server forgets all it has
/// saved from before (characters and what they know, sites and players' changes
/// to them), so anyone connecting starts again, as nothing is kept on a client.
/// Every generator version goes up with it, so sites are made as new whatever
/// else has changed. The world itself stays: it comes from its seed.
pub const FORGET_BEFORE: u32 = 5;

/// The building generator's version: bump it when its output changes in a
/// way that matters (not for looks), and keep the old one runnable while
/// saved sites use it. Bumping `FORGET_BEFORE` ends that: no old version is left
/// in use, so all the code for them can go.
pub const BUILDING_VERSION: u32 = 1 + FORGET_BEFORE;

/// The version of buildings on stilts (as `BUILDING_VERSION`). A stilt hut saved
/// under `BUILDING_VERSION` is still the plain hut it was first made as.
pub const STILTS_VERSION: u32 = 1 + BUILDING_VERSION;

/// The cave generator's version (as `BUILDING_VERSION`): a cave mouth's top
/// level is a hill of rock round its caves.
pub const CAVE_VERSION: u32 = 1 + BOXED_CAVE_VERSION;

/// Caves from before: a cave mouth's top level the whole rectangle of rock.
pub const BOXED_CAVE_VERSION: u32 = 1 + FORGET_BEFORE;

/// The town generator's version (as `BUILDING_VERSION`).
pub const TOWN_VERSION: u32 = 1 + FORGET_BEFORE;

/// The farm generator's version (as `BUILDING_VERSION`).
pub const FARM_VERSION: u32 = 1 + FORGET_BEFORE;

/// The herbalist's hut generator's version (as `BUILDING_VERSION`).
pub const HERBALIST_VERSION: u32 = 1 + FORGET_BEFORE;

/// The oasis generator's version (as `BUILDING_VERSION`).
pub const OASIS_VERSION: u32 = 1 + FORGET_BEFORE;

/// The Dev Area generator's version (as `BUILDING_VERSION`).
pub const DEV_AREA_VERSION: u32 = 1 + FORGET_BEFORE;
