# Method surface baselines

Method-surface comparisons must select the baseline for the current compiled configuration, rather than compare every profile against Retail expectations. The comparison and refresh helpers live in [`tests/method_diff_coverage.rs`](../../tests/method_diff_coverage.rs). See the [comparison documentation](../wow-client-diff/README.md#compiled-configuration-method-baselines) for baseline locations and proof status.

## What it must do

- [x] Select exactly one baseline using `ACTIVE.cache_subdir`, the highest actually enabled `retail-*` epoch cfg, and the `forbidden-aspects` and `on-update-modes` flags.
- [x] Use `no-retail-epoch` when no Retail epoch cfg is enabled; do not infer an epoch from metadata or substitute `1207`.
- [x] Preserve root `docs/wow-client-diff/diff_methods_{extra,missing}.txt` for `retail` / `retail-12-1-0` with both flags enabled. Use `docs/wow-client-diff/method-baselines/<profile>/<epoch>/forbidden-<N>-on-update-<N>/` otherwise, with each flag encoded as `0` or `1`.
- [x] Keep the two existing exact assertions and method discovery unchanged; ordinary execution compares only the current compiled configuration, not a loop over all profiles or epochs. The default Retail build therefore uses one Retail baseline.
- [x] Make the ignored refresh helper create the selected directory and write only that configuration's two files, including when no baseline exists. Ordinary comparison assertions must fail explicitly when the selected file cannot be read, without fallback or guessed fixtures.

## How it works

- [Method comparison investigation](../wiki/investigations/method-dispatch-refactor.md#cross-profile-comparisons--2026-10-11-utc)
- [Baseline selection and proof status](../wow-client-diff/README.md#compiled-configuration-method-baselines)

## Implementation inventory

- `tests/method_diff_coverage.rs` — existing exact method comparisons, discovery, and ignored refresh helper; configuration selection is source-audited and executed for default Retail and explicitly selected Wrath.
- `src/client_profile.rs` — active compiled profile identity.
- `docs/wow-client-diff/diff_methods_extra.txt`, `diff_methods_missing.txt` — retained default-configuration comparison fixtures.
- `docs/wow-client-diff/method-baselines/` — non-default configuration fixture location required by this contract.

## Tests asserting this spec

- `tests/method_diff_coverage.rs` — both exact assertions pass for default Retail and explicitly selected Wrath (four cases). Wrath controls detect one deleted row and one absent fixture; ignored refresh passes with all protected bytes unchanged. Selected-only writes are additionally source-inspected. See [retained receipts](https://github.com/Osso/wow-ui-sim/tree/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-baseline-selection).

## Path receiver reconciliation

Current `6b522c595` default-Retail comparison executes missing PASS and extra FAIL solely on sixteen stale `CreateControlPoint` entries (Button, CheckButton, ColorSelect, Cooldown, EditBox, FontString, Frame, GameTooltip, MessageFrame, Model, PlayerModel, ScrollFrame, SimpleHTML, Slider, StatusBar, Texture). Path creation now attaches that method only to Path tables, rather than the shared ordinary-frame index. Remove exactly those observed resolved entries from the selected root extra fixture; discovery, missing fixture, other configurations and native captures remain unchanged. This records simulator surface, not native parity. The baseline is read at runtime; corrected extra assertion remains pending without rebuilding unchanged source or rerunning unaffected tests.

## Known gaps (current cycle)

- [x] Verify selection, exact assertions, selected-only refresh, and explicit missing-baseline failure at `67598f430`: source audit, Retail/Wrath positives and Wrath controls independently accepted; formatting/check exit 0 with existing warnings.
- [ ] PTR/Mists/Era/Anniversary/Forever fixture copies are byte-verified but lack fresh assertions under the new selection code. Older Retail epochs still lack baseline data and runtime proof. No full matrix or native acceptance is claimed.

## Out of scope

- Running all baselines on every default test invocation: baseline selection follows the current build only.
- Changing discovery, native reference data, method implementations, model behavior, or receiver semantics: this change selects comparison expectations only.
- Establishing native parity or closing historical failures: historical suite FAIL and parent OPEN remain until separately proven resolved.
