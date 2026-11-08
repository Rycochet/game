//! How `Skills` is written. In saves (formats people can read, RON) it is a list of
//! each node's path, practice and what rust goes by (`saved_node.rs`),
//! `[("athletics/climbing", 100, 120, 7200, 79200)]`, so the tree can change
//! between versions; a node that moved is found where it went (`old::moved`),
//! and a path no longer in the tree is dropped. A save from before the tree was a
//! map of old names to levels, `{Climbing: 1}`, read by the old names (`old`). In messages (postcard) each node goes by its number.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::learned::{Learned, NOT_SEEN};
use super::old::{moved, old_skill};
use super::saved_node::SavedNode;
use super::{SkillId, Skills, level_practice};

impl Serialize for Skills {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        let mut list = serializer.serialize_seq(Some(self.0.len()))?;
        for (id, learned) in &self.0 {
            let Learned {
                practice,
                best,
                used,
                clock,
            } = *learned;
            list.serialize_element(&(id.path(), practice, best, used, clock))?;
        }
        list.end()
    }
}

impl<'de> Deserialize<'de> for Skills {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(ByPath)
        } else {
            BTreeMap::<SkillId, Learned>::deserialize(deserializer).map(Skills)
        }
    }
}

/// An old skill's bare name, a key of the old map.
struct OldName(String);

impl<'de> Deserialize<'de> for OldName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Words;
        impl Visitor<'_> for Words {
            type Value = OldName;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an old skill's name")
            }
            fn visit_str<E: serde::de::Error>(self, words: &str) -> Result<OldName, E> {
                Ok(OldName(words.to_string()))
            }
        }
        deserializer.deserialize_identifier(Words)
    }
}

struct ByPath;

impl<'de> Visitor<'de> for ByPath {
    type Value = Skills;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("skills by path")
    }

    fn visit_seq<L: SeqAccess<'de>>(self, mut list: L) -> Result<Skills, L::Error> {
        let mut skills = Skills::default();
        while let Some(SavedNode { path, learned }) = list.next_element()? {
            let found = SkillId::find(&path).or_else(|| moved(&path).and_then(SkillId::find));
            let Some(id) = found else {
                continue;
            };
            // Two old paths moved to one node: theirs together.
            let kept = skills.0.entry(id).or_default();
            kept.practice = kept.practice.saturating_add(learned.practice);
            kept.best = kept.best.saturating_add(learned.best);
            kept.used = kept.used.max(learned.used);
            kept.clock = kept.clock.max(learned.clock);
        }
        Ok(skills)
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Skills, M::Error> {
        let mut skills = Skills::default();
        while let Some((OldName(name), level)) = map.next_entry::<OldName, u32>()? {
            if let Some(id) = old_skill(&name).and_then(SkillId::find) {
                skills.practise(id, level_practice(level));
            }
        }
        for learned in skills.0.values_mut() {
            learned.used = NOT_SEEN;
        }
        Ok(skills)
    }
}
