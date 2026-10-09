# Patch 1.60.1 literal SOURCE audit

Independent bounded audit at base `66a5c16599cdf48243ebf8348e78cd3e3c829c26`, branch `p1601-source`. Frozen page 707613/revision 6902509 (`2026-10-07T05:30:51Z`), cached response and body only; no network or native client. Source accounting is not publication, runtime or native parity proof.

## Source, configured and native identities

Literal resources line 5 says `TOC: 16001`; local September 15 blue-post text names “Forever and modern WoW (Midnight)” as game types in the Mainline family, “Camelot … and Standard”. Source examples use `WOW_PROJECT_CAMELOT`, `camelot`, metadata `Camelot` and an interface range strictly between 16000 and 20000. Those are source claims/examples, not installed production policy.

| Evidence | Exact scope | Proof |
|---|---|---|
| Source | 1.60.1 (70205), October 2, 2026; comparison 12.1.5 (69952) → 1.60.1 (70205) | Frozen text, not native capture |
| Source sub-build claims | 70170/October 1; 70009/September 23; local blue-post 70009/September 24 and 70058/October 2 titled October 1 | Separate literal dates/builds retained |
| Configured feature | `client-wowforever` in `Cargo.toml`; **no `client-wowforever16001` feature** | Static base-code observation |
| Configured interface | `ClientProfile::WowForever`, `ACTIVE_INTERFACE_VERSION=16001`, interface method returns 16001 | Static source snapshot, not runtime observation |
| Configured compatibility identity | `client_info_defaults.rs`: version 1.60.1, build **69977**, synthetic date June 17, 2026; explicitly temporary metadata defaults | Not build 70205 or native session identity |
| Native/client/loaded UI | None measured | UNPROVEN |

The September 15 prose says the “vast majority” of 12.1.5 APIs and secrets/aura UI changes are shared. It does not establish all APIs, historical epochs, security behavior, or loaded UI. No Retail/Era/TBC retirement/supersession is inferred. `next=1.60.2` stays pending navigation; actual successor integration belongs to main.

## Exact frozen byte identities

| Artifact | Bytes / SHA-256 |
|---|---|
| Body | 125,873 / `f0e3dc86be50c02dbbc2a5962adda9cf96aa8f3a3959995a4cf7a6a9c8b43ce7` |
| Response | 128,467 / `687133019ca6e39cfe2c2902fd398ac2c6f161cab5aed52e7809ca562c6ea5e5` |
| Registry | 101 pages, endpoint 1.0.0 / `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c` |

The original response's main-slot body equals the cached wikitext exactly. Manifest pin membership and every retained manifest page identity are compared to the sealed registry, independent of current Git or collector state.

## Complete coverage matrix

All original behavioral rows are **UNPROVEN**, capabilities empty. Exact row-level contracts and line numbers live in evidence `original/ledger.json`; no names are deduplicated or repaired.

| Literal category | Count | Source contract / limit |
|---|---:|---|
| Global API added / removed | 175 / 10 | Names only; arguments/returns/state/coercion/defaults/security unspecified |
| FrameXML added / removed | 471 / 1,041 | Named UI-source diff, not native-global absence; loaded vendor implementations and effects unmeasured |
| Widgets added / removed | 7 / 0 | Four ScriptRegion focus/mouse methods, ModelSceneActorBase model-by-unit-display method and two Minimap cursor/mouseover methods; no argument/return/effect definitions |
| Events added / removed | 25 / 2 | Names only, no payload/producer/order/firing proof |
| CVar-column added / removed | 138 / 7 | Added column includes one command `dumpSmallAlloc` (type command/category 4), leaving 137 added CVars; seven removed CVars |
| Total inventory | 1,876 | 816 added / 1,060 removed occurrences; removal is comparison text, not runtime retirement |
| Nonblank source rows | 2,070 | 1,876 inventory / 107 metadata / 87 other literal substantive rows |
| Headers | 32 | 22 ordinary/template headings and 10 numeric column headers |
| Signature ledger | 1,712 | 1,704 identity-only/unspecified signatures; eight literal call occurrences from prose/examples |
| Prose limits | 211 | 87 other substantive rows plus 124 literal CVar descriptions; identity proof cannot close described effects |
| Linked reference boundaries | 15 | Nine external links / six wiki links; linked bodies unexpanded and UNPROVEN. Zero explicit transclusions in this frozen page; synthetic transclusion fixture also stays unexpanded |

Numeric headers: global added **176 versus 175** literal rows; events added **24 versus 25** rows. Other eight headers reconcile (10, 471, 1,041, 7, 0, 2, 138, 7). Do not invent a missing API or drop an event to fix counts. All CVar named fields, including absent defaults, scopes, categories, formatting and descriptions, remain literal strings. Examples: `CameraFollowPitchOffset` default `15.000000`/Account; `ClientSettings_LOW_LATENCY_CBS_BY_API_MASK` has no default; `winePlatformTTS` describes a crash risk, not permission to enable it.

