# Patch 7.3.2 publication and session protection

Audit Warcraft Wiki page 230850, revision 6200179, refetched 2026-10-08. Current retail is the target, not a reconstructed Legion client. [Pinned source](../../data/patch-api/sources/7.3.2-api-changes.wikitext) has one Changes statement: Logout and Quit became protected. [Audit](../wiki/investigations/patch-7-3-2-api-audit.md) describes implementation and proof.

## What it must do

- [ ] Retain both changed API-link occurrences with their literal annotation, plus every extract identity. No source additions/removals are invented.
- [ ] Both globals remain published after unmodified cached full-UI startup. Publication alone does not prove protection.
- [ ] Addon-tainted Logout/Quit fail before changing login/exit state, in bare and cached UI environments.
- [ ] Secure calls, including calls after a blocked insecure call, retain existing transitions: Logout clears login state; Quit requests GUI-owned exit, never exits the test process.
- [ ] Mists simulator behavior remains unchanged; no historical native parity claim.
- [ ] Evidence pins source, logs, command revisions, exact gaps, negative control, prior preservation and reproduction. Historical register scope is revision-pinned; counts derive from files; no absolute-path equality gates.

## How it works

- [Audit and model boundary](../wiki/investigations/patch-7-3-2-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `src/lua_api/globals/real/session_exit.rs` — existing session state transitions, retail stack-taint gate.
- `src/lua_api/globals/register.rs`, `real/mod.rs`, `globals/mod.rs`, `admin.rs` — registration and Admin query wiring.
- `src/lua_api/workarounds/temporary/session_exit_defaults.rs`, `temporary/mod.rs` — relocate existing CancelLogout no-op without changing behavior; not protection coverage.
- `tools/gen_patch_wikitext_register.py` — opt-in Changes prose API-link occurrence capture.
- `tools/extract_patch_non_inventory.py` — existing opt-in reference retention accepts template case.
- `data/patch-api/evidence/7.3.2-session-2026-10-08/` — immutable proof receipts, reproduction, accounting, portable read-only validator.

## Tests asserting this spec

- `tests/patch_7_3_2_publication_sweep.rs` — register-driven cached publication/absence with later-register precedence.
- `tests/patch_7_3_2_session_protection.rs` — tainted-call rejection and state nonmutation; secure transition/error recovery; cached UI probe.
- `tests/patch_7_3_2_classic_session.rs` — Mists simulator preservation.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — opt-in parser contracts.
- Evidence `validate.py` and `test_validator_portability.py` — occurrence accounting, receipts, preservation and scope/tamper proof.

## Known gaps (current cycle)

- [ ] Targeted verification and evidence acceptance pending.

## Out of scope

Native notification/error wording, hardware-event policy and logout countdown/cancellation lack source contracts or native captures. No historical 7.3.2 signature/security parity claim. ForceLogout/ForceQuit/QuitGame are not named by this page and remain unchanged. Reference list/Reddit citation remain unexpanded. No full-suite/CASC texture acceptance, vendor mutation, Blizzard Lua patch or shim-based protection closure.
