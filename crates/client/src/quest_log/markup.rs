//! The Quest Log as markup: a moving panel with the planet chooser and the
//! list on the left, a quest in full on the right. Its rows are copies of
//! templates, written out by `show`: a quest's row by its marker's colour, as
//! an element's writing takes its class's colour only when it is made.

use crate::dom::Template;
use crate::html;

pub(super) static QUEST_LOG: Template = html!(
    "quest_log",
    r#"
<form id="quest_log">
  <panel id="panel" title="Quest Log" moveable left="10" top="12">
    <span class="apart" valign="top">
      <div width="300">
        <select id="planet" width="300">
          <option>Here</option>
        </select>
        <scroll id="list_scroll" max-height="60%">
          <div id="list" class="tight"/>
        </scroll>
      </div>
      <div width="420">
        <p id="choose" class="faint"/>
        <div id="detail" hidden>
          <h2 id="title"/>
          <p id="giver" class="faint"/>
          <p id="state"/>
          <scroll id="detail_scroll" max-height="45%">
            <div id="paras"/>
          </scroll>
          <p id="steps_head"/>
          <div id="steps" class="tight"/>
          <p id="reward"/>
          <p id="targets_head" hidden/>
          <div id="targets" class="tight"/>
          <span>
            <button id="track">Track</button>
            <button id="abandon" class="danger">Abandon</button>
          </span>
        </div>
      </div>
    </span>
    <template id="heading"><p id="heading" class="faint"/></template>
    <template id="row_yellow">
      <span id="row" class="joined">
        <span id="mark" class="mark-yellow"/>
        <button id="quest" class="quiet"/>
      </span>
    </template>
    <template id="row_blue">
      <span id="row" class="joined">
        <span id="mark" class="mark-blue"/>
        <button id="quest" class="quiet"/>
      </span>
    </template>
    <template id="row_orange">
      <span id="row" class="joined">
        <span id="mark" class="mark-orange"/>
        <button id="quest" class="quiet"/>
      </span>
    </template>
    <template id="row_grey">
      <span id="row" class="joined">
        <span id="mark" class="mark-grey"/>
        <button id="quest" class="quiet"/>
      </span>
    </template>
    <template id="line"><p id="line"/></template>
    <template id="para"><p id="para"/></template>
    <template id="step"><p id="step"/></template>
    <template id="target"><button id="target" class="quiet"/></template>
  </panel>
</form>
"#
);
