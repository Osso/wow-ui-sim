# Bounded 3.0.2 current-retail factory handoff

Worktree `/home/osso/.worktrees/wow-ui-sim-p302-factory`, branch `p302-factory`, base `efbe97f20fa49dd12c284f8e3505e16decd3d711`. Standalone `tests/patch_3_0_2_factory.rs`, whole-file client-retail guard and Cargo required-features. No runtime/model implementation or full-goal acceptance.

## Observed capability matrix

| Scope | Count/result | Proof boundary |
|---|---:|---|
| Literal occurrences | 373 | Exact original register retained; duplicates not deduplicated |
| Current classifier | 185 matches / 188 gaps | 69 published/accepted and 116 absent matches; not historical behavior |
| Current profile | Retail interface 120100 | Not historical retail 3.0.2 or Wrath Classic |
| Successors | 76 actual retail registers, oldest first | 3.0.3 → 3.0.8 → 3.1.0 → 3.2.0 → 3.3.0 → 3.3.3 → 3.3.5 → 4.0.1 → … → 12.1.0; no Classic supersession |
| Supplemental controls | Corrected 7/7 own target | Inventory, exact gaps, foreign-client rejection, raw FrameXML/handler, modified click, current event registration, CVar versus Command records |
| Fabricated global | Same 373 observations; 189 gaps | Only AutoLootMailItem's retained source ID changes; original register untouched |
| Historical/native/meaningful models | 0 / 0 / 0 | 431 raw rows, 367 signatures, 80 prose retain original source-only status; no rewrite credit |
| Historical seals | 36 original + 1 closure + 2 receipts unchanged | Before/after SHA256 including three seal manifests; not signed native attestations |

## Command proof ledger

All commands use explicit worktree cwd and own target `/tmp/wow-ui-sim-p302-factory-target` through Pyrun argv helpers, with Cargo offline/locked. No Bash, cwd switching or network. Exact argv, relevant inherited environment, overrides, stdout/stderr and SHA256 are in each `*-command.json`; [proof ledger](proof-ledger.json) records scope and invalidation. Code/expectation hashes at exact revisions are in [revision scope hashes](revision-scope-hashes.json).

| Receipt | Exact revision | Result |
|---|---|---|
| `discovery-command.json` | `982ccc3d99158084db17660463e0ecac21c967b5` | Empty known-gap set; expected RED exit 101 after writing all 373 observations, 188 gaps |
| `green-command.json` | `1fdb73aa66b95e738af0bde956b47113ed334128` | Retained first reviewed attempt: 6/7 RED; exact 188-gap sweep passed, console control expectation wrong |
| `negative-command.json` | `1fdb73aa66b95e738af0bde956b47113ed334128` | Fabricated unknown global: expected RED exit 101; same 373 rows, 188 → 189 gaps, only one observation differs |
| `green-corrected-command.json` | `57c9f73e3c3c48cb261031009eab2d4adf918280` | Corrected targeted GREEN exit 0, 7/7; all 373 observations equal original discovery |

No receipt overwritten. The console-only test correction does not invalidate the negative control or change the publication sweep. Current-retail `src/c_api/c_console.rs` appends 12 baseline plus two later Command records; Wrath's zero-record assumption was inappropriate. Corrected control observes its CVar as a CVar, not a Command, and 14 command records. No runtime fix made.

## Classifier limits

- FrameXML raw/lookup names both nil in bare environment; removed OpenToPage matches absence, added OpenToCategory is a gap. Loaded FrameXML rename behavior not measured. Only simulator intrinsic templates initialize; inherited secure-template construction and the unspecified SecureStateHeader replacement remain unproven prose, not reconstructed factories.
- Source GameTooltip heading backs `OnTooltipSetAchievement`; actual GameTooltip HasScript=false. Widget method lookup uses real constructors; shared inherited exposure (Button:GetFont remains a function) is not historical signatures, restrictions or semantics. Model/PlayerModel publication does not close intentional 3D gaps.
- FOCUSCAST and a fabricated unknown modifier both read NONE from temporary input defaults. No existence catalog discriminates them; retain unprobeable gap, not invented console command, alias or native absence. synchronizeSettings value/default are both absent; server synchronization is unmodeled by this measurement.
- Retail registration accepts all three literal events and rejects a fabricated event. This proves simulator catalog eligibility only, not arguments, causal delivery or native history. Console distinguishes Command/CVar record kinds; source has zero literal console commands, so catalog control adds no source contract credit.

Six inherited iced manifest deprecation warnings and six non-vendor no-default-feature build warnings remain in full stderr. No suppression or adjacent runtime edits. Runtime `src` Git tree identical to base (`ae453e3709a335ac3caecb272fde6772407e04d2`).

## Commits and retained evidence

- `d3903d884`: standalone discovery and test-only classifier support, separate spec/baseline seals.
- `982ccc3d9`: bounded plan before discovery.
- `1fdb73aa6`: 373 discovery observations, exact 188 gap reviews, source/code input snapshots and bounded controls/docs, committed before tests.
- `57c9f73e3`: console-test correction and preserved failed/negative receipts, committed before corrected GREEN.
- Subsequent documentation/evidence commit seals corrected proof without modifying tested code.

`inputs/` and `inputs-manifest.json` preserve 77 literal input registers plus Cargo manifest/lock and original reviewed test/classifier snapshots; `final-code/` preserves the console-corrected test. `historical-seals-before.json`/`historical-seals-after.json` are identical. `result-summary.json` and `gap-review.json` retain exact counts and all observed gap identities/limits. `seals.json` covers this separate evidence directory; prior evidence seals were never modified.

No canonical/vendor/cache/Wowless writes, rebase/push/merge/deploy/delegation/model CLI. Only targeted RED/GREEN and changed-file Rust formatter run. No check/lint/readability/coverage/broad/startup/final acceptance gates. Main owns integration and any literally backed state proposal before meaningful-model work; historical/native/model claims remain zero.