No duplicate section/symbol/direction inventory identities occur in the frozen inventory. Repeated headings and repeated literal UnitName citation occurrences remain separate. Misspellings such as `PaperDollItemSlotButton_OnModifableClick`, `GarrisonMissonListTab_SetSelected`, and `GarrisonMonuntmentFrame_*` are preserved. A concrete duplicate/malformed fixture retains both `C_Example.Read` occurrences and `C_Example.Mispelled`; no guessed alias is added.

## Literal signature and prose boundaries

Eight call occurrences are `UnitName("player")` twice on line 12 (statement plus linked citation title), `select(4, GetBuildInfo())` and nested `GetBuildInfo()` on line 54, `C_AddOns.GetAddOnMetadata("MyAddOn", "X-Game")` on line 64, and the three `C_GameRules.IsGameRuleActive(Enum.GameRule.*Ruleset)` conditions on lines 72/74/76. The quoted `"Unknown", nil` is the described **old failure**, not the corrected return tuple. No inventory row supplies a complete signature.

Local prose accounts build-qualified constant 18, UnitName pre-login/token consistency, SavedVariables and RestrictedEnvironment fixes; TOC allow/exclude/file/title-order examples; game-type and ruleset examples; blue-post shared-architecture/secrets/aura statements; cooldown disabled-by-default/class/rank coverage; flyout/bar/targeting/binding/hover/layout gamepad changes; character-sheet low-rank benefit/proc example; character/art/stable/combo/name/auction changes. Every exact statement has its own retained row and limits; linked details are not reconstructed. Non-inventory `literal-extract.wikitext` retains original markup/scaffolding without MediaWiki/template expansion. Shared generator defaults/flags are unchanged; original default register bytes reproduce exactly.

## Existing model candidate and production-edit policy

Selected narrow contract: line 12's `UnitName("player")` must not return the unknown placeholder for a configured player before login. Existing `group_queries.rs` reads `SimState.player.name`; no runtime edit is proposed or made. Owned test sets Ada then Grace, independently holds Linus in a second environment, and reads the names before any test-dispatched PLAYER_LOGIN. This can prove a current configured-state read, not native login ordering, corrected arity/realm returns, unavailable/default handling, arbitrary token aliases, security or loaded UI. Token consistency remains separate and UNPROVEN.

If this test reveals a runtime defect, stop before production changes and report the literal counterexample/proposal. No added aliases/defaults/coercion/shims/fallbacks or production-policy changes are authorized here.

## Bounded proof ledger

| Command / scope | Revision | Result / validity |
|---|---|---|
| `python3 -B …/test_source.py` before accounting/validator existed | Base 66a5c1659 plus fixture | RED: 8/8 missing-validator failures; retained original/red.log |
| Default shared generator, frozen page, no flags | Base 66a5c1659 | Discovery: 1,876 rows; exact original/register.json bytes; no runtime credit |
| `rustfmt --edition 2024 patch-tests/patch_1_60_1_source_model.rs` | New test only | Exit 0; no check/lint/readability gate |
| `python3 -B …/test_source.py`: own SOURCE fixtures | `88ce7032563b32bc0b8ccac8fe0097124abe4d37` | GREEN 8/8, 0.592 s; 5,916 per-row omission controls; copied fresh process with empty PATH/no Git/target/current tools passes; disk ledger/red-log tampering rejects and exact bytes restore |
| Owned model target preparation (`cp -a --reflink=always` dependency snapshot) | Same revision | BLOCKED before Cargo execution: /tmp is cross-device; bounded same-device retry reports Operation not supported. Original caches untouched; no expensive full copy or original-cache fallback |
| Intended `cargo test --offline --locked --no-default-features --features client-wowforever --test patch_1_60_1_source_model -- --nocapture` | Test committed at `291cc22c8`; receipt at `88ce70325` | **NOT EXECUTED**. Zero meaningful model closures; local observed compiler 1.99.0, unlike documented 1.98.1 |

Original source-only ledger and gaps are immutable: **19 original seals**, largest original file 3,326,475 bytes. Later proof/attempt receipts and empty closure claims live separately under `current/` with **eight separate seals**. Both full preparation logs are losslessly gzip-compressed (627,865 / 633,544 bytes); uncompressed hashes are recorded, every retained file is below 5 MB. No original seal or behavioral status changed. Current source proof remains applicable: later changes are receipts/docs only, so no redundant source-suite rerun. No whole-project/check/lint/readability/coverage/startup/final gate is run. Main owns actual successors, integration and native acceptance.

## Sources

- [Frozen source pin](../../../data/patch-api/evidence/1.60.1-session-2026-10-09/original/source-pin.json)
- [Exact source ledger](../../../data/patch-api/evidence/1.60.1-session-2026-10-09/original/ledger.json)
- [Spec](../../specs/patch-1-60-1-source-accounting.md)
- Base `Cargo.toml`, `src/client_profile.rs`, `src/lua_api/globals/group_queries.rs`, `src/lua_api/workarounds/temporary/client_info_defaults.rs` (sparse sealed code snapshots).

## See Also

- [[client-profiles]] — configured profile architecture, not native identity.
- [[patch-3-4-2-api-audit]] — publication versus state/native proof boundaries.
