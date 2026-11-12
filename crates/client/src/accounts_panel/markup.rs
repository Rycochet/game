//! The panel as markup: its tabs (those the role may see are put in when it
//! opens), a page for each, and why the last thing asked was refused. Only
//! the chosen tab's page is laid out; what a page shows is changed in place
//! (`show`).

use crate::dom::Template;
use crate::html;

pub(super) static ACCOUNTS: Template = html!(
    "accounts_panel.accounts",
    r#"
<form id="accounts">
  <panel id="panel" title="Accounts" moveable left="120" top="100">
    <tab-bar id="tabs">
      <option>Find</option>
      <option>Roles</option>
      <option>Audit Log</option>
    </tab-bar>
    <div id="find_page" width="520" hidden>
      <span class="columns" nowrap>
        <label for="name" width="110">Character</label>
        <input id="name" type="text" width="240"/>
        <button id="find" class="main" width="90">Find</button>
      </span>
      <div id="found" hidden>
        <p id="about"/>
        <p id="others" hidden/>
        <p id="cannot" hidden/>
        <div id="give" hidden>
          <span class="columns" nowrap>
            <label for="give_role" width="110">Role</label>
            <select id="give_role" width="240">
              <option>Player</option>
            </select>
          </span>
          <span class="columns" nowrap>
            <label for="give_for" width="110">For</label>
            <select id="give_for" width="240">
              <option>Always</option>
              <option>An hour</option>
              <option>A day</option>
              <option>A week</option>
              <option>30 days</option>
            </select>
          </span>
          <span class="columns" nowrap>
            <label for="reason" width="110">Why</label>
            <input id="reason" type="text" width="340"/>
          </span>
          <span>
            <button id="apply" class="main" width="110">Apply</button>
          </span>
        </div>
      </div>
    </div>
    <div id="roles_page" width="520" hidden>
      <p id="asking">Asking the server...</p>
      <div id="roles" hidden>
        <span class="columns" nowrap>
          <label for="role" width="110">Role</label>
          <select id="role" width="240">
            <option>player</option>
          </select>
        </span>
        <span class="columns" nowrap>
          <label for="role_name" width="110">Name</label>
          <input id="role_name" type="text" width="240"/>
        </span>
        <span class="columns" nowrap>
          <label for="role_title" width="110">Title</label>
          <input id="role_title" type="text" width="240"/>
        </span>
        <p>A title, if given, is shown instead of the name.</p>
        <span>
          <button id="save" class="main" width="110">Save</button>
        </span>
      </div>
    </div>
    <div id="audit_page" width="520" hidden>
      <p id="no_audit">Nothing has been written down yet.</p>
      <scroll id="audit" max-height="360" hidden>
        <small id="lines"/>
      </scroll>
      <span class="columns">
        <button id="newer" width="110" disabled>Newer</button>
        <button id="older" width="110" disabled>Older</button>
      </span>
    </div>
    <p id="nothing" hidden>Your role has nothing here.</p>
    <p id="refused" class="warning" hidden/>
  </panel>
</form>
"#
);
