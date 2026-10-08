# Patch 6.2.4 API audit

Warcraft Wiki page **212964**, revision **2072181**, timestamp **2023-09-27T22:21:22Z**, refetched **2026-10-08**. Not a redirect or stub. Accounted **35 register occurrences + 46 retained plaintext rows = 81 IDs**. Current retail publication gaps: **zero**. Seven substantive plaintext rows remain pending; absence after later removal is not historical signature or behavioral parity.

## Source and extraction boundary

- [Pinned raw source](../../../data/patch-api/sources/6.2.4-api-changes.wikitext), [provenance](../../../data/patch-api/sources/6.2.4-api-changes.provenance.json), [register](../../../data/patch-api/sources/6.2.4-wikitext-register.json), [coverage ledger](../../../data/patch-api/sources/6.2.4-page-coverage.json).
- Additive generator flag `--indented-api-lists` retains three standalone New references, both identities in three indented rename examples, 25 bare removal names, and the explicitly removed `realmName` CVar. Default generator behavior is unchanged.
- Additive extractor flag `--retain-patch-diff-reference` retains `Patch 6.2.4/API changes/diff` as an **unexpanded** source reference. The diff subpage is not silently dropped or credited as audited. Existing `--preserve-examples` is recorded alongside it.
- Both new flags have concrete RED/GREEN fixtures. One extractor fixture's initial expected heading spacing was corrected; original failed log remains retained. Final fixture suites pass.
- Three GameAccount additions and their duplicate rename destinations are superseded by explicit **8.2.5 removals**; current raw/lookup absence is observed rather than treated as missing historical publication.

## Capability coverage

| Exact statement / feature | Handled | Still missing / proof boundary |
|---|---|---|
| Three New BN* globals and three Toon→GameAccount rename pairs | Every occurrence probed; current names absent after explicit later removals | Historical tuple/input/native rename parity not claimed |
| 25 named BN* removals | All current retail raw/lookup values absent | No extra runtime retirement necessary; classic compatibility remains untouched |
| `realmName` CVar removal | Value/default absent in cached retail | Does not remove unrelated realmName fields or variables |
| Separate account/game identities | Existing C_BattleNet backing tested with parent ID 91017, game IDs 62041/62042 and friend index 1; primary/secondary GUID lookups, wrong-kind GUIDs and state removal/count updates | Numeric-ID globals and historical tuple positions are not reintroduced |
| Strict Account ID vs Game Account ID inputs | Current successor distinction bounded in bare and cached tests | No exhaustive historical all-functions strict-input/invalid-ID contract or native capture; one pending row |
| `select(17, BNGetGameAccountInfo(gameID))` | Exact statement and example retained | Current GUID/table surface has no numeric gameID→parent query or 17-return tuple; two pending rows |
| `select(6, BNGetFriendInfoByID(accountID))` | Exact statement and example retained | Current GetFriendAccountInfo takes a friend index and selects its first game account; optional account selection is ignored. Historical active-account/sixth-return contract unmodeled; two pending rows |
| “many functions” architecture transition | Three named examples separately accounted; current parent/game backing exists | Unquantified transition is not an exhaustive migration contract; one pending row |
| Current realm via `GetRealmName()` | Existing temporary implementation inspected | Constant `SimulatedRealm` is not authoritative realm state. No realm-selection/update fixture on this page; one pending row |
| Automated diff transclusion | Reference explicitly retained | Linked subpage not expanded; metadata-only, no runtime credit |

No simulator `src/` changes, new shim, native-retirement gate, C API placement change, vendor Lua rewrite or constant promoted as modeled behavior. [Own behavior tests](../../../tests/patch_6_2_4_behavior.rs) exercise existing real `SimState.bnet_friends` data through C_BattleNet; GUID queries and counts are bounded successor coverage, not retroactive BN* tuple support.

## Retirement decisions and complete scans

