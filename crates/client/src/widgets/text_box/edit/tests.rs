use super::*;

fn edited(text: &str, caret: usize, anchor: usize) -> Edited {
    Edited {
        text: text.to_owned(),
        caret,
        anchor,
    }
}

#[test]
fn typing_goes_in_at_the_caret() {
    let mut box_ = edited("helo", 3, 3);
    box_.insert("l");
    assert_eq!(box_, edited("hello", 4, 4));
}

#[test]
fn typing_replaces_what_is_selected() {
    let mut box_ = edited("hello world", 6, 11);
    box_.insert("there");
    assert_eq!(box_, edited("hello there", 11, 11));
}

#[test]
fn places_count_characters_not_bytes() {
    let mut box_ = edited("café au", 4, 4);
    box_.insert("!");
    assert_eq!(box_.text, "café! au");
    box_.backspace(false);
    box_.backspace(false);
    assert_eq!(box_.text, "caf au");
}

#[test]
fn arrows_move_and_stop_at_the_ends() {
    let mut box_ = edited("ab", 0, 0);
    box_.move_caret(Motion::Left, false);
    assert_eq!(box_.caret, 0);
    box_.move_caret(Motion::Right, false);
    box_.move_caret(Motion::Right, false);
    box_.move_caret(Motion::Right, false);
    assert_eq!((box_.caret, box_.anchor), (2, 2));
    box_.move_caret(Motion::Home, false);
    assert_eq!(box_.caret, 0);
    box_.move_caret(Motion::End, false);
    assert_eq!(box_.caret, 2);
}

#[test]
fn shift_grows_a_selection_and_arrows_collapse_it() {
    let mut box_ = edited("abcdef", 2, 2);
    box_.move_caret(Motion::Right, true);
    box_.move_caret(Motion::Right, true);
    assert_eq!(box_.selection(), Some((2, 4)));
    assert_eq!(box_.selected(), "cd");
    box_.move_caret(Motion::Left, false);
    assert_eq!((box_.caret, box_.anchor), (2, 2));
    box_.move_caret(Motion::End, true);
    box_.move_caret(Motion::Right, false);
    assert_eq!((box_.caret, box_.anchor), (6, 6));
}

#[test]
fn words_are_stepped_over() {
    let mut box_ = edited("one two, three", 14, 14);
    box_.move_caret(Motion::WordLeft, false);
    assert_eq!(box_.caret, 9);
    box_.move_caret(Motion::WordLeft, false);
    assert_eq!(box_.caret, 7);
    box_.move_caret(Motion::WordLeft, false);
    assert_eq!(box_.caret, 4);
    box_.move_caret(Motion::WordRight, false);
    assert_eq!(box_.caret, 7);
    box_.move_caret(Motion::WordRight, false);
    assert_eq!(box_.caret, 9);
    box_.move_caret(Motion::WordRight, false);
    assert_eq!(box_.caret, 14);
}

#[test]
fn backspace_and_delete_take_characters_or_words() {
    let mut box_ = edited("one two three", 7, 7);
    box_.backspace(true);
    assert_eq!(box_, edited("one  three", 4, 4));
    box_.delete(true);
    assert_eq!(box_, edited("one three", 4, 4));
    box_.delete(false);
    assert_eq!(box_, edited("one hree", 4, 4));
    box_.move_caret(Motion::Home, false);
    box_.backspace(false);
    assert_eq!(box_, edited("one hree", 0, 0));
}

#[test]
fn backspace_takes_a_selection_whole() {
    let mut box_ = edited("abcdef", 4, 1);
    box_.backspace(true);
    assert_eq!(box_, edited("aef", 1, 1));
}

#[test]
fn cut_gives_back_what_was_selected() {
    let mut box_ = edited("hello world", 5, 0);
    assert_eq!(box_.cut(), "hello");
    assert_eq!(box_, edited(" world", 0, 0));
    assert_eq!(box_.cut(), "");
}

#[test]
fn double_and_triple_clicks_select_a_word_and_everything() {
    let mut box_ = edited("say hello, friend", 0, 0);
    box_.select_word(6);
    assert_eq!(box_.selected(), "hello");
    box_.select_word(17);
    assert_eq!(box_.selected(), "friend");
    box_.select_word(3);
    assert_eq!(box_.selected(), " ");
    box_.select_all();
    assert_eq!(box_.selected(), "say hello, friend");
    let mut empty = edited("", 0, 0);
    empty.select_word(0);
    assert_eq!(empty.selection(), None);
}

#[test]
fn at_end_puts_the_caret_after_the_text() {
    assert_eq!(Edited::at_end("abc"), edited("abc", 3, 3));
}
