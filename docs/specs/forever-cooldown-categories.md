# Forever cooldown viewer categories

## Contract

Forever 1.60.1.69913 publishes `Enum.CooldownViewerCategory` with Essential=0,
Utility=1, TrackedBuff=2, TrackedBar=3, GroupBuff=4,
SpecAgnosticEssential=5, SpecAgnosticTracked=6, EquipSlotEssential=7,
EquipSlotTracked=8. Metadata reports MinValue=0, MaxValue=8, NumValues=9.
Non-Forever builds with the `retail-12-1-0` epoch publish the same nine
values and metadata. Non-Forever builds without that epoch retain the
simulator's earlier four-category baseline (MinValue=0, MaxValue=3,
NumValues=4); this is not proven native history for those profiles.

Sources: authenticated Forever
`Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua`,
and the pinned Retail 12.1.0 declaration in the same file, lines 71–88
(`CooldownViewerCategory`: nine explicit values 0–8, metadata 0/8/9).
The Retail cache copy is at
`~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua`.
The shared publisher is
[`src/c_api/patch_12_1_0_enums.rs`](../../src/c_api/patch_12_1_0_enums.rs)
(`CooldownViewerCategory`, lines 49–60).
[`env_init/mod.rs`](../../src/lua_api/env_init/mod.rs) calls
[`ptr::compat_bootstrap::init`](../../src/ptr/compat_bootstrap.rs) under
`retail-12-1-0`; `init` unconditionally calls the shared enum publisher.
The `client-ptr` guards select bootstrap strings, not that publisher call.
The vendor's negative HiddenActive/HiddenPassive pseudo-categories remain
vendor-defined; they are not published native enum members.

## Behavioral coverage

`tests/wowforever_cooldown_categories.rs` asserts the published values and metadata,
loads real TableUtil, CooldownViewerSettingsConstants, and
CooldownViewerSettingsDataProvider sources, and checks the provider's eight-category
ordering plus negative pseudo-category separation. GroupBuff is published but is
not part of that vendor provider list. The non-Forever epoch regression
asserts all nine explicit values and all three metadata fields. The non-Forever
regression without `retail-12-1-0` retains every original four-category baseline
assertion and also checks MinValue=0. Neither baseline asserts universal native
history. This does not establish cooldown population, interactive settings
behavior, or native conformance.
