//! How `Skills` is written. In saves (formats people can read, RON) it is a list of
//! each node's path and practice, `[("athletics/climbing", 100)]`, so the tree can
//! change between versions; a node that moved is found where it went (`old::moved`),
//! and a path no longer in the tree is dropped. A save from before the tree was a
//! map of old names to levels, `{Climbing: 1}`, read by the old names (`old`). In messages (postcard) each node goes by its number.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::old::{moved, old_skill};
use super::{SkillId, Skills, level_practice};

impl Serialize for Skills {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        let mut list = serializer.serialize_seq(Some(self.0.len()))?;
        for (id, practice) in &self.0 {
            list.serialize_element(&(id.path(), practice))?;
        }
        list.end()
    }
}

impl<'de> Deserialize<'de> for Skills {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(ByPath)
        } else {
            BTreeMap::<SkillId, u32>::deserialize(deserializer).map(Skills)
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
        while let Some((path, practice)) = list.next_element::<(String, u32)>()? {
            let found = SkillId::find(&path).or_else(|| moved(&path).and_then(SkillId::find));
            if let Some(id) = found {
                let kept = skills.0.entry(id).or_insert(0);
                *kept = kept.saturating_add(practice);
            }
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
        Ok(skills)
    }
}
