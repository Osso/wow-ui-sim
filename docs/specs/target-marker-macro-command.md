# Target-marker macro command (bounded Retail 12.0.5 slice)

`C_Macro.RunMacroText('/tm ...')` selects numeric marker commands against live simulator state. Source: [12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), line 98 (`prose-2026-03-25-098`). Runtime flow: [Lua API](../lua-api.md). This authoring slice is unexecuted; it does not establish full native macro compatibility.

## What it must do

- [ ] A selected `/tm [combat] 1` assigns marker 1 and delivers the existing `RAID_TARGET_UPDATE` notification without a Lua error; changing combat state changes selection.
- [ ] An unselected condition preserves unit icons and emits no marker notification. Existing comma conjunction, supported modifier predicates and first-matching semicolon clauses select commands from current state.
- [ ] Unconditional `/tm 2` assigns marker 2; `/tm 0` clears it. Selected `@focus`, `target=focus` and `unit=focus` selectors drive both predicate evaluation and the destination unit.
- [ ] **INFERRED simulator policy:** invalid, malformed, unselected or out-of-range decimal integer commands are atomic no-ops. A later valid command still succeeds. Destination absence follows the existing marker API's no-op policy; no synthetic unit is created.
- [ ] Preserve existing marker assignment/movement and synchronous notification semantics. Their equivalence to native repeated-assignment/toggle behavior remains unproven.

## How it works

- [Lua API](../lua-api.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/spell_macro_verbs.rs`: Retail-gated `/tm` command dispatch.
- `src/lua_api/globals/security/cmd_option.rs`: shared condition selection and unit-selector extraction; no second parser.
- `src/lua_api/globals/security/mod.rs`: crate-visible parser entry point.
- `src/lua_api/globals/targeting_verbs.rs`: existing GUID-keyed marker mutation/query and notifications; unchanged.

## Tests asserting this spec

`tests/mouse_tm_commands.rs` — one auto-included `integration` module, feature-gated for `retail-12-0-5`. Assertions cover marker queries, actual stored unit icons, queued events and synchronous Lua event callbacks.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 5 FAIL. GREEN: 5/5 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. Macro-runner path only; vendor slash handler compatibility not earned. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-098 bounded-coverage under capability `target-marker-macro-command`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Main must apply tests alone and record RED, then apply producers and record GREEN. Authoring task forbids test execution.
- [ ] Native permission/taint/secret-argument matrix and exact invalid-input behavior lack probe evidence.

## Out of scope

This bounded numeric slice does not cover native `!`/`~` prefixes, toggle parity, slash aliases, Lua `tonumber`'s broader coercion, adjacent-bracket alternatives, unsupported macro predicates, or a fully loaded chat slash-handler path. Existing parser predicates are reused, not certified wholesale (notably placeholder pet/vehicle existence and coarse group matching are not native proof). `SecureCmdOptionParse`'s missing second Lua return is not changed; the shared Rust selector supplies the destination to this command.

Row `prose-2026-03-25-099` is skipped, not counted covered: cached declarations contain `IsMouseOver` but no exact `IsUnderMouse` declaration. No invented alias or geometry policy is supplied.
