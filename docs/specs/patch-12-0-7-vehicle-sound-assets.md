# Retail 12.0.7 vehicle aura, Mythic+ sound permission and known-asset inputs

B23/B24/B28 bounded host-input slice integrated on `p1207-r6`. The [retained 12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) states the vehicle aura marker and out-of-combat Mythic+ sound permission changes and adds the UI asset namespace. Later cached declarations supply signature context, not 12.0.7 proof. Separate Retail 12.1.5.70688 native evidence now verifies the asset predicate and path behavior below; applicability to earlier patches remains unknown. B27 IMPORTANT removal is excluded because cached Blizzard Lua republishes that entry.

| Source row | Query / change | Explicit input | Returns |
|---|---|---|---|
| `prose-undated-018` | AuraData `isFromPlayerOrPlayerPet` | existing aura marker and `player_controlled_vehicle_sources` | public boolean field |
| `prose-undated-019` | `C_UnitAuras.AddPrivateAuraAppliedSound` permission | existing M+, player combat, encounter, PvP and registration state | one public numeric handle, or error without allocation |
| `prose-undated-011` | namespace addition | existing GetFileID plus predicates below | partial namespace candidate only |
| `global api-C_UIFileAsset-IsKnownFile-050` | `C_UIFileAsset.IsKnownFile` | `known_shipped_asset_ids`, `known_loose_asset_paths` | one public boolean |
| `global api-C_UIFileAsset-IsLooseFile-051` | `C_UIFileAsset.IsLooseFile` | same inputs | one public boolean |

Cached `UIFileAssetAPIDocumentation.lua:29,45` declares `AllowedWhenUntainted`; :39 says loose-file existence/openability is not verified. Later `UnitAuraDocumentation.lua:11–14` declares `AddAuraSound` with `HasRestrictions` and `AllowedWhenUntainted`; `Deprecated_12_1_0.lua:22–23` forwards the legacy sound call. No exact generated vehicle-field declaration was located. Source/declaration/provider quotes and exact edits live in the external author handoff, not in claimed execution evidence.

## What it must do

### Vehicle marker — B23

- [ ] With `retail-12-0-7`, a stored aura whose explicit source token belongs to the host's player-controlled vehicle set publishes `isFromPlayerOrPlayerPet = true`. Set defaults empty. No wrapped constant earns coverage.
- [ ] Existing direct-player/pet classification is preserved independently. Unrelated or uncontrolled vehicle sources remain false unless the stored marker explicitly says true.
- [ ] Slot, index and instance-ID DTO queries reflect additions/removals live without rewriting stored AuraInfo. Environments do not share ownership sets.
- [ ] Classification remains a plain public boolean for secure and tainted callers. Existing slot-unit `NeverSecret` rejects secret units for either caller; this slice does not weaken it.
- [ ] INFERRED: source-token identity, exact token matching and ownership-at-query timing. No GUID/alias/past-owner derivation is claimed.

### Mythic+ permission — B24

- [ ] Insecure addon registration during active Mythic+ succeeds out of combat under `retail-12-0-7`, but remains denied in combat. Earlier epochs continue to deny active Mythic+ regardless of combat.
- [ ] Secure callers still register in every host context. Ordinary combat outside all three restriction contexts does not alone deny registration.
- [ ] INFERRED: independent encounter/PvP restrictions continue to deny insecure registration even alongside out-of-combat Mythic+. The 32-case secure/tainted × encounter × M+ × PvP × combat matrix preserves this distinction.
- [ ] Context changes are read live. Denials do not consume IDs or alter registrations. Successful calls retain actual payload values, allocator ownership and per-environment isolation.
- [ ] Sound argument/fields and every extra retain existing `AllowedWhenUntainted` authentication before type/context validation. New permission does not permit tainted secret consumption or change caller taint.

### Known-asset host catalog candidate — B28

