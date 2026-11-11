//! The Graphics page, as markup: a document of its own, built into Settings'
//! page for it. Each row is a label and what it sets, with the row's tip as
//! its `title`; the few tips that differ in a browser are set by code
//! (`fixed`). The options each select offers are checked against the
//! settings' own lists by a test.

use crate::dom::Template;
use crate::html;

pub(super) static GRAPHICS: Template = html!(
    "system_menu.settings.graphics",
    r#"
<form id="settings.graphics" class="rows" fills>
  <span id="preset_row" class="labelled" nowrap
        title="Sets the Graphics settings together, from fastest to best looking. Brightness and Reduce Motion are yours, and stay as they are. Changing any other makes this Custom.">
    <label for="preset" width="240">Preset</label>
    <select id="preset" width="200">
      <option>Low</option>
      <option>Medium</option>
      <option>High</option>
      <option disabled>Custom</option>
    </select>
  </span>
  <scroll id="scroll" fills>
    <div class="groups">
      <div class="rows">
        <div class="tight"><h2>Window</h2><hr/></div>
        <span id="display_mode_row" class="labelled" nowrap>
          <label for="display_mode" width="240">Display mode</label>
          <select id="display_mode" width="300">
            <option>Windowed</option>
            <option>Borderless full screen</option>
            <option>Full screen</option>
          </select>
        </span>
        <span id="resolution_row" class="labelled" nowrap>
          <label for="resolution" width="240">Resolution</label>
          <select id="resolution" width="300">
            <option>Desktop</option>
          </select>
        </span>
        <small id="browser_note" hidden>The browser looks after the window.</small>
        <span align="right">
          <button id="apply" class="main" width="120" disabled>Apply</button>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>Display</h2><hr/></div>
        <span class="labelled" nowrap
              title="The most frames the game draws each second. Lower limits use less power and run cooler, which helps laptops on battery. A big difference.">
          <label for="frame_rate" width="240">Frame Rate Limit</label>
          <select id="frame_rate" width="200">
            <option>30</option>
            <option>60</option>
            <option>120</option>
            <option>Unlimited</option>
          </select>
        </span>
        <span id="vsync_row" class="labelled" nowrap>
          <label for="vsync" width="240">VSync</label>
          <input id="vsync" type="checkbox" switch/>
        </span>
        <span class="labelled" nowrap
              title="Draws the world at a lower resolution and stretches it to fit. The interface stays sharp. Lower is faster, but blurrier. A big difference.">
          <label for="render_scale" width="240">Render Scale</label>
          <input id="render_scale" type="range" width="200"/>
          <span id="render_scale_percent" width="60"/>
        </span>
        <span class="labelled" nowrap
              title="The most lines the world is drawn with, however big the window or screen. Bigger windows are then stretched, so full screen costs no more to draw. The interface stays sharp. A big difference on large screens.">
          <label for="max_resolution" width="240">Max Resolution</label>
          <select id="max_resolution" width="200">
            <option>Off</option>
            <option>720p</option>
            <option>1080p</option>
            <option>1440p</option>
          </select>
        </span>
        <span id="anti_aliasing_row" class="labelled" nowrap>
          <label for="anti_aliasing" width="240">Anti-aliasing</label>
          <select id="anti_aliasing" width="200">
            <option>Off</option>
            <option>2×</option>
            <option>4×</option>
          </select>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>Lighting</h2><hr/></div>
        <span class="labelled" nowrap
              title="Raises the darkest parts of what you can see, at night and underground, up to just under half of daylight. It never shows anything your character can't see. Little difference to speed.">
          <label for="brightness" width="240">Brightness</label>
          <input id="brightness" type="range" width="200"/>
          <span id="brightness_percent" width="60"/>
        </span>
        <span class="labelled" nowrap
              title="Shadows cast by the sun or moons on the world map. Low has softer edges; Off saves graphics power. A medium difference.">
          <label for="shadows" width="240">Shadows</label>
          <select id="shadows" width="200">
            <option>Off</option>
            <option>Low</option>
            <option>High</option>
          </select>
        </span>
        <span class="labelled" nowrap
              title="Lights the world map as at noon, by the main sun alone, whatever the time, so its light never moves. Sites still follow the time of day. A light difference.">
          <label for="midday_sun" width="240">Midday Sun</label>
          <input id="midday_sun" type="checkbox" switch/>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>World Map</h2><hr/></div>
        <span class="labelled" nowrap
              title="How much land is drawn round you on the world map. Near draws less; Far keeps all of it drawn when you zoom in. A medium difference, most while travelling.">
          <label for="view_distance" width="240">View Distance</label>
          <select id="view_distance" width="200">
            <option>Near</option>
            <option>Normal</option>
            <option>Far</option>
          </select>
        </span>
        <span class="labelled" nowrap
              title="Shows the cloud layer above the world map. With Shadows on, clouds also darken the ground below them. A light difference.">
          <label for="clouds" width="240">Clouds</label>
          <input id="clouds" type="checkbox" switch/>
        </span>
        <span class="labelled" nowrap
              title="Rain, snow and hail falling, on the world map and in places open to the sky. Light draws half as much. A light difference.">
          <label for="precipitation" width="240">Weather</label>
          <select id="precipitation" width="200">
            <option>Off</option>
            <option>Light</option>
            <option>Full</option>
          </select>
        </span>
        <span class="labelled" nowrap
              title="Paints trees and low growth onto the world map's land. A light difference.">
          <label for="foliage" width="240">Foliage</label>
          <input id="foliage" type="checkbox" switch/>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>Sites</h2><hr/></div>
        <span class="labelled" nowrap
              title="Draws floors and walls in sites with their pictures, or in plain colours that make the grid and what lies on it easier to read. Little difference to speed.">
          <label for="floor_textures" width="240">Floor Textures</label>
          <input id="floor_textures" type="checkbox" switch/>
        </span>
        <span class="labelled" nowrap
              title="How places you have seen, but can't see now, are drawn in sites: as they are, darker, or not at all. Little difference to speed.">
          <label for="remembered" width="240">Remembered Places</label>
          <select id="remembered" width="200">
            <option>Show</option>
            <option>Dim</option>
            <option>Hide</option>
          </select>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>Motion</h2><hr/></div>
        <span class="labelled" nowrap
              title="Stills the world map's water, and makes tokens and the camera jump to their places instead of gliding. For anyone who finds movement uncomfortable. Little difference to speed.">
          <label for="reduce_motion" width="240">Reduce Motion</label>
          <input id="reduce_motion" type="checkbox" switch/>
        </span>
      </div>
      <div class="rows">
        <div class="tight"><h2>Background</h2><hr/></div>
        <span id="background_row" class="labelled" nowrap>
          <label for="background" width="240">When Not in Focus</label>
          <select id="background" width="200">
            <option>Slow down</option>
            <option>Keep drawing</option>
          </select>
        </span>
      </div>
    </div>
  </scroll>
</form>
"#
);

#[cfg(test)]
mod tests;
