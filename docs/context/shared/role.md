# shared: role

Account roles as both sides see them: role ids, permissions, what a role lets an account do, and what is shown of it. Which account holds which role, and every check, are the server's (`server::roles`); a client is only told its own role, to show the right menus.

## Files

- `crates/shared/src/role.rs`: `RoleId` (written as a plain string; `new`, `is`, `is_valid`: 1 to 24 of `a-z0-9-_`), the built-in ids `OWNER`, `ADMIN`, `DEVELOPER`, `MODERATOR`, `PLAYER`, `MODERATED`, `BANNED`, `GUEST` (`BUILT_IN`, highest first), `Permission` (`Debug(DebugTool)`, `ServerOptions`, `Pause`, `NeverIdle`, `PastPlayerLimit`, `LookUp`, `ReadAudit`, `RenameRoles`; `all()`; `DebugTools`, the old single permission, only read from old files and never checked), `Powers { permissions, grants, appears_as }` (`may`, `debug_tools`), `RoleBadge { id, name, title }` (`shown`: the title, or the name), `ShownRole { current, choices }` (what a character is shown as and may be, its own role first; `badge`), `is_marked` (every role but Player, Guest, Moderated and Banned gets the staff mark).

## Public API

All of the above. `protocol::ServerMessage::Welcome` carries a `RoleBadge`, `Powers` and `ShownRole`; `protocol::RoleAsk` / `RoleAnswer` and `mob::EntityLook::mark` use them too.

## Depends on

- `serde`.

## Used by

- `protocol` (`Welcome`), the server's `roles` and `connections`.

## Notes

- One role per account. Developer and Moderator don't overlap: roles are rows of permissions, not a ladder.
- `grants` are the roles an account may give, and so the roles of the accounts it may act on (Player in it means taking a role away). `appears_as` is the roles a character may be shown as to others (Settings, Gameplay, Shown As).
- One permission per debug tool (owner, 2026-11-09, for security): `role/debug_tool.rs`' `DebugTool` (each debug menu entry, Examine Mob's Kill and Destroy, and `Other` for what only the debug API and starting variables use; `ALL`), `admin_only` (World Time, Freeze Mobs: Owners and Admins only, and not given by `HOW_DEBUG_TOOLS`), `for_moderators` (what Moderators have by default; Developers have all but the Admin-only ones). `protocol::ClientMessage::dev_tool` says which a message needs. In `roles.ron` they read `Debug(GiveItem): true`.
