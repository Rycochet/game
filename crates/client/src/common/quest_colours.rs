//! The colours of quest markers by kind (owner: yellow for normal quests, blue
//! for dailies, orange for the artifact line, grey for one that can't be
//! taken yet or is under way, red over a stranger's quest boss), as sRGB
//! bytes: `quests` draws markers in them,
//! and the theme has a badge for each (`span.mark-<colour>`) for the Quest
//! Log.

pub const QUEST_YELLOW: [u8; 3] = [255, 214, 40];
pub const QUEST_BLUE: [u8; 3] = [80, 160, 255];
pub const QUEST_ORANGE: [u8; 3] = [255, 140, 30];
pub const QUEST_GREY: [u8; 3] = [165, 165, 165];
pub const QUEST_RED: [u8; 3] = [235, 45, 45];
