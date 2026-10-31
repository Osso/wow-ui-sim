# Group member role counts

## Native contract: Retail build 70688

`GetGroupMemberCounts()` returns one table. With no group or an empty group, all role and class counts are zero. Otherwise it scans up to 40 slots in the current party or raid container, including the player. A member with flag `0x10` is skipped. Class IDs outside the class range count as class 1. Every class has a numeric key even when its count is zero; the evidence indicates the key is most likely the class name, but its exact string resolution is not fully decoded.

For each non-skipped member, native role classification checks the two supplied flag bytes `Stage` and `Spec` in order: `Stage & 0x2` counts as `TANK`; otherwise `Stage & 0x4` counts as `HEALER`; otherwise `(Stage & 0xA) != 0x2` and `Spec & 0x8` counts as `DAMAGER`; otherwise it counts as `NOROLE`. The names and meaning of these bits and the skipped-member flag remain unknown. The function fires no events.

## Simulator coverage and known gaps

Solo returns zero role and class counts. The existing simulator counts the player and up to 39 other members of the active modeled party using assigned roles and class display names; invalid class IDs count as class 1. It does not yet model the two native role bytes, the skip flag, or a separate raid container. Thus its grouped result is not native parity. The exact class key spelling remains uncertain. Other profiles retain this shared implementation; build 70688 evidence does not establish their native behavior.

The pinned generated API documentation does not declare this global. Retail `Blizzard_FrameXMLUtil/Blizzard_FrameXMLUtil.toc:5,8` selects `PartyUtil.lua` for the Game screen. `PartyUtil.lua:108-112` reads `DAMAGER` and `NOROLE` and folds the latter into the former.

## Tests

`tests/group_role_counts.rs` loads the selected original PartyUtil and exercises two assigned rosters, class counts, invalid class IDs, and solo transitions. `tests/admin_pvp_guild_api.rs` covers the solo total. These tests do not establish the missing native behaviors above.
