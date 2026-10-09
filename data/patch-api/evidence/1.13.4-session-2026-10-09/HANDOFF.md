# Bounded Patch1.13.4 handoff

Own `/home/osso/.worktrees/wow-ui-sim-p1134-page`, branch `p1134-page`, base `3de87465828db7cc7f6f900d4b63e24f1b399825`. Main integrates1.13.4 before parallel1.13.3. No push/merge/deploy/delegation/network, other-checkout operations, vendor/Blizzard/Wowless changes, cache copying, broad tests/check/lint/readability/coverage/final gates. PLAN.md ignored, not tracked.

## Frozen accounting

Page333398/revision3216451, timestamp2021-05-06T13:21:20Z; retrieved2026-10-09T08:51:36.400398+00:00.2146bytes; rawSHA256 `a8cbcc1f62bb90e33a40334dece4f943b3f44648c34d50fbbb0a55160b8e00f5`, responseSHA256 `a49c62f6c4663ab12a8eac66083676c1284ab5f5ac870023e542b892924ffc55`. Frozen manifest/response/wikitext/101-page remaining registry unchanged from base. Literal source lacks client name; explicit audit history Era, current Era/Anniversary11507 not source11304/build33491 or native proof.

Derived63physical/57nonblank raw/10default-rendered rows;30metadata/27UNPROVEN raw.23 inventory (17globals/5events/1CVar),23 unspecified signature records, no declared signatures/defaults/examples,6headings/3counts/1caption/2prose/5links/26templates/no ref bodies;30UNPROVEN identity/prose/link contracts.212 occurrence omission controls. Generator/extractor defaults flags[] retained; no shared tools edits.

## Meaningful behavior and gaps

Existing current Era GetTotemInfo model demonstrated1/1: host slot active→replacement→removal, expired and0/99 out-of-range. Actual host state/time expiry, not name factory. No new runtime behavior or historical/native contract closed. First test harness E0277 u32 conversion RED preserved at d478d50f7; corrected supported i32 result conversion at ce0c5c59a. Stale security comment saying empty-only was not accepted as actual model evidence. No production/security changes.

All30 historical contracts remain UNPROVEN. GetTotemTimeLeft/TargetTotem lack exact owned implementations and page signatures/targeting semantics. China token support lacks regional policy/payment activation; generic existing commerce is not parity. Exact HasPurchaseableProducts spelling not aliased to DoesGroupHavePurchaseableProducts. Existing summon readers lack ConfirmSummon; acceptance/decline/event effects unspecified. Commentator/wargame/lock/timestep callables lack implementations/contracts; event names have no payload/producers/order; CVar default/scope/color mapping absent. Detailed exact causes/code snapshots in model-review.json/state-scan.json.

## Separate actual successors

Applied18 actual same-Era ledgers present at base, exact immutable copied inputs and ordered occurrence assessment. Only overlap:1.14.0 removes C_Commentator.GetUnitTeamIndex. Precedence only, not semantic/native retirement or model closure.1.13.5/6/7 no exact overlap. No normalization, linked foreign subset/native expansion or imported runtime proof. Frozen ledger later_registers=[]/18not-applied queue and original seal map unchanged. Future changed main integration/native/end-to-end successors outside this bounded page goal.

## Exact proof scope

SOURCE8/8 at c9c0262e371aef82a8a04f9edf1181f0b6a9ca41; successor3/3 at acddd6ef677615965401b8a731c0ebc4ff862714; Era1/1 at ce0c5c59a432cc497e58eb882419af5b92dea0cc. Portable3/3 and historical validator exit0 at bec06a7a2192aab1bd92367c12e5a6520c702c37. Portable workers copied SOURCE8/successor3, default-register/default-extract exact bytes; nine fresh processes, empty PATH, copied cwd, no Git/target/current tools/workspace lookup. Both serialized ledger/log seal tampers rejected; exact original bytes restored and112seals/validator pass.113-member390803byte archive; original112seals/3122928bytes. Original mapSHA256 `543edf3bd874f54ae190e2df92254db6f808a986728512098a8ad8620171f66f`; later GREEN receipts separately sealed.

Proof argv and exact start/end epochs live in original proof-ledger.json/current-proof-ledger.json and per-command receipts. Primary Era argv:

```
cargo test --offline --locked --no-default-features --features client-era --target-dir /home/osso/.worktrees/wow-ui-sim-p1134-page/target --test patch_1_13_4_totems -- --nocapture
```

Every outer command executed via cli.*.cwd(ownpath). Portable worker subprocesses use copied evidence cwd to avoid original-directory dependency. Source/successor argv `python3 -B <absolute-own-evidence>/test_source_accounting.py` / `test_successors.py`; portable full argv in portable-green-proof.json/REPLAY.md. No aggregate/iced test target or model test rerun at final receipts.

Epochs: Era1791563716.405308→1791563717.3497257; SOURCE1791563998.5832474→1791564005.1683917; successor1791564214.1217024→1791564216.0583503; portable1791564314.7840827→1791564319.1907156. Python tests emitted no warnings. Rust retained6existing library warnings/1binary unused-import warning/6vendor manifest deprecations. No suppression or warning-free claim.

See docs/specs/patch-1-13-4-source-accounting.md and docs/wiki/investigations/patch-1-13-4-api-audit.md. Main owns integration/native/final-goal gates; these are targeted development receipts only.
