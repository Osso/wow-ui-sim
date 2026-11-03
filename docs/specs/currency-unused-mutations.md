# Retail unused currency mutation (B5)

Retail unused flags for the original TokenUI checkbox and popup. Producer: `src/c_api/item_spell/c_currency.rs`; [C API architecture](../wiki/systems/lua-api.md) owns subsystem context. This bounded flag contract does not adopt native unused-group ordering.

## What it must do

- [x] `C_CurrencyInfo.SetCurrencyUnused(index, unused)` targets the current displayed list index, not currency ID, and synchronously updates the existing currency's unused flag. Generated Retail `CurrencyInfoDocumentation.lua:566-574` declares `index: luaIndex`, `unused: bool`.
- [x] Original `InactiveCurrencyCheckboxMixin:OnClick` sets/clears status, rescans the selected name and refreshes the original TokenFrame provider. List `isTypeUnused` and ID-based `CurrencyInfo.isTypeUnused` agree.
- [x] List `discovered` and `isTypeUnused` project supplied false/true values. Original `TokenFrameMixin:UpdatePopup` exposes the discovered currency checkbox and displays its checked unused state.
- [x] Invalid/header/currency-ID-as-index and missing supplied-row targets are no-ops. Repeated requests are idempotent; fresh environments do not inherit unused state.
- [x] Preserve active catalog order, collapse behavior, quantities and watched backpack membership/flags. No grouping or automatic watch mutation. Preservation/no-op policy is parent-directed simulator scope, not current-Retail native parity.

## How it works

- [C API and Lua boundary](../lua-api.md)
- [Existing query models](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/item_spell/c_currency.rs` — Retail mutator and list status projection over the existing supplied CurrencyInfo row; no second membership store.
- `src/c_api/currency_list.rs` — existing displayed-index/expansion model, unchanged by this slice.

## Tests asserting this spec

- `tests/currency_unused_mutations.rs` — original checkbox/current-index/provider transitions, real TokenEntryTemplate -> original popup visibility/checked state, supplied boolean controls, invalid/missing targets, idempotence and environment isolation. No copied/replaced consumers.
- `tests/currency_displayed_indices.rs` — original header expansion/index/provider preservation.
- `tests/currency_watched_mutations.rs` — existing displayed-index and ID-based watched behavior.
- `tests/currency_info_probes.rs` — supplied display/missing-row preservation; Retail list field count13 includes the two boolean fields and documented `currencyID`.

### Bounded proof

Tests-only Rust scope `bd45dfd02` reached original-consumer RED: checkbox failed `B5 unused: original checkbox must mark selected currency unused`; real-entry original popup failed discovered-checkbox visibility. Repeated mutation also failed; no-op controls passed. No registration/loader/setup exception counted as RED. The same worktree-bound ELF passed displayed2/watched1/info16 BEFORE controls with zero compiler warnings.

Producer `d27e79dd9` passed unused5/5 GREEN and displayed2/watched1/info16 AFTER controls (19/19, matching BEFORE). Both integration builds and the final headless Retail check emitted zero warnings; `cargo fmt --all --check` passed. Stock `wow-sim --no-addons --no-saved-vars lua-errors` returned `[]`, exit0, using a fresh child-only XDG_DATA_HOME containing the exact current persisted CVar file. Original CVar inode/size/mtime/ctime/hash remained unchanged. No rendering, full-suite or other-profile acceptance is claimed.

Commands: `cargo test --no-default-features --features client-retail --test integration --no-run -j1 --message-format=json`; each named module filter ran via `timeout 90 <ELF> <module>:: --nocapture --test-threads=1`. Check: `cargo check --no-default-features --features client-retail -j1 --message-format=json`. Builds/check ran only in admitted async agents.slice units with MemoryMax9G, MemorySwapMax0 and slot-1; each build touched only this worktree's test-harness build script. Every filter executed >0 tests; both ELF source-root checks identified only this worktree's tests paths.

Private handoff `b5-unused` records exact commands, source hashes, ELF hashes and the proof ledger. Supplemental supplied-state/checked-state controls `295fbd07a` precede the producer; they have no standalone RED receipt. Docs-only follow-ups do not alter the tested Rust source. Compiler: Arch rustc1.99.0 b940084d7 / cargo1.99.0 5f94df478.

## Known gaps (current cycle)

None within the bounded flag slice. Native grouping remains held below.

## Out of scope

Current Retail 12.0.x native grouping is UNKNOWN: laptop reported no current source available (2026-10-20). Historical 3.3.5a observations are not a Retail contract. INFERRED/held: header name/position, unused localized ordering, empty source categories, unused-header collapse/retained expansion and persistence. No grouping producer until a separate decision adopts a model. The staged `30-unused.patch` is a hypothesis, not an oracle; no native source read in this worktree.

Cached Retail `Blizzard_TokenUI.lua:505-542` consumes the two booleans, calls the setter, rescans by name and refreshes the provider. It does not establish native grouping. Duplicate names can select the first visible match; this work does not replace that consumer with an ID-based scan.

No vendor edits, shims, rilua edits, other-profile changes, new catalog, transfer/filter changes or event policy. Existing [displayed-index](currency-displayed-indices.md), [field-projection](currency-list-field-projection.md) and [watched](currency-watched-mutations.md) contracts otherwise remain unchanged.
