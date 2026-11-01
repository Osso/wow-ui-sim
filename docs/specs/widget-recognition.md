# Widget recognition

## Scope

Retail `C_Widget.IsWidget(value)` returns one boolean. It recognizes simulator Region-backed frame, texture and font string tables. It rejects ordinary non-tables, unrelated tables, animation groups, animations and control points. The function reads the slot-zero identity rather than recovering a missing identity from table backing. Extra arguments do not affect the result.

The pinned Retail `Blizzard_DebugTools/Blizzard_TableInspectorAttributeDataProvider.lua:47` calls this undocumented member. Retail build 70688 native evidence identifies Region compatibility and excludes Animation and AnimationGroup; the supplied static evidence does not establish every subclass or runtime state transition.

## Known gaps

Exact tainted-table access and secret-value behavior are not established by this contract. Native class construction and all subclass compatibility are not proven. Other client profiles are outside this change.
