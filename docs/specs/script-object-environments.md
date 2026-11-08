# Script-object environment crossings

Retail 12.1+ and Forever secure Blizzard functions use a distinct object-table partition for frame-private state. Public addon code must not obtain private mixin methods by overriding similarly named public fields. [XML mixin bindings](../xml-template-system.md) and the native AuraContainer frame provider exercise this boundary.

## Bounded non-secret identity and field-isolation policy

`PrivateIdentity` and `InaccessiblePublicKeys` can describe simulator partition behavior without claiming native private-object identity or security. The selected policy is:

- Public and private projections are distinct Lua values for one underlying simulator frame; repeated private projection is interned and idempotent.
- Private fields remain independent between frames and from same-named public assignments. Private XML key values/mixin methods are absent publicly unless explicitly exported or delegated; this is partition separation, not a universal inaccessible-key list.
- Native frame methods and parent arguments accept the private projection as the same underlying frame. Ordinary tables cannot impersonate that identity.
- Actual AuraContainer provider callbacks retain their public view; public overrides do not replace private implementations.
- Under `forbidden-aspects`, XML mixins with `secureDelegates="true"` and `inboundPartition="forbidden"` project direct native-frame arguments, including ordinary Cooldown, Texture and FontString children, into canonical forbidden views. They invoke the native delegate through `securecallfunction`: caller taint is suspended during the call and restored on return/error; addon callback closure taint remains effective. Other arguments retain identity, position and nil slots; nested tables are not traversed. Ordinary secure calls and other XML delegate modes retain their existing argument behavior.

Focused proof must cover projection identity, field isolation, native parent/method behavior, spoof rejection, and the unmodified provider boundary. Caller authority, secret accessibility, hooks, handler-storage isolation, and native security remain unverified.

## CreateFrame parent-view selection

Native evidence WM70, retail 12.1.5 build 70688 (partial):

- **VERIFIED:** One native object has distinct cached public and forbidden Lua tables. Raw table identity distinguishes them; native lookup resolves either view to the same object.
- **VERIFIED:** Creation classifies the supplied parent by raw table identity and publishes the new object's explicitly selected cached view. The cache selector does not consult the caller environment.
- **INFERRED:** Ordinary `CreateFrame` results follow the supplied parent view; with no selected parent view, the ordinary view is returned. This is not established for every template or context.
- **VERIFIED consumer evidence, source build unverified:** Secure frame pools retain the exact acquired table as a key and require that same key on release; their checked Lua pool and proxy paths do not invoke `GetForbiddenObjectTable`.
- **UNKNOWN:** The guarded `GetForbiddenObjectTable` bridge, arbitrary secure-boundary conversion, and general equality semantics are not established. Do not infer universal equality or pool-key canonicalization.

## GetParent and function boundaries

Native evidence WM73, retail 12.1.5 build 70688 (partial):

- **VERIFIED:** `GetParent` classifies the receiver's cached view and publishes the parent in that view, without using the calling environment to select it.
- **INFERRED:** Normally completed calls on a default child return the default parent; calls on a forbidden child return the forbidden parent. Repeated forbidden-view ancestor traversal retains that view.
- **VERIFIED:** The inspected `GetChildren`, `GetRegions`, and `GetPoint` relative-to output fragments use the default-only publisher, not `GetParent`'s view selection.
- **VERIFIED:** The checked Lua and C argument, vararg, and result transfers copy table identity without selecting another cached view. **INFERRED:** Ordinary environment crossings do not canonicalize object tables.
- **EMULATED:** The simulator's explicit XML delegate projection remains a scoped compatibility policy; WM73 does not establish its native mechanism. **UNKNOWN:** Getter guards and all engine callbacks were not fully traced.

## What it must do

- [x] Preserve simulator frame identity in the forbidden partition: native parent arguments recognize the same frame while public and forbidden Lua tables remain distinct.
- [ ] Return the new frame's parent-selected table view from `CreateFrame`, so an ordinary secure frame pool retains the same key when its consumer later releases it. TargetFrame consumer verification is pending.
- [ ] Preserve the supplied frame-table identity across ordinary Lua environment crossings. Explicit XML delegates and native publication points retain their separate projection behavior; untraced engine callbacks remain UNKNOWN.
- [ ] Publish `GetParent` in the receiver's view, independently of the calling environment; do not apply this selection to `GetChildren`, `GetRegions`, or `GetPoint`.
- [x] Keep explicit native `GetObjectTable` results public for covered outbound addon initializers.
- [x] Publish inbound delegate frame results in the public object view for container receivers whose pinned vendor passes document it (`Blizzard_CustomAuraContainer.lua:437,444,505,512`), while AuraButton display-element getters keep returning the adopted private elements.
- [x] Pass the mixed-argument XML delegate and unchanged native AuraContainer initializer regressions after scoped inbound argument projection.
- [x] Intern forbidden views and make repeated `GetForbiddenObjectTable` projection idempotent.
- [ ] Engine proxy tables and bound-method adapters do not inherit the caller that creates or reads them. Invocation still preserves the active caller and underlying method's own addon taint.
- [x] Run AuraContainer provider creation and inbound child ownership validation without publishing private methods or invoking public overrides of private methods.
- [x] Expose XML parentKey children to private frame mixins under forbidden-object-table scope while keeping ordinary public Lua fields isolated.

