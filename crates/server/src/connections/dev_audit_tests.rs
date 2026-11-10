use shared::id::Id;
use shared::protocol::{ClientMessage, DevMobTool};
use shared::role::DebugTool;

use super::tests::TestClient;

#[test]
fn debug_tools_used_go_in_the_audit_log_but_asking_does_not() {
    let mut test = TestClient::new();
    // Not the single player's own, whose use isn't logged.
    test.client.host = false;
    test.send(&[ClientMessage::DevGodMode(true)]);
    let inspect = DevMobTool::Inspect(Id(uuid::Uuid::nil()));
    test.send(&[ClientMessage::DevMobs(inspect)]);
    let lines = test.audit_lines();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("GodMode"), "{lines:?}");
}

#[test]
fn everyone_given_the_debug_tools_gets_all_but_the_admins() {
    let mut test = TestClient::new();
    test.client.host = false;
    let tools = test.client.debug_tools(&test.options());
    assert!(tools.contains(&DebugTool::GiveItem));
    assert!(!tools.contains(&DebugTool::WorldTime) && !tools.contains(&DebugTool::FreezeMobs));
    test.dev_tools = false;
    assert!(test.client.debug_tools(&test.options()).is_empty());
    // Refused, so not in the log either.
    test.send(&[ClientMessage::DevGodMode(true)]);
    assert!(test.audit_lines().is_empty());
}

#[test]
fn the_single_players_own_has_every_tool_unlogged() {
    let mut test = TestClient::new();
    test.dev_tools = false;
    assert_eq!(
        test.client.debug_tools(&test.options()).len(),
        DebugTool::ALL.len()
    );
    test.send(&[ClientMessage::DevGodMode(true)]);
    assert!(test.audit_lines().is_empty());
}
