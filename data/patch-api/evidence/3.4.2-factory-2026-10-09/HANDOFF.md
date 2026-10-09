# Bounded 3.4.2 factory measurement handoff

Branch `p342-source`; worktree `/home/osso/.worktrees/wow-ui-sim-p342-source`; base `990dae19c30b7b71c0ae3e1a4dfcd0a444546d84`. Current commit is the commit containing this handoff; exact post-commit SHA is reported to parent. Historical source inventory, ledger and source-audit evidence/23 seals unchanged. Current evidence is separate.

## Results and boundaries

- Exact 155 source occurrences: 72 publication-direction matches, 83 strict mismatches. Globals 34/60; widgets 13/5; events 4/4; CVars 21/14 (match/mismatch). Zero probe errors/unprobeable observations. All 18 literal widget owners construct; this is not native widget identity or behavior.
- Added CVars: 14 absent registrations, 11 published defaults equal literal source, one published mismatch (`TargetAutoLock`: source `1`, value/default `0`). 15 recorded default differences include 14 absent registrations. Nine removed CVars observe value/default nil. All source `default/desc/scope/cat` fields retained verbatim in current measurement ledger.
- Globals preserve both raw and ordinary lookup: 27 added raw-nil/lookup-function, 11 added both-nil, five added raw-parent-absent/lookup-function; 16 removed raw-nil/lookup-function and `GetAddOnMetadata` raw/lookup-function. “83 gaps” does NOT mean 83 absent/native APIs.
- Events accept every nonempty name, including nonsense control and all four source removals. Added-name acceptance has weak discrimination, no native event existence/emission/payload/retirement credit. Factory method lookup/global publication proves no argument, result, security, state-transition or linked contract parity.
- Actual simulator: `ClientProfile::Wrath`, configured interface 38001, 3.3.5-era architecture. Source TOC 30402/native Wrath Classic 3.4.2 is NOT the measured client. `WowLuaEnv::new` only: no full Game, Blizzard publisher files or native client. Foreign retail/Mists successors ignored; actual 3.4.3 has zero explicit rows, no supersession; actual measurement uses empty successors.

## Targeted commands

Run from owned worktree, via Pyrun argv builders (no Bash):

```
cargo test --no-default-features --features sound,gui,casc,client-wrath --test patch_3_4_2_factory -- --nocapture --test-threads=1
rustfmt --edition 2024 tests/common/publication_sweep.rs tests/patch_3_4_2_factory.rs
```

Set `WOW_SIM_P342_FACTORY_OUT` to an absolute scratch JSON path to retain all 155 sweep observations. `WOW_SIM_P342_FACTORY_REGISTER` selects a full scratch inventory while enforcing exact 155 rows; no generator flag or regenerated ledger required.

First bounded probe: 18 factories accessible; classifier RED rejects unsupported `wrath-classic`. Discovery RED persists exact 83 mismatches against empty known list; unrelated control syntax failure retained honestly and fixed by using `exec` instead of `eval` for assignment. Final targeted GREEN result is recorded in `proof-ledger.json`/`green.log`; six cases cover widgets, exact inventory, CVar defaults, event nondiscrimination, retail-line rejection and foreign/empty successor behavior. Final formatter and exact code/input hashes are retained in proof ledger. Six inherited vendor-manifest deprecated lint-name warnings remain; no suppression/vendor edit.

## Ownership

No production API fixes, shims/fallbacks/native values, generator/source-ledger regeneration, other-branch edits, broad/check/profile/readability/startup/full-suite gates, deployment, push, merge or delegation. Parent owns integration (after 3.4.3), broad/profile/readability verification and any loaded-Blizzard-UI/native Wrath Classic measurements. Source audit's two prose summaries remain UNPROVEN. Standalone GREEN establishes only stable current factory observations, not completion of the parent goal.
