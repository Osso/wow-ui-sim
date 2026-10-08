# Patch 7.3.2 API audit

Verified source 2026-10-08: Warcraft Wiki page **230850**, revision **6200179** (2024-12-21T14:17:59Z), refetched rather than inherited from the remaining-pages inventory. Current retail is the target. [Spec](../../specs/patch-7-3-2-publication-sweep.md) defines the boundary.

## Source accounting

The complete page contains one Changes statement: **Logout and Quit lua functions are now protected**. Both API-linked occurrences are retained as `changed`, not invented additions/removals. The register has two identities; the extract has five identities (one compound behavioral statement, four source/heading/reference contexts). The ledger accounts for seven identities; no raw context is duplicated outside the extract.

Generator `--prose-api-links` is opt-in; existing inventory/bullet/rename flags do not capture Changes prose API links. Extractor reuses `--retain-reference-notes`, accepting capitalized `Ref web` only behind that flag. Prior default behavior remains unchanged. Linked Reddit discussion and reference list are retained but not expanded. The integrated generator retains both 8.0.1's `--bfa-prepatch` and 7.3.2's `--prose-api-links`. Rebase onto `af7a101e2` incorporates the merged 8.0.1 audit; `ad188f494` replaces the later-register placeholder with its real register, followed by 8.1.0 and all later registers. Integrated reproduction retains each page's recorded flags.

## Coverage matrix

| Contract | Implementation / observed evidence | Boundary |
|---|---|---|
| Publication | Two real session globals; initial cached sweep passes both | Presence is not protection |
| Insecure-call rejection | Initial bare and cached probes show both tainted calls succeeding and mutating state; retail guard now checks rilua active call-frame taint before mutation | Bare 2/2 and cached 2/2 cases pass |
| Secure transition/recovery | Existing `is_logged_in` and `simulator_exit_requested` state remain the action model; test asserts secure calls after caught failures | No native countdown/travel/process exit claim |
| Older profiles | Gate is retail-only; explicit Mists preservation test | Simulator preservation, not native historical parity |
| Native policy/lifecycle | Precisely recorded, not shimmed | Block-event/error wording, hardware-event entitlement, logout countdown/cancellation and historical/secret/restricted-context parity unverified |

## Root cause and modeled fix

Existing `session_exit.rs` published real actions but performed state mutations without checking insecure execution. Discovery proves stack taint was `Patch732Addon`; both Logout and Quit still succeeded, respectively clearing login state and requesting simulator exit. The failure occurs in the simulator action boundary, not Blizzard GameMenu Lua or the VM's ability to label the call.

Modeled globals move to `src/lua_api/globals/real/session_exit.rs`. A shared retail-only guard checks taint across rilua's active call frames, returns an explicit Lua error for insecure code, and runs before either state mutation. Checking only the current native frame failed: native closures have their own untainted `CallInfo`, while the Lua caller carries addon taint. The corrected active-stack check matches pinned rilua `debug.getstacktaint()` semantics; no VM/vendor change is needed. Secure Quit retains GUI-owned exit-request semantics; secure Logout retains the existing immediate login-state transition. ForceQuit/ForceLogout/QuitGame remain on their previous paths. The existing CancelLogout no-op is relocated to `workarounds/temporary/session_exit_defaults.rs`; it is unchanged, names the missing countdown/cancellation model, and receives no coverage credit.

Tests retain concrete state observations rather than merely checking function type or source text. They record taint separately from the call result so a failed taint assertion cannot masquerade as action protection. Cached probes run after unmodified real Blizzard UI startup. Secure calls following rejected insecure calls exercise error-boundary recovery in the same VM.

## Retirement decisions

**No retirements.** Source has no removals. Both changed members have live cached consumers and must remain published. [Complete scans](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-removal-consumers.json) use `/usr/bin/grep -rnE` with whole-word `\b`, excluding `*Documentation*` files/directories. Qualified and bare scans coincide for these non-namespaced globals; complete `src/`/`tests/` outputs include indirect callers. Cached consumers: six Logout lines, three Quit lines. Neither the audit-base later registers nor the recorded p801-page 8.0.1 register snapshot re-adds either symbol. A final [caller/master recheck](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-whole-callers-after.json) retains thirteen source/test lines per member and confirms no re-addition in merged master `af7a101e2`'s 8.0.1 register. No vendor files were edited.

## Recorded problematic boundaries

- **Native block notifications/error text:** page supplies no event name/payload or error wording, and no native capture exists. Explicit simulator failure is not native event parity.
- **Hardware-event policy:** no captured entitlement/gating rules; current stack taint proves only insecure-call rejection.
- **Logout countdown/cancellation:** existing state has no pending deadline/rest-area policy; immediate secure logout is preserved, not claimed as a countdown model.
- **Historical/secret/restricted execution:** retail target and lack of native captures cannot prove reconstructed 7.3.2 or full restricted-context policy.

## Integrated caller coverage