**No new runtime retirements.** [Scan receipts](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-retirement-scans.json) retain **52 untruncated `/usr/bin/grep -rnE` scans**: every one of 25 globals plus `realmName`, across cached non-documentation retail AddOns and `src`/`tests`. Whole-word `\bName\b` includes indirect `pcall(Name, ...)`, `and Name then`, registrations and references. These are unqualified globals, so qualified and bare forms coincide; the CVar name is also unqualified.

All 25 global retail scans have zero matches. All corresponding src/tests scans are zero except `BNGetMaxPlayersInConversation`: two lines in `src/wrath/compat_bootstrap.lua`, retained for classic clients. `realmName` has 61 retail and 23 src/tests field/variable matches; those are not assumed to be CVar accesses and nothing is retired. All 26 named removals already satisfy current retail absence.

[Historical later-register check](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-later-retirement-check.json) records exact master and p703-page revisions and complete register sets; **no re-additions**. At original audit time, the unmerged 7.0.3 register was read from its Git snapshot. After integration onto master **846a30663**, `later_registers` contains the real **7.0.1** and **7.0.3** registers, followed by **7.1.0** and all remaining later registers. [Integrated supersession review](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/supersession-review.json) finds no identity intersection with either newly merged page; zero gaps remain unchanged.

## Original targeted proof (historical)

Evidence: [6.2.4-session-2026-10-08](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/). [Command ledger](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-command-ledger.md) records exact commands, revisions, exits, source scopes and log hashes. Long Cargo commands ran asynchronously into files with the dedicated `p624-page` target; no poll-wait or full integration suite.

| Command scope | Result |
|---|---|
| `cargo test --test prefork_full_ui -- patch_6_2_4` initial discovery | 1/1; all 35 observations OK, zero gaps |
| `cargo test --test prefork_full_ui -- publication_sweep` | **48/48**: 47 pages + factory; **8,917 observations** |
| `cargo test --test prefork_full_ui -- patch_6_2_4_cached` | **1/1** current successor identity/state case |
| `cargo test --test integration patch_6_2_4` | **1/1** same backing-state case in bare environment |
| `cargo test --test integration c_battle_net_probes::` | **11/11** existing real-model regressions |
| `cargo test --test prefork_full_ui -- blizzard_deprecated_battle_net` | **3/3** existing cached loader/publication boundary |
| Generator / extractor / validator Python fixtures (`python3 -B tools/test_*.py`) | **31/35/8** passing |
| Recorded source reproduction | **47/47 registers**, **44/47 extracts**; inherited 12.0.5/12.0.7 mismatches and 12.1.0 template error unchanged |
| Exact negative control | Replace one added identity; **0 → 1** gap, expected exit 1 |
| `cargo fmt --check` | Exit 0 |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; **zero non-vendor warnings**, six inherited iced manifest deprecations unsuppressed |
| All prior `validate.py` / `validate_integrated.py` files | **27/27** passing |

All **239** pre-audit source files and **100** old extraction-mode outcomes remain unchanged. Counts derive from files; the three inherited extract failures are not relabeled as successes. No runtime path changed against master, so the conditional addons-enabled master/branch `lua-errors` comparison was not required or claimed. No CASC texture tests were selected on this host without a WoW install.

## Validator and integration boundary

[Read-only validator](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/validate.py) derives historical register/sweep sets using `historical_registers` / `historical_sweep_tests` at recorded revision **a690a6959**, never current globs or receipt-derived subsets. Evidence paths are relative to the resolved checkout; absolute cwd/target equality is not a gate. Wiki preservation is checked against a sealed historical revision, not mutable current wiki text. Complete history containing recorded revisions is required.

