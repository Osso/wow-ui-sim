# Aura container carryover and private caster queries

## Contract

- `C_UnitAuras.GetRefreshCarryOverDuration` uses the same aura lookup, recast metadata, argument validation, access policy, and eligibility as `GetRefreshExtendedDuration`. It returns exactly one nullable number: `min(max(expiration - now, 0), max_carryover_seconds)`, or nil for unknown metadata or an ineligible refresh. This is a **user-approved policy, not native evidence**. The environment-local metadata map starts empty; no production duration records or percentage caps are inferred from the current aura.
- `C_UnitAurasPrivate.GetPrivateAuraCasterGUID(unitToken, auraInstanceID)` returns the supplied private record's `sourceGUID`, or nil when the record or GUID is unknown. This is **INFERRED**, not native evidence. Retail and Forever read this from typed host-supplied private-aura data.
- Private-aura entries use the documented public `AuraData` fields (**VERIFIED** as the public structure); that private entries share this shape is **INFERRED**. The selected consumers also read `hideOnPartyFrames` (`Blizzard_PrivateAurasUI.lua:443`) and `canActivePlayerDispel` (`:428`); their private-aura values are **INFERRED** from consumer usage.
- Carryover is available with `retail-12-1-5` and on Forever. Private aura data queries and caster lookup use the Rust model on Retail and Forever; legacy profiles retain the Lua compatibility store.

## Evidence

The Retail 12.1.5.70448 cache's `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:412–429` declares the carryover query and its optional spell identifier. `Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua:13–27` consumes its result as a pandemic window. The existing [refresh-duration contract](aura-refresh-duration.md) describes the inferred metadata and eligibility policy.

Forever 1.60.1.70707 also [declares the nullable numeric query](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua). Its selected [pandemic-window consumer](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua) returns no window for nil or nonpositive carryover; otherwise it subtracts carryover from expiration time. These sources establish availability and numeric interpretation, not the simulator's cap formula or missing-data policy.

The pinned `Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua:72–73` calls the private caster query with a unit token and aura instance ID. No generated declaration was identified for this private query; its behavior is not native-verified.

The [Retail and Forever private-aura regression](../../tests/blizzard_private_auras_ui_loads.rs) loads the full UI and supplies two distinct host entries. It exercises `Blizzard_PrivateAurasUI.lua:487`, `:497`, and `:1196`, plus the caster query through the selected AuraContainer source. `:1228` remains untested because the simulator has no update-callback producer. The watcher reaches `:1097`, where the selected UI calls the unimplemented `AnchorPrivateAura`; rendered-output proof remains blocked by that placement gap.

The [Retail consumer regressions](../../tests/aura_container_missing_queries.rs) exercise the original methods with two supplied carryover caps and two supplied GUIDs, plus unknown private records on legacy profiles.

The selected Forever `GetPandemicWindow` consumer must use distinct supplied
carryover values before and after post-load cleanup.
[Forever regressions](../../tests/forever_aura_carryover.rs)
