//! Show Widgets' sample of the interface written as markup (`dom`): what
//! there is of it so far, under the widgets made the old way.

use bevy::prelude::*;

use crate::dom::{Document, Dom, Template};
use crate::html;

pub(super) static HTML_SAMPLE: Template = html!(
    "debug.html_sample",
    r#"
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
  <template id="row"><p margin-bottom="0">Copy <span id="number"/> of a template</p></template>
  <div id="rows"/>
  <canvas id="canvas" width="64" height="32" title="A canvas, for code to draw in"/>
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

#[cfg(test)]
mod tests {
    #[test]
    fn the_sample_is_good_markup() {
        super::HTML_SAMPLE.assert_good();
    }
}