- [x] Under `retail-12-0-7`, predicates read explicit per-environment catalogs; `GetFileID` results do not fabricate shipped membership. `GetFileID` remains unchanged.
- [x] VERIFIED (Retail 12.1.5.70688): `IsKnownFile` is true when either runtime table contains the resolved file ID. `IsLooseFile` is true when the first table's marker is nonzero, independently of `IsKnownFile`'s second-table lookup.
- [x] VERIFIED (Retail 12.1.5.70688): string paths are upper-cased and `/` becomes `\\`; an exact lookup miss retries with the final extension replaced by `.BLP`, or `.BLP` appended when no extension exists. A resolved file ID of zero returns false for both predicates.
- [x] VERIFIED (Retail 12.1.5.70688): these predicates do not check the disk. A known loose-file marker does not prove that the loose file exists or can be opened.
- [x] INFERRED simulator representation: `known_shipped_asset_ids` models the second table; the normalized `known_loose_asset_paths` catalog models first-table path entries and markers. Exact native backing structures remain UNKNOWN.
- [x] INFERRED: regular files under each loaded addon's root are inserted into the loose-path catalog before its Lua files run. Whether native runtime tables include addon loose files is UNKNOWN.
- [ ] Host changes and revocations are visible without recreating the environment. Another environment remains empty and unaffected.
- [ ] Both queries authenticate every original argument and every extra with `rilua::table_security::unwrap_secret` before validating any. Secure authentic secrets work; tainted authentic secrets fail, including extras after a malformed public first argument. Caller taint and input secrecy remain unchanged.
- [ ] INFERRED: reuse GetFileID's positive integral u32 boundary and mandatory number/string type; invalid numeric domains return false, invalid types error, authenticated extras are ignored. No numeric-string coercion.
- [ ] INFERRED: catalog acquisition and the exact path-to-file-ID table model under `retail-12-0-7`; no complete-catalog or historical population proof is claimed.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)
- [Existing asset ID contract](asset-id-12-0-7.md)
- [Existing sound registration/context contract](private-aura-sound-add-context.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — feature-gated empty-default catalogs and controlled vehicle source set.
- `src/lua_api/globals/auras.rs` — classify copied aura DTOs from live ownership inputs before publication/filtering.
- `src/c_api/private_aura_sounds/add.rs` — epoch-gated M+/combat permission predicate; existing authentication and registration preserved.
- `src/c_api/c_ui_file_asset.rs` — authenticated catalog predicates and loaded-addon path catalog writer; prior epoch implementation remains statically separate, never a fallback.
- `src/loader/addon.rs` — populate the inferred catalog before add-on loading begins.

## Tests asserting this spec

`tests/patch_12_0_7_b23_b28.rs`: default Retail build RED 1 passed / 12 behavioral failures; GREEN 13 passed. Three B23 cases, four B24 cases (one with 32 independent contexts; cached forwarding requires `retail-12-1-0`), six B28 cases. Sound tests also assert earlier-epoch expectations when built without `retail-12-0-7` but with `retail-12-0-5`.

Existing-test changes: restricted M+ denial fixture in `tests/private_aura_sound_add_context.rs` enters combat; two shipped-path fixtures seed explicit ID membership; legacy filesystem tests in `tests/ui_file_assets.rs` run only before 12.0.7, replaced for this epoch by explicit catalog behavior tests. `tests/retail_ellesmere_core_gamepad_consumer.rs` exercises loaded-addon file registration before its Lua consumer runs. This is a bounded host-catalog contract, not proof of native catalog acquisition.

## Known gaps (current cycle)

- [x] Targeted Retail compilation and the original consumer test pass. The isolated load also reached all 20 installed EllesmereUI add-ons; remaining Lua errors and native-parity limits are recorded in the investigation page.
- [ ] The pinned API documentation declares the C_UIFileAsset surface, but native asset-table identities and catalog acquisition remain unknown. The 12.1.5 byte evidence does not prove identical behavior in every earlier patch.
- [ ] Selected-root reconciliation and completeness of the loaded-addon loose-path catalog remain unproved; do not claim complete native B28 parity.

## Out of scope

- B27 `source-context-004`: cached `Blizzard_FrameXMLUtil/AuraUtil.lua:270–286` overwrites `AuraFilters` and publishes `Important = "IMPORTANT"`. No vendor patches, post-load cleanup, parser reinterpretation or knowingly failing full-load test. Requires epoch-pinned cache reconciliation first.
- Native vehicle ownership/GUID aliasing, source-token reuse and ownership history; only declared live source-token relationships are modeled.
- Sound playback, sound eligibility service, new allocator/removal policy and automatic context acquisition; existing modeled registration state is retained.
- Complete client asset catalog, discovery outside a loaded add-on root, root reconciliation, CASC availability and earlier-patch behavior. B28 is a bounded loaded-addon catalog model, not complete native parity.
- GetFileID reimplementation, source ledger/capability promotion, older-profile broad parity and GUI execution.
