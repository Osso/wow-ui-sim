# Retail 12.0.0 supplemental publication sweep

Publication/absence breadth evidence for the [raw wikitext register](../../data/patch-api/sources/12.0.0-wikitext-register.json), revision 6747189. This supplements, not replaces, the older occurrence audit. [Investigation](../wiki/investigations/patch-12-0-0-api-audit.md#wikitext-supplement) owns observations and review classifications.

## What it must do

- [x] Retain raw revision provenance and reproducibly parse all 1,010 consolidated rows, including six uncollapsed script-object kinds; every printed inventory count must match.
- [x] Preserve byte-identical 12.0.5, 12.0.7 and 12.1.0 registers. The legacy 12.1.0 register omits optional metadata; regenerate it with `--inventory-only`.
- [x] Probe all rows in one fully loaded cached Game environment; require the non-OK ID set to equal the reviewed known-gap set exactly.
- [x] Apply later 12.0.5, 12.0.7 and 12.1.0 add/remove supersession, in order. Changed rows do not alter publication expectations.
- [x] Support `P1200_SWEEP_REGISTER` for a full scratch register and `P1200_SWEEP_OUT` for every observation, including failed runs. One flipped unsuperseded row must produce exactly one new gap.

## How it works

- [Shared sweep contract](patch-12-0-7-publication-sweep.md): classifier, cached Game preload, later-register supersession, deprecation fallback attribution.
- Script-object kinds are probed through their Lua constructors; abstract curve base uses the scalar curve, Region methods use a Texture. Constructor publication does not prove implementation semantics.

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: raw table parser, inventory-only output option for legacy serialization.
- `tools/test_gen_patch_wikitext_register.py`: uncollapsed script-object table fixture.
- `tests/common/publication_sweep.rs`: shared classification and exact gap comparison.
- `tests/patch_12_0_0_publication_sweep.rs`: source inputs and three later registers.
- `tests/data/patch_12_0_0_sweep_known_gaps.json`: 111 retained reviewed non-OK source IDs after housing/initiative publication closure; historical counts below retain their original proof scope.
- `tests/data/patch_12_0_0_sweep_known_gaps.json`: 93 retained reviewed non-OK source IDs after transmog, C_Secrets and this 35-row plain-global closure.
- `src/c_api/patch_retired_members.rs`: 64 exact removed namespace keys gated from `retail-12-0-0`; later removals retain later epoch gates.

## Tests asserting this spec

- Python inventory fixture and byte comparisons against all three committed later registers.
- `patch_12_0_0_publication_sweep`, separately filtered, plus one-row negative control.

## Known gaps (current cycle)

Original baseline: 812 OK / 198 reviewed non-OK. The transmog implementation retires exactly 50 IDs; [historical proof](#transmog-verification--2026-10-05) observes 862 OK / 148 reviewed non-OK. C_Secrets then retired 20 IDs: base `a10d55822` has 128 gaps. This plain-global closure retires exactly 35 more: [final proof](#plain-global-verification--2026-10-05) observes 917 OK / 93 reviewed non-OK. [Evidence and row-by-row review](../../data/patch-api/evidence/12.0.0-session-2026-10-05/) record observations; older page-coverage ledger stays unchanged.

- [ ] Remaining raw-absent autostub-only namespaces/members need publication/backing semantics; two assigned plain globals remain blocked by pinned rilua.
- [x] Nineteen removed globals reviewed: four absent after epoch-gating simulator publishers; 15 legitimate cached Blizzard aliases attributed by exact identity. Cached CombatLog and DeathRecap consumers pass.
- [ ] Five callback/no-script/internal event rows lack a supported probe path; registration/delivery eligibility remains unproven.
- [ ] Four event removals are also listed as additions by the page. Preserve both rows as metadata conflicts, not runtime removals.
- [ ] Two added CVars lack getter/default publication; one removed CVar is a case-only scope rename resolved by case-insensitive getters. No default or scope semantics were invented.

## Plain-global closure contract

Assigned 37 global IDs are investigated only in `p1200-globals`. Simulator legacy combat-log aliases/navigation, `GetBattlegroundInfo` and `SetPortraitToTexture` stop publication at `retail-12-0-0`; earlier profiles retain their surface. Cached Blizzard deprecation aliases remain legitimate client publications, including direct references whose native/Lua destination source lacks `Deprecated`. The sweep parses simple `C_*` assignments from the loaded cached `Deprecated_CombatLog.lua`, checks exact function identity and records both source and destination. Unrelated replacements remain gaps. This closes 15 alias rows without modifying Blizzard Lua; four other removed globals are absent. Sixteen added globals have real registrations; `dropsecretaccess` and `issecrettable` retain their fixture IDs. Current fixture: 93 reviewed gaps after removing exactly 35 assigned IDs from the 128-gap base.

Player cost and raid-marker enablement read explicit host inputs. Cloak/helm visibility is independent and reversible. No existing simulator clothing state exists at base `a10d55822` (the earlier `cloak_helm_transition.lua` is a native-probe fixture, not simulator state); these globals share one new state. Defaults are inferred, not native evidence; no persistence or 3D effects are claimed. Clothing setters authenticate VM-secret arguments before applying Lua truthiness (inferred conversion), so tainted callers cannot turn an opaque secret false into true.

`canaccesssecrets` uses the same VM guard as secret unwrapping; `hasanysecretvalues` checks direct VM wrappers, not nested contents. Pinned rilua `a76ffa8` rejects `SecretWrapContents`; a table containing a secret is not itself a secret table. `issecrettable` remains blocked: `Userdata::secret_value` is VM-private (`E0624`), and guarded `unwrap_secret` cannot inspect wrapper kind for tainted callers. `dropsecretaccess` remains blocked: no separate caller-context revocation primitive exists. Stack-taint mutation would also change `issecure` and is not a faithful implementation. Neither fake global is registered.

Unit role flags are explicit known-unit GUID sets; missing units cannot acquire roles through dangling input records. Spell-target class uses the actual player cast recipient GUID plus explicit class metadata, not the selected target; target-name display uses an explicit recipient on that cast. Channels/other casters remain outside the existing cast model. Empowered stage objects span successive stage boundaries and include the final hold interval. Threat lead reads explicit unit/mob GUID pair snapshots; nonleaders return red, unknown pairs return nil, restricted results stay VM-secret. Thresholds and display policy remain inferred.

`SetCursorPosition` updates simulator mouse input coordinates; it does not warp the OS pointer. Untainted calls are allowed, addon calls consume an explicit one-use gamepad grant; no hardware source is fabricated. Coordinates use the existing unscaled screen mapping (inferred). Rejected calls do not mutate input.

## Plain-global verification — 2026-10-05

Code revision: `a631b3e7b758560dd0d73b3426e65c39b341f03c`. [Per-ID outcomes](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-globals-outcomes.json) retain all 37 original IDs, publisher paths and final observations. [Proof ledger](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-globals-proof.json) retains exact commands, scopes, revisions and results. Later documentation/data-only commits do not invalidate code proof.

All helper invocations used `python3 /home/osso/.worktrees/wow-ui-sim-p1200-globals/scripts/build-host.py --build-host local`; default debug retail, existing target directory only. No housing/vendor/Wowless/page-coverage edits, pushes, merges or delegation.

| Integration filter | Before at master base `a10d55822` | After |
|---|---|---|
| combat_log | 11 passed | 10 passed; obsolete native-global test retained only for earlier epochs |
| death_recap | 9 passed | 9 passed |
| secret | 271 passed | 276 passed |
| unit | 609 passed / 1 failed | 614 passed / same 1 failed |
| cloak | 0 matched | 2 passed |
| threat | 4 passed | 6 passed |

The unchanged failure is `edit_mode_api::enums::unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size`: both runs observe 22 metadata entries versus expected 21. It was reproduced before implementation on the starting master revision, then again by the exact focused test at current master `d9776867159b7acd5a3847f8ea11095218e1bb15` (0 passed / 1 failed, same 22 versus 21). That comparison used detached HEAD inside this same worktree, then restored `p1200-globals`; no canonical/other-worktree commands or edits. No adjacent enum fix was made.

Additional proof: `p1200_` 13 passed; alias-attribution test 1 passed (15 exact cached aliases plus unrelated-replacement negative control); `pvp_info` 8 passed; `methods_texture` 53 passed; lib `combat_log` 3 passed. Cached full-UI prefork consumers: combat_log 31 passed, death_recap 8 passed. These exercise actual cached Blizzard loads, not replacement vendor Lua.

| Isolated sweep (`--test-threads=1`) | Rows | OK | Reviewed non-OK | Result |
|---|---:|---:|---:|---|
| patch_12_0_0_publication_sweep | 1,010 | 917 | 93 | Pass |
| patch_12_0_5_publication_sweep | 363 | 351 | 12 | Pass |
| patch_12_0_7_publication_sweep | 174 | 171 | 3 | Pass |
| patch_12_1_0_publication_sweep | 778 | 768 | 10 | Pass |

12.0.0 output: `/home/osso/.cache/wow-ui-sim-audit/p1200-globals-sweep.json`. Exactly 35 assigned IDs were removed from the fixture; `dropsecretaccess-470` and `issecrettable-472` remain reviewed gaps, with VM evidence above. Earlier-profile preservation is source-gate reviewed, not runtime-tested (retail-only build constraint).

`cargo fmt --check`, direct `rustfmt --check` on changed test modules and local `--check` pass. Manual changed-Rust readability audit split stage-vector emission and threat-category calculation; no warning suppression was added. Six pre-existing iced vendor manifest-key deprecations remain, also present in baseline builds. Separate local build followed by `timeout 90` around `--no-build --run -- --no-addons --no-saved-vars lua-errors` exits 0 with `[]`, zero unique/total errors.

## Transmog outfit closure contract

The assigned transmog follow-up adds behavioral proof beyond publication: catalog creation and rename, displayed selection/locks, per-outfit saved slot contents, pending appearance and situation overlays, synchronous documented events, atomic apply with collection eligibility and host pricing. Invalid applications must preserve pending state and money. Native pricing/caps, persistence and automatic situation-trigger evaluation remain unverified; local choices are marked `INFERRED` in code. Publication fixtures are retired only after the sweep observes real registrations.

Synchronous viewed weapon-option events must fire only when the stored option changes; same-option refreshes inside event handlers must not recursively redispatch. Modern retail does not execute the old slot-topology defaults; only the existing temporary sheathe gap remains there, with a retirement note.

Slot topology must use `Enum.TransmogOutfitSlot`, not zero-based inventory slots. Both shoulder locations are cacheable; displayed groups include the secondary shoulder only when the outfit split is enabled. Armor/weapon option collection queries use enum categories; category and illusion queries read collection state. Custom-set name validation and creation share one policy and enforce a host-configurable capacity. Set-filter reset restores checked collected/uncollected/PvE/PvP defaults (inferred).

Set catalogs and slot-source membership are explicit simulator inputs; imports stage collected set alternatives or inventory-indexed custom-set appearance triples into the pending model. Available-set queries obey collection and PvE/PvP filters. Pending price queries derive from host slot pricing when no explicit cost snapshot is provided; failed applies preserve money and pending contents. Outfit pickup sets cursor payload (including equipped-gear ID zero). Atlas assignment and reversible custom-set hyperlink encoding are inferred; native hyperlink interoperability is not claimed. Outfit cursor payloads can be placed on and picked up from existing outfit action slots; zero marks the equipped-gear action. Compact world collection sources and rich host metadata share a source lookup so existing collected appearances remain usable.

`C_Transmog.GetSlotVisualInfo` consumes the real `TransmogLocationMixin:GetData()` payload (`slotID/type/modification`), mapping inventory IDs and secondary shoulders to outfit slots. `HasAvailableSets` reads eligible catalog membership independently of UI filters; obsolete source-substring placement assertions are replaced by eligibility/visibility behavior tests.

`C_Transmog.GetSlotVisualInfo` returns the documented single structure with equipped, pending and applied source/visual identities from equipment and outfit state. `C_Item.CanItemTransmogAppearance` returns boolean plus `TransmogOutfitSlotError`, with inferred inventory-type/quality eligibility; binding, race, class and form restrictions remain unverified. Retired APIs stay absent; `patch_retired_members.rs` is not changed.

Situation UI categories/groups/options are explicit host metadata; option values must reflect pending and saved outfit selections. Equipped weapon-option queries inspect actual equipped item inventory types, not viewed-option preferences. No automatic native situation catalog or artifact option is fabricated.

Tests: `tests/transmog_outfit_visuals.rs`, `tests/transmog_outfit_situation_catalog.rs`, `tests/transmog_outfit_lifecycle.rs`, `tests/transmog_outfit_slots.rs`, `tests/transmog_outfit_transactions.rs`, `tests/transmog_outfit_catalog_inputs.rs`. Implementation: `src/c_api/c_transmog_outfit_info/`, `c_transmog_collection.rs`, `c_transmog_sets.rs`.

## Transmog verification — 2026-10-05

Proof scope: code revision `85f738227510e7c95cd1ae524932f8d4b6c10395`; comparison master `3b4f660c4b83a25ed0d7dd2c84f37ce07cb17c74`, checked detached inside the same worktree, then restored to `p1200-transmog`. Later documentation and formatting-only changes do not invalidate these results. All builds used local debug retail and the existing target directory. No vendor, secrets or page-coverage files changed.

Command prefix: `python3 /home/osso/.worktrees/wow-ui-sim-p1200-transmog/scripts/build-host.py --build-host local`.

| Command suffix | Result at code revision |
|---|---|
| `--test --test integration transmog` | 139 passed, 0 failed; includes visual inventory/secondary/illusion selectors and catalog eligibility/filter behavior |
| `--test --test integration wardrobe` | 2 passed, 0 failed |
| `--test --test integration collections` | 47 passed, 0 failed |
| `--test --test integration outfit` | 50 passed, 0 failed; overlaps transmog filter |
| `--test --test prefork_full_ui -- transmog` | 8 passed, 0 failed; cached full UI |
| `--test --test prefork_full_ui -- wardrobe` | 7 passed, 0 failed; master: 3 passed, 4 failed with `itemModifiedAppearanceID requires a number` |
| `--test --test integration patch_12_0_0_publication_sweep -- --test-threads=1 --nocapture` | Isolated pass; 1,010 rows, 862 OK / 148 exact known non-OK |
| `--test --test integration patch_12_0_5_publication_sweep -- --test-threads=1 --nocapture` | Isolated pass; 363 rows, 351 OK / 12 exact known non-OK |
| `--test --test integration patch_12_0_7_publication_sweep -- --test-threads=1 --nocapture` | Isolated pass; 174 rows, 171 OK / 3 exact known non-OK |
| `--test --test integration patch_12_1_0_publication_sweep -- --test-threads=1 --nocapture` | Isolated pass; 778 rows, 768 OK / 10 exact known non-OK |
| `--test --lib` | Full suite once: 1,969 passed, 7 failed; all seven names fail in focused master comparisons |
| `--check` | Exit 0 |
| bare prefix, then `--no-build --run -- --no-addons --no-saved-vars lua-errors` under `timeout 90` | Build exit 0; startup exit 0, `[]`, zero unique/total Lua errors |

`cargo fmt --manifest-path /home/osso/.worktrees/wow-ui-sim-p1200-transmog/Cargo.toml` and subsequent `--check` exit 0. Because `autotests = false` hides generated integration modules from Cargo formatting discovery, direct `rustfmt --edition 2024` and subsequent `--check` also cover `tests/c_api_surface.rs` and `tests/transmog_outfit_visuals.rs`. Six existing deprecated Clippy manifest-key warnings in `iced-wgpu-patched/Cargo.toml` also appear on master; no suppression or vendor edits.

Master failure comparisons use prefix plus `--test --lib FILTER`, with filters `transmog_situation`, `debug_environment_defaults`, `housing_catalog_state`, `apply_system_anchors`, and `unit_cast_duration_clears_before_completion_callbacks`. Exact failures: transmog situation enum (`metadata.NumValues=32`), debug defaults (calling userdata), housing searcher (`bad_searcher`), three anchor tests (missing `InitSystemAnchors`), and cast-duration clearing (assertion failure). The duration test's source differs on current master, but the same named test fails there too. No transmog regression is attributed to these failures.

All 50 assigned IDs are removed from the known-gap fixture, with no other fixture removal. Their closure is **closed-modeled**: registered C API implementations backed by simulator state or explicit slot/codec policies, not temporary defaults. This is bounded simulator behavior plus publication proof, not native pricing, eligibility, codec or persistence parity. Other unassigned set-relation and sheathe defaults remain temporary workarounds.

## Out of scope

Behavior, signatures, output shapes, secrecy/security, callback-event delivery, defaults/mutability, native-client parity, strict historical epoch builds, Enums/Structures sections, and club model changes. CVar default mismatches are diagnostic only. Contradictory added/removed rows remain separate observations rather than being silently deduplicated.

## Housing/initiative closure contract

The 17 assigned housing IDs retire from the publication fixture: 13 initiative methods and two housing methods are **closed-modeled** with explicit host state; the two fixture-debug methods are **closed-temporary-publication** only. Their signatures/payloads are absent from cached exterior docs and consumers, so unavailable debug data returns one nil by an explicitly inferred temporary policy. Retirement requires native diagnostic evidence and a GUID/selected-fixture model; no native diagnostic parity is claimed. Page-coverage ledgers remain untouched.

Initiative getters expose active/viewing GUID identity, documented info/tasks/milestones/progress and activity entries. Required level compares to player level; entitlement and current party's qualifying neighborhood membership are separate inputs. Task links come from host data, not an invented native codec. Request calls defer replies to a timer tick because cached dashboard OnShow requests before registering listeners, then fire `NEIGHBORHOOD_INITIATIVE_UPDATED` or `INITIATIVE_ACTIVITY_LOG_UPDATED` with zero payload and getter-visible loaded state. Reply identity stays bound to the requested neighborhood; a stale-view reply loads its own cache without notifying the new view. Active-selection changes dispatch a refresh, identical writes do not.

INFERRED policies: empty string denotes unset active GUID; default required level is 1 and entitlement/group/shop flags are unconfigured false; missing requested initiative records use choosing-stage ID zero; task links are active-neighborhood host strings, unknown links empty; stale-view reply notifications are suppressed; plot clicks record selection without purchase/teleport/event. No service acquisition, native progression, persistence or native default policy is claimed.

Tests: `tests/housing.rs`, covering concrete neighborhood/task/milestone/activity data, callback ordering, same-write reentry, detached results, view-switch reply identity, host level/access/group/shop gates and environment isolation. [Runtime model](../wiki/systems/neighborhood-initiatives.md).

## Housing/initiative verification — 2026-10-05

Code revision `daa2476ac`; base/master comparison `a10d55822`, tested before production edits in this same worktree. All commands use local debug Retail and this worktree's existing target directory. No vendor, Wowless, other worktree, plain-global or page-coverage changes. No agents/models, push or merge.

Command prefix: `python3 /home/osso/.worktrees/wow-ui-sim-p1200-housing/scripts/build-host.py --build-host local`.

| Command suffix | Base/master | Final code |
|---|---|---|
| `--test --test integration housing -- --nocapture` | 281 passed | 287 passed, zero failed |
| `--test --test integration neighborhood -- --nocapture` | 14 passed | 18 passed, zero failed |
| `--test --test integration initiative -- --nocapture` | 3 passed | 7 passed, zero failed |
| `--test --test prefork_full_ui -- housing` | 201 passed / 2 failed | 201 passed / same 2 failed |
| `--test --test integration patch_12_0_0_publication_sweep -- --test-threads=1 --nocapture` | fixture: 128 known gaps | isolated pass: 1,010 rows, 899 OK / 111 exact known non-OK |
| `--test --test integration patch_12_0_5_publication_sweep -- --test-threads=1 --nocapture` | not rerun at base | isolated pass: 363 rows, 351 OK / 12 exact known non-OK |
| `--test --test integration patch_12_0_7_publication_sweep -- --test-threads=1 --nocapture` | not rerun at base | isolated pass: 174 rows, 171 OK / 3 exact known non-OK |
| `--test --test integration patch_12_1_0_publication_sweep -- --test-threads=1 --nocapture` | not rerun at base | isolated pass: 778 rows, 768 OK / 10 exact known non-OK |
| `--check` | focused tests compiled base | exit 0 |
| bare prefix, then `timeout 90` + prefix + `--no-build --run -- --no-addons --no-saved-vars lua-errors` | not rerun at base | build/run exit 0, `[]`, zero unique/total errors |

The unchanged prefork failures are `blizzard_deprecated_housing_catalog_loads::blizzard_deprecated_housing_catalog_wraps_category_info_with_legacy_field` and `blizzard_deprecated_housing_catalog_loads::blizzard_deprecated_housing_catalog_wraps_get_catalog_entry_info_with_legacy_fields`. Their deprecated category/entry field assertions failed at base before implementation.

`cargo fmt --manifest-path /home/osso/.worktrees/wow-ui-sim-p1200-housing/Cargo.toml -- --check` and `rustfmt --edition 2024 --check /home/osso/.worktrees/wow-ui-sim-p1200-housing/tests/housing.rs` exit 0. Manual changed-Rust review covers model wiring, field names, borrow lifetimes across callbacks, timer neighborhood identity, table rooting and explicit diagnostic limitations. Six pre-existing deprecated Clippy manifest-key warnings from `iced-wgpu-patched/Cargo.toml` appear in base and final commands; no new compiler warnings or suppressions.

All 17 assigned observations are `ok: true`, `raw=function; lookup=function` in `/home/osso/.cache/wow-ui-sim-audit/p1200-housing-sweep.json`. Exactly those 17 IDs, no others, were removed from the fixture. Logs reside in this worktree's `target/p1200-final-*.log`; documentation-only proof recording does not invalidate code-revision results.

## Mixed namespace closure contract — 2026-10-05

`C_StringUtil.FloorToNearestString` and `RoundToNearestString` format finite numeric inputs as integer strings. INFERRED: rounding ties toward positive infinity; nonnumeric/nonfinite inputs error; negative zero formats as `0`. Cached StringUtilDocumentation supplies the one-number/one-string contract, not native tie/coercion/secrecy parity. Concrete signed fractional cases exercise both producers in `tests/p1200_mixed.rs`.

Five published events now pass finite Retail event validation: `CHAT_MSG_ENCOUNTER_EVENT`, `COMBAT_LOG_APPLY_FILTER_SETTINGS`, `COMBAT_LOG_EVENT_INTERNAL_UNFILTERED`, `COMBAT_LOG_REFILTER_ENTRIES`, `TOOLTIP_SHOW_ITEM_COMPARISON`. Existing restricted/callback policy is unchanged; closure proves registration and unregistration, not native dispatch. `minimapTrackedInfov2` and `useCompactPartyFrames` use the existing mutable CVar store with INFERRED default `0`.

Removed event rows remain gaps: the same source page also adds all four names; current cached ActionBarController/Game EventRouting, ActionButton/UnitDocumentation and HousingBlueprint/HouseEditor consume three. `TRANSMOG_OUTFITS_CHANGED` has the same add/remove contradiction; no historical epoch is invented. `nameplateShowFriendlyNPCs` remains a gap because this same patch publishes `nameplateShowFriendlyNpcs`; case-insensitive CVar identity makes removal incompatible with that addition (and existing metadata tests). No later-patch re-add found in committed wikitext sources.

`C_ActionBar.UnregisterActionUIButton` removes only the supplied frame from existing action-button registration state, including repeated unregistration. `C_Spell.GetSpellLossOfControlCooldownDuration` and its spellbook counterpart return a new duration proxy using the existing LoC record's start/duration/modRate. Missing/inactive records return zero values (documented MayReturnNothing). INFERRED: `is_active`, not wall-clock expiry, selects presence; player-bank/off-spec identity follows existing cooldown selection; no additional secrecy/native expiry policy is claimed.
