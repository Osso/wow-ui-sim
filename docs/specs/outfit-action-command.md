# Outfit actions and conditional commands

Retail 12.0.5 outfit selection uses the existing [outfit catalog](outfit-catalog-lookups.md). Source prose rows `prose-2026-03-25-111`/`prose-2026-03-31-165` announce the secure action; `prose-2026-03-25-122`/`prose-2026-03-31-179` describe the conditional-command error fix in `data/patch-api/sources/12.0.5-api-changes.txt`. Cached retail `SecureTemplates.lua:653–671`, `SlashCommands.lua:1708–1731`, and `TransmogOutfitInfoDocumentation.lua:51–61` establish dispatch attributes, numeric player-facing indices, toggle/bang/empty-command behavior, and secret-argument policy. These are bounded simulator contracts, not native-client verification.

## What it must do

- [ ] Keep per-environment host-owned applied selection independent of catalog, viewed and pending metadata. `GetActiveOutfitID()` observes that selection, not writable Lua namespace fields. **INFERRED:** initial absence and numeric zero for absent selection.
- [ ] `ChangeViewedOutfit(outfitID)` resolves the catalog ID, not a player-facing index, and updates separate host-owned viewed selection. `GetCurrentlyViewedOutfitID()` reads that selection as one public numeric result; the mutation returns no values. Apply `AllowedWhenUntainted` validation before mutation; ordinary addon arguments do not cleanse taint. No selected-epoch Lua storage reader/writer or fallback remains. **INFERRED:** strict number input, initial zero, first matching catalog ID, unknown IDs preserve selection, and disabled flags do not define eligibility.
- [ ] Deliver `VIEWED_TRANSMOG_OUTFIT_CHANGED` synchronously after a valid viewed request, with no payload and already-updated selection. **INFERRED:** this event's association with the setter and refresh on repeated valid selection; the cache declares synchronous delivery/no payload but not the precise trigger policy. Invalid/missing IDs emit no event. Do not invent displayed-outfit or catalog-change notifications.
- [ ] **INFERRED:** viewed selection preserves active selection, pending cost, viewed-slot snapshots and pending sheathe metadata. Active change/clear continues to preserve viewed selection. No native pending-discard policy is claimed.
- [ ] `ChangeToOutfit(index, allowRemoveOutfit)` resolves the catalog's published player-facing index, not its ID or vector position. A repeated selection clears only when removal is allowed; a different selection applies. `ClearOutfit()` removes selection idempotently. Both mutation APIs return zero values.
- [ ] Reuse the unchanged vendor `SecureActionButton_OnClick` outfit dispatch: `outfit-index`, `action=change/toggle/clear`, and default toggle. No simulator reimplementation of vendor action selection.
- [ ] Apply existing VM `AllowedWhenUntainted` validation to both ChangeToOutfit arguments before mutation. Ordinary addon arguments remain permitted and must not cleanse caller taint. **INFERRED:** strict number/bool argument types; missing catalog index preserves selection; duplicate indices choose first catalog entry, matching lookup policy.
- [ ] Route `C_Macro.RunMacroText` `/outfit` through the existing condition selector and the same selection transition. False conditions do nothing without errors; true conditions execute; semicolon branches honor current host conditions. Plain indices toggle, `!index` does not toggle off, and an empty command clears. Unsupported/malformed inputs do not fabricate selections. **INFERRED:** Rust decimal-number syntax, not all Lua `tonumber` syntax.

## How it works

- [Lua/C API boundary](../lua-api.md)
- [Macro and frame dispatch](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/c_transmog_outfit_info/actions.rs`: active selection query/transitions and slash-command bridge.
- `src/c_api/c_transmog_outfit_info/viewed.rs`: host-backed viewed query, catalog-ID transition and synchronous refresh.
- `src/c_api/c_transmog_outfit_info.rs`: patch-gated registration/export; catalog and unrelated settings unchanged.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: separate optional active/viewed IDs and empty initialization.
- `src/lua_api/globals/security/{cmd_option.rs,mod.rs}`: share existing pure condition resolver without changing parser behavior.
- `src/lua_api/globals/spell_macro_verbs.rs`: `/outfit` route and only that command's empty-argument allowance.
- `src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs`: remove selected-epoch Lua selection defaults. Historical pre-12.0.5 code remains compile-time isolated, never available as a fallback in the modeled epoch. Existing historical combined lifecycle test is limited to its matching epoch.

## Tests asserting this spec

`tests/outfit_action_command.rs`, auto-included in the grouped integration target: host snapshot/environment isolation, sparse-index transitions, missing/invalid inputs, secret authentication, conditional branches, macro toggles/clear, and the complete unchanged cached vendor SecureTemplates file followed by its real click-handler dispatch. No test invokes a copied/extracted handler or fabricated secure action.

Round-100 follow-up tests: `tests/viewed_outfit_selection.rs` asserts host/public query state, environment isolation, catalog-ID versus index, synchronous event payload/order, repeat/miss behavior, pending/active preservation, and authenticated secret/tainted ordinary calls. `tests/transmog_outfit_info.rs` and `tests/pending_transmog_cost.rs` select viewed fixtures through `ChangeViewedOutfit`, not raw storage writes. Round-100 viewed selection: 0/3 RED with tests/state only, 3/3 GREEN after producers. Updated transmog fixtures passed 1/1 and pending-cost controls 10/10; previous B98 proof below does not cover this follow-up.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 7 FAIL. GREEN: 7/7 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-outfit-formatter.md) SHA256 `b4386140882065dcdb77c094ae3f83ef204f4f74d76aee06633e0f01d657b870`. Vendor handler is called directly, not through the simulator click pipeline; duplicate-index policy and ClearOutfit arity untested. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-111 bounded-coverage, prose-2026-03-25-122 bounded-coverage, prose-2026-03-31-165 bounded-coverage, prose-2026-03-31-179 bounded-coverage under capability `outfit-action-command`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [x] Round-100 targeted viewed RED/GREEN and fixture controls executed against ce418cbe4 plus the integrated follow-ups.
- [ ] Native exact behavior for disabled/unavailable outfits, duplicate indices, invalid types/numeric grammar, zero/absence encoding, and catalog drift. `isDisabled` is not interpreted as an eligibility rule; no native eligibility is claimed.
- [ ] Full loader/template/frame-click/secure-authority proof. The vendor test loads the complete file and calls the real handler with a real Button; it does not prove secure-template XML inheritance, physical input, protected-attribute permissions or all startup dependencies.
- [ ] Conditional-only empty branches and unsupported condition grammar remain limitations of the existing selector; this change does not extend it.

## Out of scope

Catalog population/create/delete/reorder, native wardrobe availability, 3D/transmog rendering, displayed-outfit transitions, unrelated events, persistence, pending edit/discard lifecycle, protected-frame authority and older-profile parity. Cached `TransmogOutfitInfoDocumentation.lua:62–70,175–182,835–839` declares the viewed setter/query and payload-free synchronous refresh event; `Blizzard_Transmog.lua:514–527` distinguishes active from viewed, and `Blizzard_TransmogTemplates.lua:140–159` selects by ID. Exact native miss/repeat/pending policies remain unproven and are marked INFERRED above. No complete-row/full-page acceptance is claimed before the bounded tests are executed and accepted.