Two additional cached cases exercise the live GameMenu logout callback and QUIT acceptance/CAMP acceptance/cancellation/QUIT hide callbacks. ForceQuit retains its existing exit transition; cancellation retains the documented no-op, not native countdown parity. Integration and prefork session/menu/popup regressions are required by [integrated proof](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-integration-proof.json).

The supplemental slash-command diagnostic found an inherited registration gap **before** any session action: active mode `0`, Standard enum `1`, no secure `/logout` registration. [Diagnostic and unchanged master input hashes](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-integration-slash-registration-gap.json) establish that the C_GameRules/state/enum inputs match `af7a101e2`. The test's default-registry precondition was unsupported; no production workaround, mode-default change, or vendor patch was made. This is not claimed as slash-command execution coverage.

## Integrated proof

[Current command ledger](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-final-command-ledger.md) and [validator matrix](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-integration-validator-matrix.json) retain commands, revisions, complete logs and exits. Runtime scope is pinned to `e5ed4121d`, containing the complete 42-register/sweep set; original RED/behavior receipts remain historical evidence, not fresh integrated acceptance.

Own gaps remain **0 → 0**; all later sweep gaps are unchanged, including 8.0.1's 17 existing gaps. No supersession closure or new preservation exception is justified. All **42 publication sweeps plus factory pass (43/43)** across **8,850 observations**. Negative control retains the single Logout direction mutation and produces exactly **0 → 1 gaps**, expected exit 1.

Integrated session cases pass **2 integration / 4 prefork**; key dispatch **29**, game-menu suites **24 integration / 14 prefork**, popup suites **40 integration / 33 prefork**, and Mists preservation **1** pass. [Untruncated caller scan](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-integration-session-callers.json) retains **109 matches**: src 10, tests 12, all-profile cache 87, bundled non-Wowless addons 0. No matching addon `run-tests` cases exist. These counts include context/documentation matches; they are not 109 distinct executed calls. Slash execution coverage remains bounded by the registration gap above.

All **42 registers** and **39/42 extracts** reproduce with recorded/inherited flags. The same 12.0.5/12.0.7 byte mismatches and 12.1.0 unsupported-template failure remain recorded. Shared receipt helper adds the real 8.0.1 register/sweep row (269 observations, 252 OK, 17 exact gaps). Generator/extractor/shared-validator fixtures pass **25/34/8**. Format and requested Mists tests check pass with **zero non-vendor warnings**; inherited iced manifest warnings are unsuppressed. Retail build passes; separate bounded startup prints **`[]`**. Every evidence validator passes **19/19**. No vendor edits, agents, push, merge or `__pycache__`.

## Original proof (historical)

[Evidence directory](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/) retains source/fetch receipt, RED discovery, complete scans, reproduction and scoped validators. Dedicated p732 target; long Cargo commands write logs asynchronously, without polling waits. No full suite, WoW asset tests, push, merge, model/agent subprocess or session-cwd mutation.

[Original proof manifest](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-proof.json) retains pre-integration receipt revisions, scopes and results. All **41 publication sweeps plus factory regression pass (42/42)** across **8,581 observations**. Cached 7.3.2 **2/2**, bare 7.3.2 **2/2**, key dispatch **29/29**, GameMenu bare **4/4** and cached **10/10**, Mists session preservation **1/1** pass. Generator/extractor/shared-validator fixtures **24/33/8** pass. Negative control changes only Logout `changed` → `removed`: exactly **0 → 1 gaps**. Format and requested Mists tests check pass with **zero non-vendor warnings**; six inherited iced manifest warnings remain unsuppressed.

All **41 registers** reproduce. **38/41 saved extracts** reproduce; the inherited 12.0.5/12.0.7 byte mismatches and 12.1.0 unsupported-description-template failure remain unchanged, not falsely reported green. All **209 original inputs and 86 old extraction-mode outcomes** are preserved. Every local evidence validator passes (**18/18**). Relocation proof accepts an out-of-scope future register, rejects preserved-ledger whitespace tampering and rejects a missing historical reproduction row. Original scope was recorded at `c5f05d05d`; integrated `validate.py` now scopes complete register/sweep sets with `historical_registers`/`historical_sweep_tests` at `e5ed4121d`, never a moving glob or receipt-defined subset. Receipt cwd/target descriptions are not equality gates. Historical revisions must remain reachable when integrating or the corresponding proof must be refreshed.

Source ledger has three bounded rows (both API occurrences and their compound prose statement) and four metadata rows; no publication gaps remain. Native-policy/lifecycle boundaries above remain deliberately unverified. No full-suite, standalone startup/CASC or reconstructed Legion acceptance is claimed.

## Sources

- [Pinned page provenance](../../../data/patch-api/sources/7.3.2-api-changes.provenance.json).
- [Source ledger](../../../data/patch-api/sources/7.3.2-page-coverage.json).
- [Evidence validator](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/validate.py).
- [Binding handoff procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-8-1-0-api-audit]] — source/proof template and later publication precedence.
- [[patch-audit-validator-portability]] — historical scope and exact preservation.
