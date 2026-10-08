# Patch 7.3.2 API audit

Verified source 2026-10-08: Warcraft Wiki page **230850**, revision **6200179** (2024-12-21T14:17:59Z), refetched rather than inherited from the remaining-pages inventory. Current retail is the target. [Spec](../../specs/patch-7-3-2-publication-sweep.md) defines the boundary.

## Source accounting

The complete page contains one Changes statement: **Logout and Quit lua functions are now protected**. Both API-linked occurrences are retained as `changed`, not invented additions/removals. The register has two identities; the extract has five identities (one compound behavioral statement, four source/heading/reference contexts). The ledger accounts for seven identities; no raw context is duplicated outside the extract.

Generator `--prose-api-links` is opt-in; existing inventory/bullet/rename flags do not capture Changes prose API links. Extractor reuses `--retain-reference-notes`, accepting capitalized `Ref web` only behind that flag. Prior default behavior remains unchanged. Linked Reddit discussion and reference list are retained but not expanded. Parallel 8.0.1's `--bfa-prepatch` is not duplicated. The later-register list starts with its one-line integration placeholder, then 8.1.0 and all master's later registers.

## Coverage matrix

| Contract | Implementation / observed evidence | Boundary |
|---|---|---|
| Publication | Two real session globals; initial cached sweep passes both | Presence is not protection |
| Insecure-call rejection | Initial bare and cached probes show both tainted calls succeeding and mutating state; retail guard now checks rilua active call-frame taint before mutation | Targeted GREEN verification pending |
| Secure transition/recovery | Existing `is_logged_in` and `simulator_exit_requested` state remain the action model; test asserts secure calls after caught failures | No native countdown/travel/process exit claim |
| Older profiles | Gate is retail-only; explicit Mists preservation test | Simulator preservation, not native historical parity |
| Native policy/lifecycle | Precisely recorded, not shimmed | Block-event/error wording, hardware-event entitlement, logout countdown/cancellation and historical/secret/restricted-context parity unverified |

## Root cause and modeled fix

Existing `session_exit.rs` published real actions but performed state mutations without checking insecure execution. Discovery proves stack taint was `Patch732Addon`; both Logout and Quit still succeeded, respectively clearing login state and requesting simulator exit. The failure occurs in the simulator action boundary, not Blizzard GameMenu Lua or the VM's ability to label the call.

Modeled globals move to `src/lua_api/globals/real/session_exit.rs`. A shared retail-only guard reads rilua's active `CallInfo.taint`, returns an explicit Lua error for insecure code, and runs before either state mutation. Secure Quit retains GUI-owned exit-request semantics; secure Logout retains the existing immediate login-state transition. ForceQuit/ForceLogout/QuitGame remain on their previous paths. The existing CancelLogout no-op is relocated to `workarounds/temporary/session_exit_defaults.rs`; it is unchanged, names the missing countdown/cancellation model, and receives no coverage credit.

Tests retain concrete state observations rather than merely checking function type or source text. They record taint separately from the call result so a failed taint assertion cannot masquerade as action protection. Cached probes run after unmodified real Blizzard UI startup. Secure calls following rejected insecure calls exercise error-boundary recovery in the same VM.

## Retirement decisions

**No retirements.** Source has no removals. Both changed members have live cached consumers and must remain published. [Complete scans](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/p732-removal-consumers.json) use `/usr/bin/grep -rnE` with whole-word `\b`, excluding `*Documentation*` files/directories. Qualified and bare scans coincide for these non-namespaced globals; complete `src/`/`tests/` outputs include indirect callers. Cached consumers: six Logout lines, three Quit lines. Neither master's later registers nor the recorded p801-page 8.0.1 register snapshot re-adds either symbol. No vendor files were edited.

## Recorded problematic boundaries

- **Native block notifications/error text:** page supplies no event name/payload or error wording, and no native capture exists. Explicit simulator failure is not native event parity.
- **Hardware-event policy:** no captured entitlement/gating rules; current stack taint proves only insecure-call rejection.
- **Logout countdown/cancellation:** existing state has no pending deadline/rest-area policy; immediate secure logout is preserved, not claimed as a countdown model.
- **Historical/secret/restricted execution:** retail target and lack of native captures cannot prove reconstructed 7.3.2 or full restricted-context policy.

## Proof

[Evidence directory](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/) retains source/fetch receipt, RED discovery, complete scans, reproduction and scoped validators. Dedicated p732 target; long Cargo commands write logs asynchronously, without polling waits. No full suite, WoW asset tests, push, merge, model/agent subprocess or session-cwd mutation.

At the implementation checkpoint: all 41 registers reproduce; old extraction outcomes and original inputs remain preserved; all seventeen existing evidence validators pass. Final GREEN gates and portable-validator acceptance remain pending. Counts are derived from files; `validate.py` scopes registers/sweep sources with `historical_registers`/`historical_sweep_tests` at recorded revision `c5f05d05d`, never a moving glob or receipt-defined subset. Receipt cwd/target descriptions are not equality gates.

## Sources

- [Pinned page provenance](../../../data/patch-api/sources/7.3.2-api-changes.provenance.json).
- [Source ledger](../../../data/patch-api/sources/7.3.2-page-coverage.json).
- [Evidence validator](../../../data/patch-api/evidence/7.3.2-session-2026-10-08/validate.py).
- [Binding handoff procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-8-1-0-api-audit]] — source/proof template and later publication precedence.
- [[patch-audit-validator-portability]] — historical scope and exact preservation.
