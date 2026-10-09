# Narrow validator discovery and Forever cfg repairs

Two distinct repairs, verified 2026-10-09 for documentation retention only: discovery has bounded historical proof at `894680abe`; cfg correction `0b5c69d6f` is **RED E0609** in retained independent proof; follow-up canonical `1044215d0`, copied as `2218c0f41`, has bounded compile/configured-name proof, not chat/removal or native acceptance. Neither establishes runtime/API parity or broader acceptance. Original evidence is copied byte-for-byte into a separate [integrated evidence directory](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/retention-manifest.json); original receipts and prior patch evidence remain unchanged.

## Validator discovery: historical bounded PASS

The retained independent report records `python3 -B tools/check_patch_validators.py 894680abe` at exact revision `894680abe9e7282453152f6372c34680d61e2cf0`, exit 0. Of 90 discovered paths, three archived `template-inputs` snapshots were excluded; 87 genuine validators remained. Clean phase **87/87**, synthetic later-audit phase **88/88**, no failures, missing/unexpected paths or executed snapshots. The extra later validator is the synthetic `9.9.9-synthetic-later-audit` input, not additional production coverage.

Discovery excludes only the exact path component `template-inputs`; root and other nested current proofs remain eligible. RED at `7d91acfd5` records two expected failures; GREEN at `894680abe` records eight tests passing. New fixtures cover a nested genuine validator and a genuine nested missing-seal failure. No dedicated root missing-seal fixture was added; the report distinguishes existing root failure propagation evidence from that missing fixture. The report also records 65 protected seal/template files unchanged.

These receipts invalidate only the discovery-related global gate failure at that revision. No fresh latest-HEAD gate was run for this documentation task. Source contracts marked UNPROVEN stay UNPROVEN; archived replay is not native/model implementation. Parent fullsuite614 remains user-reported FAIL (23+1+6 failures; 74 publication checks OK), not rerun or independently re-established here. Prior 50/50, six history and six copied proofs remain prior evidence. No broader acceptance.

## Forever cfg correction: RED; bounded follow-up PASS

Commit `0b5c69d6fdac137e57f4cb9a9f16114773234e34` changes only two module declaration gates in `src/c_api/mod.rs`: `addon_messages` and `c_combat_log` now compile under `retail-12-0-0` **or** `client-wowforever`. This is inspected implementation, not passing compile/test evidence.

The retained read-only cfg map explains the original E0433 boundary: Forever registration call sites referenced modules removed by Retail-only declaration gates. Registration intent is specifically Forever chat `SendAddonMessage`/`SendAddonMessageLogged` and combat-log removed-member publication for `GetCurrentEventInfo`; it does not authorize adding Retail `IsCombatLogRestricted`, BNet exposure, or the whole Retail feature bundle. The map predates the correction and its “not applied” wording remains preserved as historical evidence.

The failed source-model invocation never reached tests or assertions. [Retained failure/functions gates](forever-cfg-failure-and-function-gates.md) record the exact independent run: Forever E0609, zero tests; Retail library compilation PASS only; aggregate Retail behavior build-blocked by disabled iced. [Separate follow-up](forever-cfg-failure-and-function-gates.md#follow-up-bounded-compilestate-pass) records one exact repaired command at `2218c0f41`, exit 0, 1/1 configured pre-login name test. Chat/removal API execution and native-source compatibility remain UNPROVEN. Module availability and registration intent alone grant no runtime/API parity or broader acceptance.

## Sources

- [Independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-independent-report.md) — bounded conclusion and original auxiliary `/tmp` receipt references; those references are historical, not all retained here.
- [Global result](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-independent-global-result.json), [global log](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-independent-global.log), [discovery comparison](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-independent-discovery.json) — exact invocation/output and path counts.
- [RED result](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-red-result.json)/[log](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-red.log), [GREEN result](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-green-result.json)/[log](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/validator-discovery-green.log) — original development receipts, reused without rerun.
- [Cfg map](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-api-cfg-map.md) — read-only root cause and registration intent; `git show 0b5c69d6f -- src/c_api/mod.rs` establishes only the two applied declaration changes.
- [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/retention-manifest.json) — original source paths, sizes and SHA256 values; copying is not new execution proof.

## See Also

- [[index]] — integrated wiki catalog.
- [Client profiles](../systems/client-profiles.md) — profile boundaries, not parity evidence.
