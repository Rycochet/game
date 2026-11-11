//! Setting one attribute by its name, as markup and code both do.

use std::borrow::Cow;

use super::values::{align, count, length, number, spacing, valign};
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
        if let Some(flag) = self.flag(name) {
            // `hidden`, `hidden=""` and `hidden="hidden"` all mean yes.
            let given = value.as_deref().unwrap_or("");
            if !given.is_empty() && given != name {
                return Err(format!("`{name}` stands alone, without `{given}`"));
            }
            *flag = true;
            return Ok(());
        }
        let Some(value) = value else {
            return Err(format!("`{name}` needs a value"));
        };
        self.set_value(name, value)?;
        match self.conflict() {
            Some(conflict) => Err(conflict.into()),
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
            "for" => self.for_id = None,
            "src" => self.src = None,
            "alt" => self.alt = None,
            "hidden" => self.hidden = false,
            "disabled" => self.disabled = false,
            "inert" => self.inert = false,
            "align" => self.align = None,
            "valign" => self.valign = None,
            "top" => self.top = None,
            "left" => self.left = None,
            "right" => self.right = None,
            "bottom" => self.bottom = None,
            "width" => self.width = None,
            "height" => self.height = None,
            "margin" => self.margin = [None; 4],
            "value" => self.value = None,
            "max" => self.max = None,
            "colspan" => self.colspan = None,
            "cellspacing" => self.cellspacing = None,
            "cellpadding" => self.cellpadding = None,
            _ => self.margin[margin_side(name)?] = None,
        }
        Ok(())
    }

    /// The yes-or-no attribute of this name, if it is one.
    fn flag(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "hidden" => Some(&mut self.hidden),
            "disabled" => Some(&mut self.disabled),
            "inert" => Some(&mut self.inert),
            _ => None,
        }
    }

    fn set_value(&mut self, name: &str, value: Cow<'static, str>) -> Result<(), String> {
        match name {
            "class" => self.class = Some(value),
            "title" => self.title = Some(value),
            "for" => self.for_id = Some(value),
            "src" => self.src = Some(value),
            "alt" => self.alt = Some(value),
            "align" => self.align = Some(align(&value)?),
            "valign" => self.valign = Some(valign(&value)?),
            "top" => self.top = Some(length(name, &value)?),
            "left" => self.left = Some(length(name, &value)?),
            "right" => self.right = Some(length(name, &value)?),
            "bottom" => self.bottom = Some(length(name, &value)?),
            "width" => self.width = Some(length(name, &value)?),
            "height" => self.height = Some(length(name, &value)?),
            "margin" => self.margin = [Some(spacing(name, &value)?); 4],
            "value" => self.value = Some(number(name, &value)?),
            "max" => self.max = Some(number(name, &value)?),
            "colspan" => self.colspan = Some(count(name, &value)?),
            "cellspacing" => self.cellspacing = Some(spacing(name, &value)?),
            "cellpadding" => self.cellpadding = Some(spacing(name, &value)?),
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
