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

## What it must do

- [x] Preserve simulator frame identity in the forbidden partition: native parent arguments recognize the same frame while public and forbidden Lua tables remain distinct.
- [x] Convert direct references to XML-partitioned frames across the covered secure-environment boundary; ordinary frame fields remain public and nested data is not recursively rewritten.
- [x] Keep explicit native `GetObjectTable` results public for covered outbound addon initializers.
- [ ] Pass the mixed-argument XML delegate and unchanged native AuraContainer initializer regressions after scoped inbound argument projection; compilation/GREEN is deferred to the integrating parent.
- [x] Intern forbidden views and make repeated `GetForbiddenObjectTable` projection idempotent.
- [x] Run AuraContainer provider creation and inbound child ownership validation without publishing private methods or invoking public overrides of private methods.

## How it works

- `rilua` optional environment-transfer hook, pinned to `b6387563c8cd7882194cec368e626ebc72a17b81`.
- `src/lua_api/script_object_transfer.rs` registers a host conversion callback after secure-environment creation. Canonical registry mappings identify partitions, not user-controlled backlink fields alone.
- `src/lua_api/env_init/shared_bootstrap.lua` constructs and interns forbidden tables. Its XML delegate wrapper uses a registered native argument projector only for the explicit forbidden secure-delegate boundary.
- `src/lua_api/env_init/bootstrap.rs` registers that helper before either environment can use the wrapper and passes the compile-time capability into the bootstrap chunk; older profiles retain receiver-only XML behavior without a missing-helper fallback.

## Implementation inventory

- `src/lua_api/script_object_transfer.rs` — direct frame-reference conversion.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/mod.rs` — installation under the shared `forbidden-aspects` capability.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — parent return projection under the same capability.
- `src/lua_api/env_init/shared_bootstrap.lua` — partition identity and projection.
- `Cargo.toml`, `Cargo.lock` — published runtime dependency pin.

## Tests asserting this spec

- `tests/userdata_proxy.rs` — native parent acceptance, spoof rejection, real provider acquisition, initializer ownership, and private/public isolation.
- `tests/xml_secure_delegates.rs` — secure entry, addon callback taint, caller restoration after return/error, and receiver-only negative control. Compiled GREEN pending; native tainted AuraContainer creation reproduces the pre-fix table-security failure.
- Runtime environment-transfer tests cover arguments, varargs, multiple results, tail returns, native callbacks, reentry and error propagation.

## Known gaps

- [x] Focused retail proof at `8787273ad`: six `forbidden_partition_` cases cover interned projection/field isolation, native parent identity, spoof rejection, ordinary frame transfer, and real AuraContainer provider/initializer boundaries.
- [x] Three earlier-12.0.7 controls pass, preserving focused partition behavior outside 12.1.
- [ ] Forever compiled GREEN remains pending. Shared environment transfer resolves the earlier private `UpdateAuraDisplay` lookup boundary, but the existing initializer regression at `e137d9df6` rejects an ordinary child with `expected forbidden object reference`. Forever's native validator checks forbidden identity, unlike pinned Retail. Scoped XML argument projection addresses that boundary without changing public `GetObjectTable` or adding Rust aura methods. Missing `CustomAuraButtonUpdateMode` is a separate provider dependency; earlier Retail proof does not establish Forever GREEN.
- Coroutine yield/resume conversion and recursive conversion of table contents are not implemented by this facility.
- Conditional aura-secrecy access enforcement is separate from partition conversion and remains unmodeled.

## Out of scope

Vendor Lua changes, private-method stubs in the public table, or equating distinct Lua tables through a universal equality override.
