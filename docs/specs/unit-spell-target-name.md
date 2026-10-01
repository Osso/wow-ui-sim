# UnitSpellTargetName cast-target snapshot

Bounded Mainline 12.0.5+ contract for `UnitSpellTargetName`, using an explicit optional resolved target on `CastingState` in `src/lua_api/game_data.rs`. This slice registers a modeled query backed by that snapshot, with grouped actual-query fixtures. Producer `5beaf7545` follows actual RED; saved build `024afed64` passes ten target fixtures and 36 flyout/vehicle controls. Independent bounded behavior/security acceptance passes; default fmt/check gates cover snapshot `4c5aeb2d8`, excluding later party tests `03ebe6972`. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

Criteria below apply to the modeled player caster with explicit snapshot input. Saved ten-case GREEN and independent bounded acceptance cover these criteria, not native or full API compatibility.

- [x] Use only the current actual cast's explicit target snapshot (GUID, name, player classification), never the selected `current_target` or a deferred unit-token lookup.
- [x] Return nil for no cast, no explicit cast target, or an NPC cast target. Retained [12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt): “The UnitSpellTargetName API now only returns names for player units.”
- [x] Return exactly one secret string containing the explicit player recipient's name. Cached retail `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:3056–3070` declares `SecretReturns = true`, `SecretArguments = "AllowedWhenUntainted"`, one required unit token, and one `targetName` string return.
- [x] Preserve the cast snapshot when selected target changes or disappears. Removing or replacing the cast must remove its previous target; a replacement without explicit target returns nil.
- [x] Public unit tokens from tainted callers resolve to opaque secret output without changing caller taint. Secret unit tokens are accepted untainted and rejected tainted through existing `unwrap_secret` policy; output inspection must never clear taint to bypass that policy.
- [x] Return exactly one explicit nil for absent output. **Inference:** cached documentation says “Returns nil if the unit is not casting a spell or the spell has no target.” Its single-return declaration has no `MayReturnNothing`, but marks `targetName` non-nilable; this is not native arity evidence.
- [x] Read actual casting only, not channeling. **Bounded inference:** cached docs say “currently casting a spell” and do not explicitly demonstrate channel support. Channel-only fixture input must not fabricate an actual cast target.
- [x] Existing cast/channel constructors leave target absent. Fixture input does not implement actual user targeting or change existing target selection/effects.

## How it works

- [Lua API](../wiki/systems/lua-api.md)
- [Existing channel lifecycle](channel-empower-lifecycles.md)

## Implementation inventory

- `src/lua_api/globals/real/unit_spell_target_name.rs`: strict UTF-8 string token validation after existing `unwrap_secret` AllowedWhenUntainted validation; exact `player` caster reads only `casting.target`, filters player recipients, and immediately roots the host secret string result. Other valid tokens return one nil; no generic coercion or taint clearing.
- `src/lua_api/globals/{real/mod.rs,register.rs}`: publication requires `retail-12-0-5` and Mainline (`profile-retail` or `client-ptr`); no pre-12.0.5 or Forever publication.
- `src/lua_api/game_data.rs`: `CastTargetSnapshot` and optional `CastingState.target` input.
- `src/lua_api/state.rs`: public type re-export for external grouped fixtures.
- `src/lua_api/globals/{combat_verbs.rs,admin.rs,missing_surface/profession_crafting.rs}`: existing actual-cast constructors use `target: None`.
- `src/c_api/c_spec.rs`: specialization-change cast starts without target.
- `src/lua_api/channeling/inputs.rs`: shared channel/empower input constructor starts without target.
- `tests/{c_spell_flyout_probes.rs,c_vehicle_possession_globals.rs}`: existing two cast and three channel literals gain absent target.
- `tests/unit_spell_target_name.rs`: ten grouped actual-query fixtures, gated by `retail-12-0-5` plus `profile-retail` or `client-ptr`; discovered by the existing integration harness, no new Cargo target.

## Tests asserting this spec

`tests/unit_spell_target_name.rs` covers idle/untargeted/NPC nil values and arity, exact secret player name, selection independence, cast replacement/removal, channel-only absence, public-token tainted output, and both caller classes for secret unit tokens. Secret globals are rooted before allocation/GC; trusted payload inspection runs only after tainted frames return and asserts secure state first.

**Proof ledger:** input/tests revision `c140765105c3986b240fd4ac4827df2c441d6630` compiled successfully with `cargo test --test integration --no-run --message-format=json` (exit 0; `/tmp/patch-12.0.5-batch13-red-build.json`, `.log`, and `-result.json`). Actual selected runtime command `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c unit_spell_target_name:: --nocapture --test-threads=1` failed 0/10 (exit 101; `/tmp/patch-12.0.5-batch13-red-run.log` and `.json`), all at missing query boundaries. The secret-input rejection fixture explicitly requires callable query registration before its rejection assertions. Producer `5beaf7545` was compiled at `024afed646e3acf6d514b2eb6d29bb8951c70e9a`: saved `cargo test --test integration --no-run --message-format=json` exits 0 (`/tmp/patch-12.0.5-batch13-green-build-result.json` and build log). `/tmp/patch-12.0.5-batch13-green-runs.json` and referenced run logs record actual-query GREEN 10/10, flyout controls 14/14 and vehicle/possession controls 22/22, all exit 0 on that revision. Saved startup `timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors` exits 0 with `[]` (`/tmp/patch-12.0.5-batch13-green-startup-run.json` and `-green-startup.json`); this is saved executable proof, not proof for a later overwritten binary. These are bounded development results, not native compatibility or whole-page completion. Independent `/tmp/patch-12.0.5-cast-target-independent-proof.md` confirms bounded behavior/security PASS using saved GREEN, with default `cargo fmt --check` and `cargo check` exit 0 (zero warnings/errors) at snapshot `4c5aeb2d825ce4851c0185bbce7f5e613552ab81`: all 3,220 tracked Rust/config input hashes match before/after. Later party tests `03ebe6972` are excluded; no current-worktree blanket acceptance. Ten RED cases stopped at the missing query surface, not ten independently demonstrated behavioral failures. Readability's 32-physical-line body finding is advisory, not a required refactor or failed behavior gate. Verifier did not certify startup; saved startup above remains separate revision-scoped evidence.

## Known gaps (current cycle)

- [x] Independent bounded behavior/security acceptance and snapshot-scoped default fmt/check gates above; later party tests excluded.
- [ ] Native probes for nil arity, caller/output security, channel behavior, and targeted-cast transitions; none executed here.

## Out of scope

- Non-player casters: simulator has player-only casting state; no other-unit cast model is added. Player-only caster support is a simulator limitation, not a claim about the native API's unit-token coverage.
- Actual user targeting: target snapshot is explicit host/fixture input only. Existing producers never derive it from selection, self-target heuristics, or spell metadata.
- `SpellTargetUnit`, new cast lifecycle and target-class/display APIs.
- Pre-12.0.5 and Forever publication: this slice establishes no query behavior for those profiles.
