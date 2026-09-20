# Forever LFG categories

## Contract

Forever exposes `LE_LFG_CATEGORY_LAIR = 8` and `NUM_LE_LFG_CATEGORYS = 8`. Reuse the existing Lair constant without enabling a retail epoch. Other profiles retain their existing publication and category count.

## Evidence

Ketho/BlizzardInterfaceResources, `forever` revision `659e8042049df854c114714f8ecd640823a1cd5c`: `README.md` identifies `GetBuildInfo()` as `1.60.1`, `69913`, interface `16001`; `Resources/LuaEnum.lua` publishes both values. Local retrieved artifacts and provenance: `/tmp/forever-wiki-research/`. This is exact-build extracted evidence, not an inference from retail.

## Consumer verification

`tests/wowforever_lfg.rs` executes the complete cached `Blizzard_FrameXMLBase/Constants.lua`, removing only its UTF-8 BOM as the loader does. It asserts both constants, the Lair and dungeon name mappings, and the later `MAX_WORLD_PVP_QUEUES = 2` assignment. Before publication the chunk fails with `table index is nil`. This focused test does not establish clean full startup.
