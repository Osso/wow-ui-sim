# Historical retail Patch 2.4.2 API audit

Frozen page 81145/revision 803933, revision timestamp **2021-12-28T02:03:05Z**, documents **2008 retail 2.4.2**, not 2021 TBC Classic 2.5.x. Owned branch `p242-source`, base `f95eed96e`, isolated `/home/osso/.worktrees/wow-ui-sim-p242-source`. Bounded development evidence only: main owns integration, current full-Game publication and native/final acceptance.

## Provenance

[Source pin](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/source-pin.json), committed source-cache manifest/response/raw retained independently. No network or mutable cache inputs used to reconstruct the page.

| Input | SHA-256 | Bytes |
|---|---|---:|
| Original source manifest | `b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba` | 51,323 |
| Response | `334db0c7e0245ae9fd4255663f811456e18405c38688175ab0ef4286894163a8` | 3,083 |
| Wikitext | `c3602daf94581eb87a0974d68bc0cf476be5d35015cae0c10792690782c07c29` | 2,769 |
| Full finite remaining-page registry | `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c` | See sealed snapshot |

Registry contains 101 page identities through 1.0.0; pin and minimum boundary are checked in historical replay. Registry inclusion is not an audit/parity claim for all 101 pages. Own source navigation is `next=3.0.2|prev=2.4.0`.

## Literal accounting

[Coverage ledger](../../../data/patch-api/sources/2.4.2-page-coverage.json) retains **53 unique IDs: 2 bounded publication, 44 pending, 7 metadata**. Raw rows retain every nonblank original line; separate occurrences avoid converting callable existence into signature or prose credit.

| Occurrence class | Count | Proof/limits |
|---|---:|---|
| Nonblank raw lines | 22 | Exact literal strings/line numbers; seven navigation/heading/reference metadata, fifteen substantive pending rows |
| API publication inventory | 11 | Seven added globals; two changed globals; two changed widget methods |
| Separate signatures | 11 | Five explicit partial/abbreviated signatures, six unspecified; all pending |
| Constants | 6 | GOLD/SILVER/COPPER → GOLD_AMOUNT/SILVER_AMOUNT/COPPER_AMOUNT names only; no values/retirement credit |
| Contextual APIs | 3 | CombatLogGetNumEntries, QueryAuctionItems, GetAuctionItemInfo; not additions or independently changed APIs |

Five editorial section/subsection headings have no numerical count headers. No explicit event, CVar or console-command inventory appears; this is a literal source fact, not positive runtime parity. The AH `getall` scan remains prose, not a guessed console command. Plural operator, placeholder, examples, officer-note escape rules, AoE protection, cursor shift, `addToStart`, 128-query/30-second throttle and saturation/Unknown-name workload are all retained. References are retained without expanding forums, linked API pages or citation targets.

Opt-in flags only: generator `--retail-tbc-summary --client-line retail`; extractor `--retail-tbc-markup --lowercase-reflist`. Saved flags are in own provenance. Source fixtures prove exact eleven inventory identities, signatures/markup retained, and default own generator bytes unchanged.

## Publication versus real behavior versus native proof

Own supported **bare current-retail factory**, built with `--no-default-features --features client-retail`, measured eleven rows:

| Surface | Result | Credit |
|---|---|---|
| Minimap:PingLocation | Callable | Current factory publication only; protection during AoE targeting unproven |
| ScrollingMessageFrame:AddMessage | Callable | Current factory publication only; `addToStart` unproven |
| Nine globals | raw=nil; lookup=nil | Factory gaps, **not a measurement of loaded modern Blizzard wrappers/full Game** |
| Current C_CurrencyInfo.GetCoinText | Concrete amount/separator outputs pass | Existing namespace model only; no new alias or historical legacy/native credit |
| Historical native client | Not executed | Unmeasured; main-owned acceptance |

Factory RED with empty known-gap set records nine gaps; reviewed GREEN 2/2 includes separate currency behavior. Negative control replaces an actually published widget method with a nonexistent method: **9 → 10 gaps**, exactly `p242-negative-control`. Initial weaker replacement of an already missing global is also retained as a separate new/stale-ID control, not an extra-gap claim.

### Precise remaining gaps

| Source | Missing contract/proof |
|---|---|
| AcceptLevelGrant / DeclineLevelGrant | Pending grant/level lifecycle and signatures |
| CombatLog_Object_IsA / CombatLogSetCurrentEntry | Bare publication; real historical filter/cursor/history proof. Temporary compatibility elsewhere is not loaded here or native parity |
| GetCoinText | Bare legacy callable absent; existing current namespace decomposition/separator test does not install/prove legacy alias |
| GetQuestLogSpellLink / GetQuestSpellLink | Quest reward spell-link identity, selected/log context and return contracts |
| strreplace | Page supplies no signature/replacement/error contract; no guessed string alias |
| GuildRosterSetOfficerNote | Strict escape grammar and roster-write backend |
| Minimap:PingLocation | Protected-call/AoE security transition |
| ScrollingMessageFrame:AddMessage | Additional argument position/default absent from frozen page; no guessed stack slot |
| Plural operator / constants | Last-number rendering, unspecified *_P1 migration, exact constant values/localization |
| Name-resolution queue / AH workload | Resolver cache, queued throttle, clock/network lifecycle and workload/native timing |

