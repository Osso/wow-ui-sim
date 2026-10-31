# Group member role counts

## Contract

**Best-guess, native UNKNOWN.** `GetGroupMemberCounts()` returns a fresh table with numeric `TANK`, `HEALER`, `DAMAGER`, and `NOROLE` counts. It counts the player and every member of the active modeled group using their existing assigned roles. Unassigned members count as `NOROLE`; when solo, only the player is counted. An inactive stored party is not a current group. The producer does not infer roles from class, specialization, online status, or names.

The producer is published for Retail, PTR, Mists, Era, Anniversary, and Forever, whose pinned Game-screen `Blizzard_FrameXMLUtil/Blizzard_FrameXMLUtil.toc:5,8` selects `PartyUtil.lua`. Wrath has no corresponding selected consumer or generated declaration in its pinned cache. Earlier profiles have no modeled role-assignment producer; their existing role reads remain `NONE`.

## Best-guess basis

Native behavior is unknown, including role source, player inclusion, solo, offline or pending members, and exact keys. The pinned generated API documentation does not declare `GetGroupMemberCounts`. Retail `Blizzard_FrameXMLUtil/PartyUtil.lua:108-112` reads `DAMAGER` and `NOROLE` and folds the latter into the former; `Blizzard_RaidFrame/Mainline/RaidFrameSocialView.lua:285-288` reads `TANK`, `HEALER`, and `DAMAGER`. Retail `Blizzard_RaidFrame/Blizzard_RaidFrame_Mainline.toc:6,11` selects the raid consumer for mainline. The third-party wowless `apis.yaml:35164-35173` lists the four keys; it is not native evidence. The player-inclusion and role rules above are explicitly supplied simulator assumptions, not native claims.

## Tests

`tests/group_role_counts.rs` runs the whole selected cached PartyUtil file and exercises the original display function with two distinct assigned rosters and solo/unassigned transitions. `tests/admin_pvp_guild_api.rs` covers the solo total. Native parity and other profile execution remain unverified.
