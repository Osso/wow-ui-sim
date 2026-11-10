# Full-lockdown model lane report

**Date:** 2026-11-10  
**Base:** `93125f591108a45dac8b124dee1cc59a9524fa2b` (`origin/master`)  
**Tested implementation HEAD:** `24a4d5e4a792dd86da7dfd5ef2ce2fb23f4b3c44`  
**Latest code/data HEAD before this report commit:** `5599a552e5be0456cbbf1186f438d23e06e2b593`  
**Branch:** `full-lockdown-model`; no push or rebase.

The simulator now models the R18 full-lockdown flag and the separate access-allowed flag. It honors scoped `fullLockdown="true"` and `false`, creation-time parent inheritance, the `IsForbidden()` union, and the five measured locked-down-disallowed API consumers. The XML loader uses a private constructor for XML-created FontStrings, so it does not call the public API that rejects a fully locked receiver.

## Evidence and coverage

| Behavior | R18 evidence level | Simulator proof |
|---|---|---|
| An object is fully locked when the full-lock flag is set **or** access-allowed is absent; base construction sets access-allowed. | VERIFIED predicates and writers. | `Frame::is_fully_locked_down`; test seeds absent access-allowed and observes locked parent/children. |
| A child inherits a parent's full-lock state and missing access-allowed state at creation. | VERIFIED creation path; completed child outcome INFERRED. | XML fixture covers both a full-lock parent and an access-allowed flag cleared before XML child creation. Existing-child/reparent propagation is not added. |
| `ScopedModifier/@fullLockdown` sets the creation default; `false` clears it in the scope and scope exit restores the prior default. | VERIFIED parser and scoped-default writer; completed object state INFERRED. | True/false/nested-parent/restoration fixture. |
| `IsForbidden()` returns true for ordinary forbidden state or full lockdown; `SetForbidden()` changes only ordinary forbidden flags. | VERIFIED. | Secure Lua probes cover both states and show that `SetForbidden()` alone still permits the measured FontString APIs. |
| The five measured APIs use the locked-down-disallowed resolver. | VERIFIED API/check list; completed Lua error outcomes INFERRED. | Secure and tainted probes assert the exact `Attempt to access fully locked-down object.` error. |

The five APIs are `DurationTextBinding:SetFontString`, `FontString:CanWordWrap`, `FontString:CanNonSpaceWrap`, `FontString:ClearAlphaGradient`, and `Frame:CreateFontString` when its receiver is fully locked.

**VERIFIED local source locations:** Retail `UI.xsd:1266` declares `fullLockdown` as a boolean with default `false`. Cached Retail and WowForever `Blizzard_AuthChallengeUI.xml:5` open a `fullLockdown="true"` scope; FontStrings occur at `:37` (`Label`) and `:107` (`Prompt`). The Retail manifest lists the AuthChallengeUI TOC/XML at lines 871–872 and `UI.xsd` at line 3392; the WowForever manifest lists them at lines 882–883 and 3723. Both TOCs contain `UseSecureEnvironment: 1` and `AllowLoad: Both`. Retail and WowForever cache XML bytes match SHA-256 `a0ffc89f0db9847754d1189641100be9775887c70c916d66fd33b56bb3537c5b`; the Retail XSD hash is `0384f91ba56e2e070b6f215c495d8ea63156c3c501b5333365b3ffdddd7f2478`. R18's earlier cache-build example remains labeled UNVERIFIED for its exact client-build identity.

## RED acceptance proof

Tests were committed before the producer (`d3b4fc8ec`). The initial Retail run failed as expected because the false scope did not clear the creation default: `tests/full_lockdown.rs:57:6`, `explicit false clears the creation default`. Log: `/var/tmp/osso/prship/full-lockdown-red-retail-case1.log`. The producer commit is `24a4d5e4a`; acceptance refinements are in `9d03ad245`, `bcd152bab`, and `b64979e2d`.

## Requested gates

All listed compiler builds reported **0 compiler warnings and 0 compiler errors** in Cargo JSON messages.

| Gate | Result |
|---|---|
| `cargo check --all-targets` — Retail, PTR, Mists, Era, Anniversary, WowForever | PASS for all six profiles; 0 warnings each. Receipts: `full-lockdown-check-{retail,ptr,mists,era,anniversary,wowforever}` under slot-1 receipts. |
| Wrath `cargo check --no-default-features --features sound,gui,casc,client-wrath` | PASS; 0 warnings. Receipt: `full-lockdown-check-wrath`. |
| Retail `cargo test --lib` | PASS: 2,243 passed, 0 failed, 1 ignored; 0 warnings. |
| WowForever `cargo test --lib --no-default-features --features sound,gui,casc,client-wowforever` | PASS: 2,011 passed, 0 failed, 1 ignored; 0 warnings. |
| Integration filter `full_lockdown` — Retail and WowForever | PASS: 7/7 each. This includes `Blizzard_AuthChallengeUI` and the existing `DurationTextBinding:SetFontString` XML-scope test. Logs: `/var/tmp/osso/prship/full-lockdown-green-retail-cases.log` and `full-lockdown-green-forever-cases.log`. |
| Retail integration filters `forbidden`, `duration`, `cooldown_restriction` | PASS: 47/47, 147/147, and 17/17. Each has zero failing names; no new failure names versus the source-matched base runs. Logs use `full-lockdown-green-retail-filter-*`. |
| `cargo test --bin wow-cli --features tools audit_api::patch_manifest` | PASS: 67 passed, 0 failed; 0 compiler warnings. Receipt: `full-lockdown-manifest-bin-fixed`. |

The manifest test first found historical evidence hashes for source files changed by this lane. The source hashes already matched the exact `93125f591` base bytes. Commits `7f9e97bb8` and `5599a552e` changed 28 evidence references across three checked-in manifests to immutable `git:93125f591...:<path>` references and left the recorded hashes unchanged. The final manifest-bin run passed all 67 tests.

## UNKNOWN and out of scope

UNKNOWN: live native reproduction; whether arbitrary AddOns may declare `fullLockdown`; which engine contexts mask the requested default; the complete API/writer inventory; the origin of a missing access-allowed flag; teardown behavior; propagation to existing children; and reparent behavior. R18 found no Lua full-lock mutator, but that search does not prove none exists. These cases remain unimplemented. Full lockdown is not mapped onto `SetForbidden`, aura access restrictions, `IsProtected()`, or `InCombatLockdown()`.

## Completion

Slot 1 was released with `python3 /var/tmp/osso/slot-claim.py release full-lockdown-model 1`. The worktree was clean at `5599a552e` before this report/wiki update. Final report/wiki commit and current HEAD are sent to the parent.