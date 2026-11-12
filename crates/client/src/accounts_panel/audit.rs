//! The Audit Log page: the server's record of roles given, taken, run out and
//! renamed, newest first, a page at a time.

use super::state::Panel;
use crate::dom::Dom;

/// How many lines one page of the log is (`server`'s `AUDIT_PAGE`).
pub(super) const PAGE: u32 = 50;

pub(super) fn show_audit(panel: &Panel, dom: &mut Dom) {
    let none = panel.audit.is_empty();
    dom.set_flag("accounts/no_audit", "hidden", !none);
    dom.set_flag("accounts/audit", "hidden", none);
    dom.set_text("accounts/lines", panel.audit.join("\n"));
    dom.set_flag("accounts/newer", "disabled", panel.audit_skip == 0);
    dom.set_flag("accounts/older", "disabled", !panel.audit_more);
}
