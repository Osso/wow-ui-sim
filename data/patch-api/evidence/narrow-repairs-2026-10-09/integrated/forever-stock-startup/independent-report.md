# Independent Forever stock startup receipt verification

Date: 2026-10-10. Receipt epoch: `20261010T184613Z` (paths below relative to this epoch).

**Startup completion: PASS. Distinct Lua-error-state command: FAIL. Overall clean-startup claim: FAIL.** Receipts became available within one bounded wait of at most 240 seconds. Verifier inspected existing artifacts only; no builds, tests, checks, simulator invocations, operational changes, or delegation. Only this report was written.

## 1. Build and artifact binding

- Recorded revision: `98f625741d01df768af6c3c97324a3cfbc544573` (`submission.json`, `artifact.json`). This is recorded HEAD, not proof of a clean checkout.
- Producer argv: `/usr/bin/cargo build --bin wow-sim --no-default-features --features sound,gui,casc,client-wowforever --offline --locked --message-format=json --timings -j 12`.
- `compile-result.json`: exit 0, no stream errors. Actual Cargo artifact record: `fresh=false`; normal dev profile, optimized + debuginfo. No instrumentation feature asserted.
- Actual artifact features: `aura-containers`, `aura-instance-enumeration`, `aura-xml-widgets`, `base-spell-relationships`, `casc`, `client-wowforever`, `forbidden-animation-aspects`, `forbidden-aspects`, `gui`, `native-duration-formatting`, `numeric-rule-formatters`, `on-update-modes`, `player-cast-durations`, `rodio`, `sound`, `timed-signal-maps`. Neither default/retail nor fast-build is listed.
- Original artifact: `/home/osso/Projects/wow/wow-ui-sim/target/debug/wow-sim`; both commands used epoch-local `wow-sim-sealed`. Independently rehashed sealed SHA-256: `6bb1854b25187d3b16e1cd618f15db58b6e160e52a7ab4d406123803525583b3`, matching artifact receipt and unchanged receipts for both executions.
- Toolchain receipts: Cargo 1.99.0 (`5f94df478`, 2026-08-27), rustc 1.99.0 (`b940084d7`, 2026-09-28), host `x86_64-unknown-linux-gnu`, LLVM 23.1.1; both probe exits 0.

## 2. Source, cache, and resource binding

- 3,853 tracked-file hashes in `source-before.json` and `source-after.json` match; independently compared current file contents with recorded hashes, all equal. This does not establish tracked membership stability, index cleanliness, untracked inputs, or external dependency provenance.
- Active cache: `/home/osso/.cache/wow-ui-sim/blizzard-ui/wowforever/AddOns`. Completion marker is `ok`; provenance records profile `wowforever`, product `wow_classic_beta`, version `1.60.1.69977`, source `casc-local-or-cdn`, fallback `none`. No provenance keys reproduced.
- Current manifest has 4,398 nonblank/noncomment entries. SHA-256 `36b9f64e39e4031c28218865516635e85e7bdd1042fcf3ef9db0f347037e5c24` matches cache provenance. Cache contains 4,730 files, not 4,398: manifest count and total cache-file count are distinct measures; no assertion of an exact manifest-only cache.
- `cache-before.json` equals `cache-after.json`; independent current recursive file inventory and hashes also match all 4,730 entries. Producer's after-cache snapshot occurs after dump, before the separate Lua-errors command; current comparison additionally finds no surviving cache change after both commands.
- Recorded admission: 33.1788 GiB available, load1 11.6904, no canonical Cargo process, admitted=true. No shared lock used. `--locked` refers to Cargo dependency lock, not checkout serialization. Recorded cgroup CPU cap `1200000 100000` (12 CPUs), memory limit 17,179,869,184 bytes (16 GiB); build peak 9,313,058,816 bytes. Submission records agents.slice and these limits. Cgroup after receipt precedes runtimes, so peak is not full runtime peak evidence.

## 3. Stock dump completion — PASS, not error-free

Recorded command: `/usr/bin/env -u WOW_SIM_LOG_HANDLER_TIMINGS /usr/bin/timeout 90 <epoch>/wow-sim-sealed --no-addons --no-saved-vars --exec-lua "print('[ForeverStockStartupComplete]')" dump-tree --filter __ForeverStockStartup_NoMatchingFrame__`, with `WOW_SIM_NO_SOUND=1`.

