//! Show Widgets' sample of the interface written as markup (`dom`): what
//! there is of it so far, under the widgets made the old way.

use bevy::prelude::*;

use crate::dom::{Document, Dom, Template, UiEvent, UiValue, document};
use crate::html;
use crate::widgets::UiId;

pub(super) static HTML_SAMPLE: Template = html!(
    "debug.html_sample",
    // Two hashes: a colour (`"#be3c32"`) would end a string with one.
    r##"
<form id="debug.html_sample">
  <hr/>
  <h2>From markup</h2>
  <p>
    A paragraph with a picture <img src="icon:lorc/pointy-sword" alt="A sword"/>
    among its words, wrapping when it is too long for one line of the panel.
  </p>
  <p class="warning">A warning.</p>
  <small>Small words.</small>
  <table class="striped" cellpadding="4">
    <tr><th>Piece</th><th>Shown as</th></tr>
    <tr>
      <td><label for="bar">Progress</label></td>
      <td><progress id="bar" value="60" max="100">60%</progress></td>
    </tr>
    <tr><td>Spinner</td><td><progress id="busy"/></td></tr>
    <tr><td colspan="2" align="center">A cell across both columns</td></tr>
  </table>
  <div align="right"><span>To the<br/>right</span></div>
  <h2>Controls</h2>
  <span>
    <button id="plain">Button</button>
    <button id="main" class="main" title="The main thing to do here">Main</button>
    <button id="quiet" class="quiet">Quiet</button>
    <button id="danger" class="danger">Delete</button>
    <button id="off" disabled>Disabled</button>
    <button id="icon"><img src="icon:lorc/pointy-sword"/></button>
  </span>
  <table cellpadding="4">
    <tr>
      <td><label for="tick">A checkbox</label></td>
      <td><input id="tick" type="checkbox" checked/></td>
    </tr>
    <tr>
      <td><label>A switch, in its label <input id="switch" type="checkbox" switch/></label></td>
      <td><input id="locked" type="checkbox" switch checked disabled/></td>
    </tr>
    <tr>
      <td><label for="name">Text</label></td>
      <td><input id="name" type="text" value="Words" suffix="(any)"/></td>
    </tr>
    <tr>
      <td><label for="amount">Range</label></td>
      <td><input id="amount" type="range" min="0" max="10" step="1" value="4"/></td>
    </tr>
    <tr>
      <td><label for="pick">Select</label></td>
      <td>
        <select id="pick" searchable>
          <option value="a">First</option>
          <option value="b" selected>Second</option>
          <option value="c" disabled>Third, greyed</option>
        </select>
      </td>
    </tr>
    <tr>
      <td>Toggle row</td>
      <td>
        <toggle-row id="toggle"><option>One</option><option>Two</option><option>Three</option></toggle-row>
      </td>
    </tr>
    <tr>
      <td>Key box</td>
      <td><key-box id="key" value="Shift+KeyG"/></td>
    </tr>
    <tr>
      <td>Colours</td>
      <td>
        <colour-picker id="colour">
          <option value="#be3c32" label="Red"/>
          <option value="#469646" label="Green" selected/>
          <option value="#3c5ab4" label="Blue"/>
        </colour-picker>
      </td>
    </tr>
  </table>
  <tab-bar id="tabs"><option>Tab</option><option>Other tab</option></tab-bar>
  <p id="said">Use a control, and what it says shows here.</p>
  <h2>Surfaces</h2>
  <span>
    <button id="ask">Ask in a dialog</button>
    <button id="open">Open a panel</button>
  </span>
  <scroll id="scroll" max-height="60" scrollbars="always">
    <p>A scroll area, only three lines tall,</p>
    <p>whose bar always shows,</p>
    <p>with more in it</p>
    <p>than fits,</p>
    <p>so it scrolls.</p>
  </scroll>
  <panel title="A panel">
    <p>A panel that stays where it is written.</p>
  </panel>
  <dialog id="question" title="A question" hidden>
    <p>Is this dialog made from markup?</p>
    <footer>
      <button id="no">No</button>
      <button id="yes" class="main">Yes</button>
    </footer>
  </dialog>
  <template id="row"><p>Copy <span id="number"/> of a template</p></template>
  <div id="rows"/>
  <canvas id="canvas" width="64" height="32" title="A canvas, for code to draw in"/>
</form>
"##
);

/// The panel the sample's Open a panel button opens.
static HTML_PANEL: Template = html!(
    "debug.html_panel",
    r#"
<form id="debug.html_panel">
  <panel id="panel" title="From markup" moveable left="400" top="200">
    <p>A panel that moves, made from markup.</p>
    <label>It has a switch <input id="switch" type="checkbox" switch/></label>
  </panel>
</form>
"#
);

/// How many copies of the sample's template it is filled with.
const COPIES: usize = 3;

/// Fills a new sample by its ids, as a screen's own code would.
pub(in crate::debug_menu) fn fill_html_sample(
    new: Query<&Document, Added<Document>>,
    mut dom: Dom,
) {
    for Document(template) in &new {
        if std::ptr::eq(*template, &HTML_SAMPLE) {
            for copy in 0..COPIES {
                dom.copy_template("debug.html_sample/row", "debug.html_sample/rows", copy);
                let number = format!("debug.html_sample/number.{copy}");
                dom.set_text(&number, (copy + 1).to_string());
            }
            dom.set("debug.html_sample/bar", "value", "75");
            dom.set_text("debug.html_sample/bar", "75%");
        }
    }
}

/// Shows what the sample's controls say, as a screen's own code hears it,
/// and opens and closes its dialog and its panel.
pub(in crate::debug_menu) fn show_html_events(
    mut commands: Commands,
    mut events: MessageReader<UiEvent>,
    documents: Query<(Entity, &Document)>,
    mut dom: Dom,
) {
    let id_of = |name| UiId::new("debug.html_sample", name);
    let panel = documents
        .iter()
        .find(|(_, Document(template))| std::ptr::eq(*template, &HTML_PANEL));
    for UiEvent { id, value } in events.read() {
        if id.form() == "debug.html_sample" {
            dom.set_text("debug.html_sample/said", format!("{id}: {value:?}"));
        }
        if *id == id_of("ask") {
            dom.set_flag("debug.html_sample/question", "hidden", false);
        } else if [id_of("yes"), id_of("no"), id_of("question")].contains(id) {
            dom.set_flag("debug.html_sample/question", "hidden", true);
        } else if *id == id_of("open") && panel.is_none() {
            commands.spawn(document(&HTML_PANEL));
        } else if *id == UiId::new("debug.html_panel", "panel") && *value == UiValue::Closed {
            // Its panel was closed: the rest of the document goes too.
            if let Some((document, _)) = panel {
                commands.entity(document).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sample_is_good_markup() {
        super::HTML_SAMPLE.assert_good();
        super::HTML_PANEL.assert_good();
    }
}