## How it works

- The former simulator environment-transfer hook projected direct frame values on ordinary secure/global Lua crossings. This was an emulated partition policy, not a verified native rule. It changed pool object identities on the TargetFrame consumer path; ordinary crossings now retain their supplied values.
- `src/lua_api/script_object_transfer.rs` registers the explicit projection constructor after secure-environment creation. Canonical registry mappings identify partitions, not user-controlled backlink fields alone.
- `src/lua_api/env_init/shared_bootstrap.lua` constructs and interns forbidden tables. Its XML delegate wrapper uses a registered native argument projector only for the explicit forbidden secure-delegate boundary, and publishes frame results of container inbound calls in the public view; `AuraButton` receivers keep the forbidden result view because their display-element getters return adopted private elements.
- `src/lua_api/env_init/bootstrap.rs` registers that helper before either environment can use the wrapper and passes the compile-time capability into the bootstrap chunk; older profiles retain receiver-only XML behavior without a missing-helper fallback.

## Implementation inventory

- `src/lua_api/script_object_transfer.rs` — direct frame-reference conversion.
- `src/lua_api/methods.rs` — parentKey publication to public and scoped forbidden frame views.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/mod.rs` — installation under the shared `forbidden-aspects` capability.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — parent return projection under the same capability.
- `src/lua_api/env_init/shared_bootstrap.lua` — partition identity and projection.
- `Cargo.toml`, `Cargo.lock` — published runtime dependency pin.

## Tests asserting this spec

- `tests/userdata_proxy.rs` — native parent acceptance, spoof rejection, real provider acquisition, initializer ownership, private/public isolation, and method-origin preservation across addon-created projections.
- `tests/click_targeting/forever_regressions.rs` — full startup, target-player aura tick, private Count access, and ordinary public-field isolation.
- `tests/xml_secure_delegates.rs` — secure entry, addon callback taint, caller restoration after return/error, and receiver-only negative control. At `9476efcf5`, both cases pass.
- Runtime environment-transfer tests cover arguments, varargs, multiple results, tail returns, native callbacks, reentry and error propagation.

## Known gaps

- [x] Focused retail proof at `8787273ad`: six `forbidden_partition_` cases cover interned projection/field isolation, native parent identity, spoof rejection, ordinary frame transfer, and real AuraContainer provider/initializer boundaries.
- [x] Three earlier-12.0.7 controls pass, preserving focused partition behavior outside 12.1.
- [x] At `9476efcf5`, `secure-chain-tests-ledger.json` records the native AuraContainer initializer/partition group 8/8, `forever_forbidden_consumers` 3/3, and `xml_secure_delegates` 2/2. This closes the scoped transfer/delegate proof, not full GUI aura acceptance.
- [x] Bounded integration after the separate secret-number ordering pin: 2/2 original-boundary regressions pass with zero Lua errors — full-startup target-player tick and PlayerFrame `PLAYER_ENTERING_WORLD` → ComboFrame `3 → 0` with CVar setup before `OnLoad` (`/tmp/wow-unit-frame-bug/aura-combo-green-20260925.log`). Final local GUI evidence records zero hook/final errors, Combo CVar `1` reporting `0`, and Aura ID `1` hover displaying `Arcane Intellect`; see [[final-unit-frame-click-aura-proof]]. This does not claim exact original screenshot reproduction or native-client conformance.
- [ ] Full GUI aura acceptance beyond the bounded hover/error observations remains outside this proof.
- Coroutine yield/resume conversion and recursive conversion of table contents are not implemented by this facility.
- Conditional aura-secrecy access enforcement is separate from partition conversion and remains unmodeled.

## Out of scope

Vendor Lua changes, private-method stubs in the public table, or equating distinct Lua tables through a universal equality override.
