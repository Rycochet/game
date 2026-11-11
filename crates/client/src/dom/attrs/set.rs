//! Setting one attribute by its name, as markup and code both do.

use std::borrow::Cow;

use super::own::{Kind, known};
use super::values::{align, length, spacing, valign};
use super::{Attrs, SIDES, is_known};
use crate::dom::tag::Tag;

impl Attrs {
    /// Sets `name` on a `tag` element to `value` (`None`: standing alone, as
    /// `hidden`), or says why it can't be.
    pub fn set(
        &mut self,
        tag: Tag,
        name: &str,
        value: Option<Cow<'static, str>>,
    ) -> Result<(), String> {
        if !is_known(tag, name) {
            return Err(format!("`<{}>` has no `{name}`", tag.name()));
        }
        if let Some((name, kind)) = known(name) {
            kind.check(name, value.as_deref())?;
            let kept = value.filter(|_| kind != Kind::Flag);
            self.put(name, kept.unwrap_or_default());
        } else if let Some(flag) = self.flag(name) {
            Kind::Flag.check(name, value.as_deref())?;
            *flag = true;
        } else {
            let value = value.ok_or_else(|| format!("`{name}` needs a value"))?;
            self.set_value(name, value)?;
        }
        match self.problem(tag) {
            Some(problem) => Err(problem),
            None => Ok(()),
        }
    }

    /// Takes `name` off again: a flag turned off, anything else back to
    /// what the element does by itself.
    pub fn clear(&mut self, tag: Tag, name: &str) -> Result<(), String> {
        if !is_known(tag, name) {
            return Err(format!("`<{}>` has no `{name}`", tag.name()));
        }
        match name {
            "class" => self.class = None,
            "title" => self.title = None,
            "hidden" => self.hidden = false,
            "disabled" => self.disabled = false,
            "inert" => self.inert = false,
            "nowrap" => self.nowrap = false,
            "align" => self.align = None,
            "valign" => self.valign = None,
            "top" => self.top = None,
            "left" => self.left = None,
            "right" => self.right = None,
            "bottom" => self.bottom = None,
            "width" => self.width = None,
            "height" => self.height = None,
            "margin" => self.margin = [None; 4],
            _ if known(name).is_some() => self.own.retain(|(known, _)| *known != name),
            _ => self.margin[margin_side(name)?] = None,
        }
        Ok(())
    }

    /// Keeps one of an element's own attributes, already checked, in place of
    /// any it had of that name.
    pub(in crate::dom) fn put(&mut self, name: &'static str, value: Cow<'static, str>) {
        match self.own.iter_mut().find(|(known, _)| *known == name) {
            Some((_, old)) => *old = value,
            None => self.own.push((name, value)),
        }
    }

    /// The yes-or-no attribute every element has of this name, if it is one.
    fn flag(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "hidden" => Some(&mut self.hidden),
            "disabled" => Some(&mut self.disabled),
            "inert" => Some(&mut self.inert),
            "nowrap" => Some(&mut self.nowrap),
            _ => None,
        }
    }

    fn set_value(&mut self, name: &str, value: Cow<'static, str>) -> Result<(), String> {
        match name {
            "class" => self.class = Some(value),
            "title" => self.title = Some(value),
            "align" => self.align = Some(align(&value)?),
            "valign" => self.valign = Some(valign(&value)?),
            "top" => self.top = Some(length(name, &value)?),
            "left" => self.left = Some(length(name, &value)?),
            "right" => self.right = Some(length(name, &value)?),
            "bottom" => self.bottom = Some(length(name, &value)?),
            "width" => self.width = Some(length(name, &value)?),
            "height" => self.height = Some(length(name, &value)?),
            "margin" => self.margin = [Some(spacing(name, &value)?); 4],
            _ => self.margin[margin_side(name)?] = Some(spacing(name, &value)?),
        }
        Ok(())
    }
}

/// Which side `margin-<side>` is, as in `SIDES`.
fn margin_side(name: &str) -> Result<usize, String> {
    let side = name.strip_prefix("margin-");
    let side = side.and_then(|side| SIDES.iter().position(|known| *known == side));
    side.ok_or_else(|| format!("`{name}` can't be set"))
}
