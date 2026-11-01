//! Typing into the filter on a device with no keyboard. A canvas can't raise the
//! on-screen keyboard, so in the browser a hidden HTML input takes the typing.
//! Browsers only let it take focus during the tap itself, not a frame later, so
//! a listener on the canvas does that, using the box's place sent each frame.
//! Native builds have real keys, and all of this does nothing.

use bevy::math::Rect;

/// A key pressed in the hidden input, which the game's own keys don't see.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Key {
    Return,
    Escape,
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use super::{Key, Rect};

    pub fn set_box(_place: Option<Rect>) {}
    pub fn show_text(_text: &str) {}
    pub fn typed(_current: &str) -> Option<String> {
        None
    }
    pub fn take_keys() -> Vec<Key> {
        Vec::new()
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{HtmlElement, HtmlInputElement, KeyboardEvent, PointerEvent};

    use super::{Key, Rect};

    struct Hidden {
        input: HtmlInputElement,
        place: Rc<RefCell<Option<Rect>>>,
        keys: Rc<RefCell<Vec<Key>>>,
    }

    thread_local! {
        static HIDDEN: Option<Hidden> = make_hidden();
    }

    fn make_hidden() -> Option<Hidden> {
        let document = web_sys::window()?.document()?;
        let canvas: HtmlElement = document.query_selector("#game").ok()??.dyn_into().ok()?;
        let input: HtmlInputElement = document.create_element("input").ok()?.dyn_into().ok()?;
        let style = input.style();
        // Under 16 px, iPads zoom the page to the input.
        for (name, value) in [
            ("position", "fixed"),
            ("top", "0"),
            ("left", "0"),
            ("width", "1px"),
            ("height", "1px"),
            ("opacity", "0"),
            ("font-size", "16px"),
        ] {
            style.set_property(name, value).ok()?;
        }
        input.set_attribute("autocomplete", "off").ok()?;
        input.set_attribute("autocapitalize", "off").ok()?;
        document.body()?.append_child(&input).ok()?;

        let place = Rc::new(RefCell::new(None::<Rect>));
        let on_lift = {
            let (place, input, canvas) = (place.clone(), input.clone(), canvas.clone());
            Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let corner = canvas.get_bounding_client_rect();
                let at = bevy::math::Vec2::new(
                    event.client_x() as f32 - corner.left() as f32,
                    event.client_y() as f32 - corner.top() as f32,
                );
                if place
                    .borrow()
                    .is_some_and(|box_at: Rect| box_at.contains(at))
                {
                    let _ = input.focus();
                }
            })
        };
        canvas
            .add_event_listener_with_callback("pointerup", on_lift.as_ref().unchecked_ref())
            .ok()?;
        // (It lasts as long as the page.)
        on_lift.forget();

        let keys = Rc::new(RefCell::new(Vec::new()));
        let on_key = {
            let keys = keys.clone();
            Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
                match event.key().as_str() {
                    "Enter" => keys.borrow_mut().push(Key::Return),
                    "Escape" => keys.borrow_mut().push(Key::Escape),
                    _ => {}
                }
            })
        };
        input
            .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())
            .ok()?;
        on_key.forget();

        Some(Hidden { input, place, keys })
    }

    /// Where the filter box is, in logical pixels; `None` when the menu is shut,
    /// which also puts the on-screen keyboard away.
    pub fn set_box(place: Option<Rect>) {
        HIDDEN.with(|hidden| {
            if let Some(hidden) = hidden {
                if place.is_none() && hidden.place.borrow().is_some() {
                    let _ = hidden.input.blur();
                }
                *hidden.place.borrow_mut() = place;
            }
        });
    }

    pub fn show_text(text: &str) {
        HIDDEN.with(|hidden| {
            if let Some(hidden) = hidden {
                hidden.input.set_value(text);
            }
        });
    }

    /// What has been typed there, if it differs from `current`.
    pub fn typed(current: &str) -> Option<String> {
        HIDDEN.with(|hidden| {
            let value = hidden.as_ref()?.input.value();
            (value != current).then_some(value)
        })
    }

    pub fn take_keys() -> Vec<Key> {
        HIDDEN.with(|hidden| {
            hidden.as_ref().map_or_else(Vec::new, |hidden| {
                hidden.keys.borrow_mut().drain(..).collect()
            })
        })
    }
}

pub(super) use platform::{set_box, show_text, take_keys, typed};
