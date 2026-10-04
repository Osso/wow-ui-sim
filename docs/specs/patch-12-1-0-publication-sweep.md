# Retail 12.1.0 supplemental publication sweep

A data-driven breadth probe covers all 778 entries in the [wikitext register](../../data/patch-api/sources/12.1.0-wikitext-register.json), guided by the [supplemental triage](../../data/patch-api/evidence/12.1.0-session-2026-10-04/triage-wikitext.md). This proves publication (or absence) only, not behavior. Added/changed rows earn at most `partial-development-green`; removed symbols may earn `bounded-coverage` only when their whole contract is absence.

## What it must do

- [x] Probe every unique source ID across global-api, framexml, scriptobjects, widgets, events and cvars, for added, removed and changed directions, in one fully loaded cached Game environment.
- [x] Classify raw global publication; raw namespace/mixin member publication; normal method resolution on objects made by real widget/scriptobject factories. Added/changed members must resolve to functions without accepting fabricated namespace fallbacks. Removed namespace/mixin members must be nil under both raw and ordinary lookup.
- [x] Probe concrete events through a real frame's `RegisterEvent` and registration state, preserving return/error details. Restricted-event false returns do not mean absence if the frame registered the event.
- [x] Query CVar current and default values: both nonnil for added/changed and both nil for removed. Report page-default mismatches separately without changing publication success.
- [x] Emit per-ID `{expected, observed, ok}` JSON to `P1210_SWEEP_OUT`, when set, before comparing the actual non-ok ID set with the committed known-gap list. Both newly failing and newly successful known-gap rows require review; the test never updates its own expectations.
- [x] Preserve unsupported factories, malformed symbols, wildcard inventory occurrences and probe errors as explicit non-ok observations rather than silently granting coverage. Reject missing/duplicate inventory rows and invalid section/direction data.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md) — cached Blizzard Game loading.
- [Lua API](../lua-api.md) — publication and factory surface.
- [Event system](../event-system.md) — event registerability.

## Implementation inventory

- `tests/patch_12_1_0_publication_sweep.rs` — register-driven classifier and one environment per sweep.
- `tests/data/patch_12_1_0_sweep_known_gaps.json` — reviewed exact set of non-ok IDs; membership grants no coverage.
- `tests/common/prefork_full_ui_preload.rs` — full cached Game preload using the same startup discovery as `wow-sim`, including LoD `[Bootstrap]`-only nodes (bootstrap files run; the addon stays unloaded and gets no `ADDON_LOADED`).

## Tests asserting this spec

- `tests/patch_12_1_0_publication_sweep.rs::patch_12_1_0_publication_sweep` — publication/absence breadth and exact known-gap comparison.
- `P1210_SWEEP_REGISTER` optionally selects a complete scratch register for negative controls; normal runs use the embedded committed register. Row-count, unique-ID and section/direction validation still apply. Three deliberately wrong global/member/event expectations produced exactly three additional non-ok rows; scratch data is not committed.

### Object-kind probe contract

| Register owner | Factory |
|---|---|
| Frame, FrameScriptObject, ScriptRegion | `CreateFrame('Frame')`; latter two are base interfaces |
| FontString, TextureBase, VectorGraphics | Frame's corresponding `Create*` factory |
| Minimap, StatusBar | `CreateFrame(owner)` |
| AnimationGroup, Animation | Frame's animation group, then its default animation |
| RadialProgress | Group's typed animation factory, guarded by exact `GetObjectType()` |
| DurationTextBinding, SecondsFormatter | `C_DurationUtil` / `C_StringUtil` constructors |

No per-symbol exceptions. Widget method lookup proves reachability, not owner restrictions.

Default Retail sweep: GREEN against the reviewed gap set, 778 observations (726 OK, 52 non-OK). GREEN proves the gap set is unchanged, not that all symbols work. Three negative controls were non-OK. Cached UI provenance reports Retail `12.1.0.69933`; this is provenance metadata, not native-client authentication.

## Known gaps (current cycle)

- [ ] Default Game startup has 52 non-ok rows: 39 global-api (30 added rows owned by a separate pass), 11 framexml, one widget method and one event family. These are startup-surface gaps, not proof of native-client absence. All remain non-ok even when the exact-set regression test passes.
- [ ] Removed rows republished by cached Blizzard Lua with the default `loadDeprecationFallbacks=1` stay non-ok: `getglobal`, `setglobal`, `GetWeaponEnchantInfo`, `CancelItemTempEnchantment`, `GetInspectSpecialization`, `C_UnitAuras.Add/RemovePrivateAuraAppliedSound`, `C_HousingLayout.GetNumFloors`, `C_DyeColor.GetDyeColorForItem[Location]`, `C_Housing.IsInsideOwnHouse`, `C_SuperTrack.GetNextWaypointForMap` (`Blizzard_Deprecated` 12.1.0 files), `BNGetFriendInviteInfo`, `BNSendVerifiedBattleTagInvite` (`Blizzard_DeprecatedBattleNet`), `RaidNotice_*` (`Blizzard_DeprecatedRaidWarning`). `MacroFrame_SaveMacro` comes from `Blizzard_MacroUI`, which startup loads.
- [ ] `Frame:ResizeToBoundsRect` is unpublished: its native semantics are undocumented and the simulator's `GetBoundsRect` returns only the frame's own rect.
- [ ] `PlayerChoiceToggle_TryShow` appears as both added (868) and removed (1027) in the source page; the cached 12.1.0 bootstrap publishes it, so the removed row stays non-ok.
- [ ] `CHAT_MSG_*` describes a family, not one registerable event; its row remains non-ok without an authoritative concrete expansion.
- [ ] The current register does not carry CVar page defaults (all CVar annotations are empty); default comparison can only apply when `page_default` (or `default`) metadata is supplied.

## Out of scope

- Function signatures, outputs, security/secret policies, changed behavioral contracts, event payloads, dispatch, CVar mutability, and native-client parity.
- Fully loading optional LoD panels or Glue screens beyond the cached Game startup set (their bootstrap files are part of startup). Their unpublished rows remain gaps; this does not prove they are missing in native Retail.
- Authenticating the cached UI as a particular native build, fixing runtime gaps, or automatically upgrading ledger status from a baseline pass. Known gaps remain gaps even when expected.