No runtime code changed, no retirement, new shim/fallback, alias, or speculative model. Existing currency model is tested with 12345 and ` / `, 10101 and `:`, and zero, rather than credited for mere callable existence.

## Successor boundaries

Only actual retail registers **3.2.0, 3.3.0, 3.3.3, 3.3.5, 4.0.1**, ordered chronologically, participate. [Exact overlap review](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/overlap-review.json): one tuple/bare-symbol overlap, 3.2.0's **changed** GetQuestLogSpellLink; it supplies no add/remove supersession. Other four registers overlap zero own identities. Nine gaps unchanged.

Queued frozen 3.0.2/3.0.3/3.0.8/3.1.0 sources have zero exact own-symbol overlaps and receive no publication supersession credit. Original substring scout is preserved: 3.0.2 GetCoinTextureString is **not** GetCoinText. No Classic 2.5.x, Era or Wrath 3.4.x supersession.

## Sealed historical replay

[Historical manifest](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/historical-manifest.json) seals **23 original files** plus hashes for **2,229 archived blobs**. Compact deterministic archive is **4,287,031 bytes**. Original ledger, gaps, source/registry pins, original/current parsers, relevant original runtime source, build/test inputs and all owned command logs survive independently of future closures. Ignored `.log` receipts are force-tracked explicitly. Closure directory is separate; no closure claimed.

Historical validator materializes only sealed source/tool inputs into a fresh temporary root. No Git, target, current checkout, vendor or mutable UI-cache files are read; archived Rust code is provenance, not rebuilt native acceptance. Fresh-process fixture copies only evidence and runs with `PATH=/nonexistent`. Serialized ledger, log and compressed archive mutations each fail at their exact seal; restoration preserves original hashes and reproduces the same summary.

151 retained raw-source **default differential controls** and 78 existing **recorded-option differential replays** are unchanged against base parsers, including inherited errors. This does not claim all old saved fixtures already reproduced: inherited stored register differences remain 10.0.0/12.1.0 and the 5.0.4 register error; inherited stored extract differences remain 10.0.0/10.0.2/10.1.0/10.1.7/10.2.5/12.0.5/12.0.7/9.2.5 and the 12.1.0 extract error. All differences/error boundaries match the baseline, not new regressions. Own register/extract reproduce exactly.

## Proof ledger

| Revision | Command/scope | Result |
|---|---|---|
| f827c5e9a → 040c37714 | `python3 -B tools/test_patch_2_4_2_source.py` | Expected missing-interface RED → 2/2 GREEN |
| d6662ec9d → acf89c976 | `cargo test --no-default-features --features client-retail --test patch_2_4_2_factory -- --nocapture` | Discovery RED (9 gaps, model passes) → 2/2 GREEN |
| acf89c976 | Same target, `patch_2_4_2_factory_publication`, scratch env inputs | Negative exit 101; 9 → 10 exact gaps |
| d02bee963 → c8bdc1c1c | `python3 -B tools/test_patch_2_4_2_accounting.py` | Missing-module RED → 2/2 GREEN with omission controls |
| 1a5e3860d → 576d4dcca | `python3 -B tools/test_patch_2_4_2_replay.py` | Missing-validator RED → fresh-copy/tamper/restore GREEN 1/1 |
| fbf655ef9 | Replay fixture now exposes its previously asserted summary | 1/1 GREEN; original 23 seals unchanged |

Exact discovery revision and commands/environment scope are retained in [command ledger](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/command-ledger.json); [portable summary](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/portable-proof/summary.json) is separate from original evidence. Factory output paths use P242_SWEEP_OUT; scratch register uses P242_SWEEP_REGISTER; CARGO_TARGET_DIR is the owned worktree's target. Rust file formatted before commit. Six inherited iced manifest deprecations and six inherited headless non-vendor warnings retained, not suppressed/fixed outside scope.

No broad/check/lint/readability/profile/startup/full-suite/final gates, model CLI/delegation, push, merge, deploy or operational work. Later documentation/receipt additions do not invalidate earlier code-scope proof. Main owns integration and any changed publication/native acceptance.

## Integrated evidence retention — 2026-10-09

[Integrated proof and revision boundaries](integrated-source-and-factory-proof-2026-10-09.md) retain independent evidence separately from original source seals. Integration is not new runtime/native proof; historical log entries remain unchanged.

## Sources

- [Frozen raw page](../../../data/patch-api/sources/2.4.2-api-changes.wikitext) and [rendered extract](../../../data/patch-api/sources/2.4.2-api-changes.txt).
- [Spec](../../specs/patch-2-4-2-publication-sweep.md), [handoff](../../../data/patch-api/evidence/2.4.2-session-2026-10-09/handoff.md).
- [Current currency model](../../../src/c_api/item_spell/c_currency.rs) and [owned factory tests](../../../tests/patch_2_4_2_factory.rs).

## See Also

- [[patch-3-2-0-api-audit]] — actual retail changed quest-link successor.
- [[patch-3-3-0-api-audit]] — source accounting/publication/model/native separation.
