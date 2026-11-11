//! Reading a template's source into elements and words.

use std::borrow::Cow;

use super::cursor::Cursor;
use super::text::{decode, tidy};
use super::{MarkupError, Place, RawAttr, RawElement, RawNode};

/// The elements and words of `source`, or the first thing wrong with it.
pub(in crate::dom) fn parse(source: &'static str) -> Result<Vec<RawNode>, MarkupError> {
    nodes(&mut Cursor::new(source), None)
}

/// Everything up to the end of the element named `inside`, or of the source.
fn nodes(cursor: &mut Cursor, inside: Option<&str>) -> Result<Vec<RawNode>, MarkupError> {
    let mut found = Vec::new();
    loop {
        let at = cursor.place();
        if cursor.rest().is_empty() {
            return match inside {
                Some(name) => Err(cursor.error(format!("`<{name}>` is never closed"))),
                None => Ok(found),
            };
        } else if cursor.eat("<!--") {
            if cursor.take_until("-->").is_none() || !cursor.eat("-->") {
                return Err(MarkupError::new(at, "this comment is never closed"));
            }
        } else if cursor.eat("</") {
            let name = cursor.name()?;
            cursor.skip_space();
            if !cursor.eat(">") {
                return Err(cursor.error(format!("`</{name}` needs its `>`")));
            }
            let what = match inside {
                Some(open) if open == name => return Ok(found),
                Some(open) => format!("`</{name}>` closes `<{open}>`"),
                None => format!("`</{name}>` closes nothing"),
            };
            return Err(MarkupError::new(at, what));
        } else if cursor.eat("<") {
            found.push(RawNode::Element(element(cursor, at)?));
        } else {
            let words = tidy(cursor.take_text()).map_err(|what| MarkupError::new(at, what))?;
            if !words.is_empty() {
                found.push(RawNode::Text(words, at));
            }
        }
    }
}

/// An element, from just after its `<`.
fn element(cursor: &mut Cursor, at: Place) -> Result<RawElement, MarkupError> {
    let name = cursor.name()?;
    let mut attrs: Vec<RawAttr> = Vec::new();
    let children = loop {
        cursor.skip_space();
        if cursor.eat("/>") {
            break Vec::new();
        } else if cursor.eat(">") {
            break nodes(cursor, Some(name))?;
        } else if cursor.rest().is_empty() {
            return Err(MarkupError::new(at, format!("`<{name}` needs its `>`")));
        }
        let attr = attribute(cursor)?;
        if attrs.iter().any(|earlier| earlier.name == attr.name) {
            let what = format!("`{}` is given twice", attr.name);
            return Err(MarkupError::new(attr.at, what));
        }
        attrs.push(attr);
    };
    Ok(RawElement {
        name,
        attrs,
        children,
        at,
    })
}

/// `name`, or `name="value"` in either kind of quotes.
fn attribute(cursor: &mut Cursor) -> Result<RawAttr, MarkupError> {
    let at = cursor.place();
    let name = cursor.name()?;
    cursor.skip_space();
    if !cursor.eat("=") {
        return Ok(RawAttr {
            name,
            value: None,
            at,
        });
    }
    cursor.skip_space();
    let quote = if cursor.eat("\"") {
        "\""
    } else if cursor.eat("'") {
        "'"
    } else {
        return Err(cursor.error(format!("the value of `{name}` must be in quotes")));
    };
    let unclosed = || MarkupError::new(at, format!("the value of `{name}` is never closed"));
    let raw = cursor.take_until(quote).ok_or_else(unclosed)?;
    cursor.eat(quote);
    let value: Cow<'static, str> = decode(raw).map_err(|what| MarkupError::new(at, what))?;
    Ok(RawAttr {
        name,
        value: Some(value),
        at,
    })
}
