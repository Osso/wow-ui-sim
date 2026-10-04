# Retail 12.0.7 host events and identity defaults — B39–B44

Bounded host-input slice integrated on `p1207-r6`; targeted default-profile RED/GREEN proven, historical/native parity unproved. The [retained source](../../data/patch-api/sources/12.0.7-api-changes.txt) removes chat-lockdown secrecy from nine chat events, restores three profiling APIs, adds a URL texture event, changes unsupported identity defaults, and mentions a strata secret bugfix. Later cached declarations are contract context, not authenticated historical/native proof.

| Rows | Proposed input / output boundary | Status |
|---|---|---|
| events 149–157 | Empty host chat queue → ordinary event listeners; only nine named lockdown exemptions | Bounded projection; full payload/native source classification unproved |
| prose 012, globals 052–054 | Independent zero event/function time-count and script usage snapshots | INFERRED no-argument totals following later cache; no automatic timing |
| event 146 | Empty host URL notification queue → actual texture plus result enum | Bounded host notification only; request lifecycle blocked |
| prose 008 | Empty explicit unsupported-token set → GUID nil, health/power zero, legacy aura nil | Core six entry points only; whole prose row remains partial |
| prose 020 | SetFrameStrata bugfix | BLOCKED: cached NotAllowed conflicts with guessed secret permission |

## What it must do

- [ ] Gate all proposed state, publication, producers and changed behavior on `retail-12-0-7`; older profiles retain existing behavior, not a modern fallback.
- [ ] Chat ingress is empty per environment. Publishing consumes one message before synchronous listeners, retains host tuple length/order and roots authentic VM values through nested calls and GC. INFERRED queue consumption/error policy.
- [ ] The nine named events do not gain secrecy merely because lockdown is active. Independent source-secret arguments remain secret. `CHAT_MSG_SAY` remains the restricted control; cached `NeverSecret` positions stay public and reject independently secret host inputs. No blanket declassification.
- [ ] Host chat classification reads lockdown live at delivery. Addon handler/caller taint is preserved; secret results remain opaque to tainted code without poisoning equal public literals.
- [ ] Profiling queries return live independent snapshot values: event and function queries return exactly time/count pairs; script query returns one scalar. Zero defaults, units, attribution and ordinary-extra ignoring are INFERRED simulator policy. INFERRED NeverSecret rejects authentic secrets in every argument/extra for clean and tainted callers.
- [ ] Host URL notifications carry exactly the registered texture identity and cached enum values Found=1, NotFound=2, Requested=3, NotAllowed=4. Dispatch is synchronous, once per queued record, isolated per environment. INFERRED host-notification lifecycle; not proof of SetURLTexture requests.
- [ ] Explicit unsupported tokens override live modeled target/player data only for UnitGUID, UnitHealth, UnitHealthMax, UnitPower, UnitPowerMax and legacy UnitAura. Defaults are one nil for GUID/aura and one zero for each vital. Empty set changes no supported token. Exact unsupported set/default classification is INFERRED, not an automatic PvP classifier.
- [ ] GUID/vitals authenticate ALL arguments and extras with VM `unwrap_secret` before validating any or returning unsupported defaults; clean callers may pass authentic secrets, tainted callers may not. Legacy UnitAura uses the same policy, explicitly INFERRED because no cached declaration was located. Malformed unit tokens still error.

## How it works

- [Event dispatch](../event-system.md)
- [Lua API](../lua-api.md)
- [Instance identity](instanced-identity.md)

## Implementation inventory

- `src/lua_api/host_chat_inputs.rs`, `src/lua_api/host_chat_events.rs` — queued host input and rooted normal-listener delivery.
- `src/lua_api/performance_inputs.rs`, `src/lua_api/globals/real/performance_inputs.rs` — independent snapshots and Rust getters.
- `src/c_api/url_texture_inputs.rs`, `src/lua_api/host_url_texture_events.rs` — C_Texture-owned host notification records and normal event delivery.
- `src/lua_api/unsupported_unit_inputs.rs`, `src/lua_api/globals/unit_misc.rs`, `src/lua_api/globals/utility_system_spell/spell_api.rs`, `src/lua_api/globals/auras.rs` — explicit core-six defaults and authenticated selectors.
- `src/lua_api/state{,/sim_state}.rs`, `src/lua_api/{mod,env_init/mod}.rs`, `src/c_api/mod.rs`, `src/lua_api/globals/real/mod.rs`, `src/lua_api/workarounds/temporary/performance_metric_defaults.rs` — gated wiring and removal of modern profiling constant stubs.

## Tests asserting this spec

- `tests/patch_1207_b39_b44.rs` — public Lua queries/listeners, concrete host mutations, environments, real VM secrets and addon taint. Corrected inputs-only RED: 1 passed / 16 behavioral failures. GREEN: 17 passed. Fixture names its frame explicitly so invalid-receiver checks reach the producer.
- Existing profiling-default unit test expects new arities under 12.0.7 and passes the targeted default-profile run.

## Known gaps (current cycle)

- [ ] Targeted default-profile compilation and RED/GREEN are proven. Full cached-consumer startup, alternate epochs and historical/native parity remain unproved; startup CLI and alternate features were excluded from integration.
- [ ] Chat fixtures are a four-value primitive projection, not the complete later-cache 18-value chat schema with DiscordChatInfo. Native ingress, all chat entry paths, callbacks and host-source secrecy are not authenticated. NeverSecret validation samples one forbidden position.
- [ ] No automatic profiling counters, keyed function/frame identity, ResetCPUUsage integration, strict-epoch signature proof or native addon availability proof.
- [ ] B42 request lifecycle is blocked: SetURLTexture has unspecified HasRestrictions, actual cached consumers depend on Requested and identity matching, and no native completion/error/cancellation behavior is available. No fake Cancelled result, fallback texture, success without pixels, or constant completion.
- [ ] B43 whole-row inventory remains open: remaining C_UnitAuras and health/power APIs, exact unsupported PvP token set, historical return tuples and secret outputs are not established by core-six fixtures. Existing later-cache secret-return annotations are not implemented by these default changes.
- [ ] B44 remains blocked: later cache explicitly says NotAllowed and protected. Source bugfix does not prove accepting secret strata strings. Need historical 12.0.7 declaration/native clean-versus-tainted authentic-secret matrix, malformed tokens, protected combat denial, and child propagation before any edit.

## Out of scope

- Changing coverage ledgers or altering repo/cache vendor sources.
- Network URL downloads, texture rendering, synthetic timers, native profiling measurement/reset and automatic PvP state acquisition: no established contract or producer.
- Wrapped constants, fallback compatibility, shims and vendor patches: forbidden by the task.
