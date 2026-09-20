# Forever cooldown viewer categories

## Contract

Forever 1.60.1.69913 publishes `Enum.CooldownViewerCategory` with Essential=0,
Utility=1, TrackedBuff=2, TrackedBar=3, GroupBuff=4,
SpecAgnosticEssential=5, SpecAgnosticTracked=6, EquipSlotEssential=7,
EquipSlotTracked=8. Metadata reports MinValue=0, MaxValue=8, NumValues=9.
Other profiles retain their existing four categories and metadata.

Source: authenticated Forever `Blizzard_APIDocumentationGenerated/CooldownViewerConstantsDocumentation.lua`.
The vendor's negative HiddenActive/HiddenPassive pseudo-categories remain
vendor-defined; they are not published native enum members.

## Behavioral coverage

`tests/wowforever_cooldown_categories.rs` asserts the published values and metadata,
loads real TableUtil, CooldownViewerSettingsConstants, and
CooldownViewerSettingsDataProvider sources, and checks the provider's eight-category
ordering plus negative pseudo-category separation. GroupBuff is published but is
not part of that vendor provider list. A separate profile-isolation regression
retains earlier-profile values. This does not establish cooldown population,
interactive settings behavior, or native conformance.
