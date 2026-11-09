# shared: role

Account roles as both sides see them: role ids, permissions, what a role lets an account do, and what is shown of it. Which account holds which role, and every check, are the server's (`server::roles`); a client is only told its own role, to show the right menus.

## Files

- `crates/shared/src/role.rs`: `RoleId` (written as a plain string; `new`, `is`, `is_valid`: 1 to 24 of `a-z0-9-_`), the built-in ids `OWNER`, `ADMIN`, `DEVELOPER`, `MODERATOR`, `PLAYER`, `MODERATED`, `BANNED`, `GUEST` (`BUILT_IN`, highest first), `Permission` (`DebugTools`, `ServerOptions`, `Pause`, `NeverIdle`, `PastPlayerLimit`, `LookUp`, `ReadAudit`, `RenameRoles`; `ALL`), `Powers { permissions, grants, appears_as }` (`may`), `RoleBadge { id, name, title }` (`shown`: the title, or the name).

## Public API

All of the above. `protocol::ServerMessage::Welcome` carries a `RoleBadge` and `Powers`.

## Depends on

- `serde`.

## Used by

- `protocol` (`Welcome`), the server's `roles` and `connections`.

## Notes

- One role per account. Developer and Moderator don't overlap: roles are rows of permissions, not a ladder.
- `grants` are the roles an account may give, and so the roles of the accounts it may act on (Player in it means taking a role away). `appears_as` is for showing characters as another role (planned).
- Permissions are one for all debug tools for now; a later task splits them per tool.
