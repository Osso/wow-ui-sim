# Secret-origin aura display handoffs

## Scope and evidence

Forever native `Blizzard_AuraButton.lua` feeds wrapped timing values into duration objects. `Blizzard_CustomAuraButton.lua` passes those objects to cooldown/text/bar displays and calls `SetShown(secretwrap(isAuraShown))`. Rilua's authenticated wrapper decoder rejects tainted callers. This slice preserves that check; it does not change generic `FromStack` coercion.

The user cannot run Forever probes and permits informed guesses when identified as guesses. **Simulator policy, not native-verified semantics:** direct shown/text/timing readouts derived from secret inputs reject tainted callers. Untainted native processing and Rust rendering retain access. This does not claim complete native secrecy or side-channel isolation.

## Required behavior

- [ ] `SetShown` decodes authenticated wrapped values before applying Lua truthiness; wrapped false/nil hide. Plain truthiness remains unchanged, including ordinary userdata.
- [ ] `SetText` accepts authenticated wrapped text without exposing secret-origin metadata as Lua fields. Failed wrapped writes leave existing content unchanged. Unsupported userdata is rejected on Forever's text input path.
- [ ] Cooldown and status-bar duration consumers retain the duration's secret origin. Tainted consumers reject secret duration input before mutating widgets.
- [ ] Tainted `IsShown`, effective `IsVisible` (including secret-origin ancestors), `GetText`/`GetTextData`, cooldown numeric readouts, and status-bar timer/value readouts reject secret-origin state.
- [ ] Plain complete replacement clears the corresponding origin flag; changing only cooldown duration preserves origin because the existing start time survives. Plain input behavior and other profiles remain unchanged.

## Representation and boundaries

`Frame.secret_shown`, `secret_text`, and `secret_timing` are Rust booleans derived only from accepted inputs. They are independent of `explicit_secret_aspects`; existing declarations do not activate these guards globally. Button text propagation preserves the text flag on its FontString, and button `GetText` checks the child's origin too.

Cooldown numeric state remains Rust-owned. StatusBar's existing Lua duration reference remains unchanged; it points to the duration model, whose own secret-state access policy must protect its timing values. This slice neither implements a new status-bar clock nor adds plaintext timing fields to Lua.

Native text binding must send secret-origin formatted strings through authenticated `SetText`; formatting policy and duration storage belong to their own implementations. No generic userdata unwrapping, arbitrary formatter authorization, color-secret expansion, protected-frame redesign, or universal explicit-aspect enforcement is included.

## Geometry and texture handoffs — explicit simulator guesses

Native `Blizzard_CustomAuraContainer.lua:796–800` wraps all anchor arguments and both dimensions; `Blizzard_AuraContainerUtil.lua:291` wraps aura icons. Actual runtime RED is `/tmp/ellesmere-forever/aura-layout-texture-secret-red.stderr`: wrapped anchors fail string conversion, wrapped dimensions become zero, and wrapped texture assignment/clearing silently retain the old icon.

- [ ] Specific `SetPoint` decoding preserves overloads and original stack roots without modifying argument slots; authentication and parsing finish before anchor mutation. Every provided argument is decoded, including wrapped frame references and nil.
- [ ] Size setters decode authenticated inputs and retain independent width/height origin flags, including unchanged-value replacements. Plain `SetWidth` does not clear secret height state.
- [ ] Anchor origin is retained per point, so replacing/clearing one point does not declassify another. Tainted geometry readouts conservatively reject secret origins reachable through parent or relative-anchor dependencies; cycle-safe traversal includes size, rect, edge, center and anchor queries.
- [ ] `SetTexture` accepts authenticated IDs, paths and nil, retaining a private origin flag. Direct texture source getters reject tainted readouts. Plain `SetTexture` replacement and color-texture source clearing remove that origin; partial atlas/source changes conservatively retain it.

These readout restrictions and replacement rules are **guesses**, not native-verified semantics or general layout/visual side-channel isolation. Rust layout/rendering is unchanged and untainted native queries remain available. No VM or generic `FromStack` change is needed.

## Proof

`tests/aura_secret_display.rs` covers wrapped false, ancestor visibility, authenticated text, tainted input atomicity, unsupported text userdata, duration-to-widget handoffs, restricted direct readouts, and plain replacement. Tests were written before production edits; compiled RED/GREEN is deferred to the integrating parent. Existing native duration-input RED and `CustomAuraButton` wrapped-boolean consumer establish the integration boundary, not every new policy assertion.

Related: [duration core](duration-core.md), [Forever table security](forever-table-security.md), [script-object environments](script-object-environments.md).
