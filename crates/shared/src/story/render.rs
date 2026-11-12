//! Writing a story in a language: its paragraphs, title, step words and
//! ending. The client does this; the server never renders. A beat's wording
//! is one of its message's attributes (`beat-<key>.<name>`), chosen as
//! `hash(text seed, beat) % wordings`, so languages needn't have as many.

use super::told::{Ending, Story, Truth};
use super::{Beat, words};
use crate::lang::{Args, Lang, RenderError, Selector};
use crate::rng::Seed;

/// The message or attribute to render for one of `id`'s wordings, picked by
/// the seed and what is being worded.
fn wording(lang: &Lang, seed: Seed, id: &str) -> String {
    let attributes = lang.attributes(id);
    if attributes.is_empty() {
        return id.to_string();
    }
    let which = seed.derive("wording").derive(id).value() % attributes.len() as u64;
    format!("{id}.{}", attributes[which as usize])
}

impl Story {
    /// The message id this story renders `beat` with in `lang`.
    pub fn wording_of(&self, lang: &Lang, beat: Beat) -> String {
        wording(
            lang,
            self.text_seed,
            &format!("{}-{}", beat.fact(), beat.key()),
        )
    }

    fn beats_text(&self, lang: &Lang, beats: &[Beat], args: &Args) -> Result<String, RenderError> {
        let mut sentences = Vec::new();
        for &beat in beats {
            sentences.push(lang.render(&self.wording_of(lang, beat), args)?);
        }
        Ok(sentences.join(" "))
    }

    /// The story's paragraphs, each its beats' sentences.
    pub fn paragraphs_text(&self, lang: &Lang) -> Result<Vec<String>, RenderError> {
        let args = self.args(lang)?;
        (self.paragraphs.iter())
            .map(|beats| self.beats_text(lang, beats, &args))
            .collect()
    }

    /// The quest's name, from its reason (`story-title-<reason>`).
    pub fn title(&self, lang: &Lang) -> Result<String, RenderError> {
        let id = format!("story-title-{}", self.reason.key());
        lang.render(&wording(lang, self.text_seed, &id), &self.args(lang)?)
    }

    /// A step's few words ("Slay kobolds 2/6"): ask `index`, with `done` of
    /// its count so far when counted.
    pub fn step_text(&self, lang: &Lang, index: usize, done: u32) -> Result<String, RenderError> {
        let Some(ask) = self.facts.asks.get(index) else {
            return Err(RenderError {
                id: format!("step {index}"),
                text: String::new(),
                problems: vec!["no such step".to_string()],
            });
        };
        let mut args = self.args(lang)?;
        if let Some(count) = ask.count {
            args = (args.count("count", i64::from(count))).count("done", i64::from(done));
        }
        // A boss alone, with no creatures to name: the place instead.
        let words = if ask.goal == super::Goal::Slay && self.facts.creatures.is_none() {
            "place"
        } else if ask.count.is_some() {
            "counted"
        } else {
            "plain"
        };
        lang.render(&format!("step-{}.{words}", ask.goal.key()), &args)
    }

    /// The last step's words: "Return to Mara Tull".
    pub fn return_text(&self, lang: &Lang) -> Result<String, RenderError> {
        lang.render("step-return", &self.args(lang)?)
    }

    /// What the giver says at turn-in, as one paragraph.
    pub fn ending_text(&self, lang: &Lang, ending: &Ending) -> Result<String, RenderError> {
        let mut args = self.args(lang)?;
        if let Some(lie) = ending.lies.first() {
            args = args.text("liar-name", self.role_name(lang, lie.liar)?);
            let giver = if lie.liar == super::Role::Giver {
                "yes"
            } else {
                "no"
            };
            args = args.text("liar-is-giver", giver);
            args = match lie.truth {
                Truth::Errand(errand) => args.text("truth", lang.phrase(&errand)?),
                Truth::TimeAgo(when) => args.text("truth", lang.phrase(&when)?),
            };
        }
        self.beats_text(lang, &ending.beats, &args)
    }
}

/// The reward alone in words, for panels that list it apart from the text.
pub fn reward_text(lang: &Lang, parts: &[super::RewardPart]) -> Result<String, RenderError> {
    words::reward(lang, parts)
}