Own validator passes, with **28/28 validators** also passing in an independent shared clone at sealed revision **64426ce39**. [Portability receipt](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-validator-portability.json) proves additional audit registers/sweeps and edited current wiki text do not expand historical scope. Whitespace tampering in either own accounting or a protected prior register fails; restoration passes. The clone was removed after proof. The [wiki preservation receipt](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-wiki-integrity.json) seals accounting revision **0b0bc4117**, preserving all original index/log text and growing **2667 → 2671**, **390 → 394** lines. The unexpanded diff and seven historical/model limitations remain explicit, not hidden by a passing publication sweep.

## Integration onto master 846a30663

Fresh evidence lives in [integrated/](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/), separately from every sealed original artifact. [Gap comparison](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/gap-comparison.json) records **49 pages / 50 sweep-and-factory cases / 9,051 observations**. Exact-master snapshot passes **49 cases**; all **48 other pages' complete observations** are byte-equivalent JSON data. Own **35/35** observations and **zero gaps** remain unchanged. No new supersession, retirement, runtime change or historical parity claim; seven substantive plaintext contracts remain pending.

| Integrated command scope | Result |
|---|---|
| Own prefork publication sweep | **1/1**, zero gaps |
| All prefork publication sweeps | **50/50**; exact master **49/49** |
| Prefork / integration `patch_6_2_4` | **2/2**, **1/1** |
| Integration `c_battle_net_probes::` / prefork `blizzard_deprecated_battle_net` | **11/11**, **3/3** |
| Generator / extractor / validator fixtures | **32/36/8** |
| All saved register / extract reproduction | **49/49**, **46/49**; inherited 12.0.5/12.0.7 mismatches and 12.1.0 template error unchanged |
| Negative control | **0 → 1** gap, expected exit 1; exact original one-row mutation reused |
| `cargo fmt --check` | Exit 0 |
| Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings, six inherited iced manifest deprecations unsuppressed |
| All historical/integrated validators | **32/32**, including the new sealed integrated gate |

[Command summary](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/command-summary.json) retains commands, source revisions, exits and log hashes. All Cargo uses the dedicated `p624-page` target; background runner streams logs without poll-wait. `extend_patch_audit_receipts.py` supplies real 7.0.1/7.0.3 register and sweep rows. Reproduction preserves **249 master source files** and every recorded legacy extraction-mode outcome.

The sole initial validator failure was 7.0.3's strict tool hash gate. Its [updated validator](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/validate.py) admits only the **four exact tool/fixture blobs** between master **846a30663** and **0e40aff7e**: opt-in 6.2.4 additions coexist with `--legion-prepatch`. Every unrelated byte change remains rejected. Its original validator/seal table and failing log are preserved here; only its validator's own seal entry is refreshed. A concrete test accepts original/exact replacement bytes and rejects whitespace tampering, wrong recorded hashes and unrelated paths. No accounting or gap invariant is relaxed.

[Integrated 6.2.4 validator](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/validate.py) fixes register/sweep and prior-validator sets via `git ls-tree` at **f260499ee**, not live files or receipt subsets. It seals integrated artifacts, checks original preservation, exact master comparisons, flags, inherited failures, command input scopes, negative control and prior gates. Mutable current wiki text never changes historical proof scope. [Final validator matrix](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/validator-matrix.json) covers all **32** gates at evidence revision **9abcd6e98**; the integrated gate passes with 15 command receipts and 31 prior validators.

## Sources

- [Raw fetch and HTTP receipt](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-fetch.json).
- [Exact gap review](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-gap-review.json) and [extract scout](../../../data/patch-api/evidence/6.2.4-session-2026-10-08/p624-extract-scout.json).
- [Publication contract](../../specs/patch-6-2-4-publication-sweep.md).
- [C_BattleNet backing](../../../src/c_api/c_battle_net.rs), [temporary realm default](../../../src/lua_api/workarounds/temporary/client_info_defaults.rs).

## See Also

- [[patch-7-1-0-api-audit]] — source/evidence template and next merged later register.
- [[patch-audit-validator-portability]] — fixed historical scope and exact protected-input preservation.