`runtime-result.json`: exit 0. Actual `[ForeverStockStartupComplete]` appears in `startup.stderr:845`. Actual startup output records SavedVariables and addon loading disabled, 272 Blizzard addons loaded. No matching-frame dump content is required by this filter. This proves bounded command completion, not rendered GUI/native parity or a clean startup. `startup.stderr` contains Lua diagnostics: 34 lines beginning `Lua error:`, plus five suppression notices totaling 17 suppressed repetitions; these are log-line counts, not a separately validated JSON error total.

No `[file-budget-success]`, `[file-budget-error]`, or `[handler-budget-error]` records in either runtime stderr. Zero such records does not imply zero Lua errors.

## 4. Separate requested Lua-error-state command — FAIL

Recorded command: `/usr/bin/env -u WOW_SIM_LOG_HANDLER_TIMINGS /usr/bin/timeout 90 <epoch>/wow-sim-sealed --no-addons --no-saved-vars lua-errors`, with `WOW_SIM_NO_SOUND=1`.

`lua-errors-result.json`: exit **1**. Independently parsed all `lua-errors.stdout` as a nonempty JSON array: **19 records, 19 distinct complete messages, 36 occurrences** (sum of actual `count` fields). This is a separate process/error pool, not a dump-process JSON snapshot.

Exact summary at `lua-errors.stderr:215–221`: `Status: FAILED`; `Lua errors: 19 unique, 36 occurrence(s)`; attributed owners 3; Blizzard_CompactRaidFrames 28, Blizzard_Game 2, Blizzard_SharedXMLBase 1; unattributed 5.

Observed JSON categories, preserving logged wrapper records rather than collapsing them into assumed root causes:

- Unknown `UNIT_AURA_BLOCK_LIST_CLEARED`: 16 occurrences; CompactRaidFrameManager OnLoad, CompactRaidFrameContainer (`Blizzard_CompactRaidFrameContainer.lua:61`), CompactPartyFrameMember2–5 (`CompactUnitFrame.lua:65`), CompactPartyFrame generated handler.
- `Container frame already has an OnAttributeChanged script`: 14 occurrences.
- Nil `dividerVerticalPool`: 6 occurrences; `Blizzard_CompactRaidFrameManager.lua:473`, dispatched through manager `:266` and Blizzard_Game `EventRouting.lua:4`.

No zero-error or clean-startup conclusion is supported. This report identifies observed failures; no fixes attempted.

## 5. Warnings, evidence limits, and receipt integrity

Compile stderr contains six individual deprecated hyphenated Clippy manifest-key warnings in `iced-wgpu-patched/Cargo.toml`, plus their six-warning summary. Cargo JSON also contains one compiler warning: unused `is_never_secret_aura`, `src/c_api/c_secrets.rs:110`. Total distinct warning diagnostics: seven, not eight (summary excluded). Neither runtime stderr contains explicit warning-prefixed records; both contain Lua errors. Thus build success is not warning cleanliness.

Full log files were consumed during inspection; raw vendor content, character cache identifiers, and provenance keys are omitted from this report. Runtime output indicates a read-only EditMode cache remains in use despite `--no-saved-vars`; stock/no-addon/no-SavedVariables is not a fully sealed or cold account-state environment. No native client, GUI, full-profile, test-suite, or dependency/source-to-artifact acceptance claim.

Receipt SHA-256:

| File | SHA-256 |
| --- | --- |
| compile.stdout | dba3470c63beb264e5417de3b0cfbb0179715e2a1d21fc488f450a49a08c5b21 |
| compile.stderr | 1fa023db034cd9ad0c7da3040958f80391a3a4c10ec72a935366f9bc2c5af202 |
| startup.stdout | df3287d48be1ce11e4655c4dc33f97d39a2642839063e20fb5fac3aeed1fc114 |
| startup.stderr | 640f2e14d88101f08e54efbbee35798db71df550692a4feb8de431e036321f77 |
| lua-errors.stdout | 56ee89fd73d7740bd32951cf5610078f0ae8838d7dea99fec97eba8e68e8fefa |
| lua-errors.stderr | 6e2c5fa62431c38712afbcf06fd06b7c73d4e3b986eeed4e13d686b593229199 |

Producer `outcome.json` says `CAPTURED`; that status denotes dump receipt capture only, not successful Lua-error-state verification.
