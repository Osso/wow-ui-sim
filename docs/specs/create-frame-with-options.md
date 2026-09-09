# CreateFrameWithOptions

PTR `CreateFrameWithOptions(options)` constructs a frame from a structured argument. Its declared fields and profile introduction come from the pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json). Implementation lives in `src/lua_api/globals/create_frame/`; lifecycle architecture is documented in [addon loading](../addon-loading-pipeline.md).

## What it must do

### Declared surface

- [x] Publish the constructor only with `retail-12-1-5`; keep earlier retail absent. `CreateFrameOptions` describes an argument, not a Lua global table.
- [x] Require `frameType`; accept optional `name`, frame `parent`, ordered string-array `inherits`, and numeric `id`; return the created frame.
- [x] Default `hidden` and `forbidden` to false and preserve explicit booleans. Reuse positional construction without changing its argument behavior.

### Simulator assumptions — not native lifecycle evidence

- [x] Nil/omitted parent leaves the frame unparented; valid names, parent substitution, IDs and intrinsic widget types reuse existing semantics.
- [x] Apply inheritance entries in array order; later template mixins override earlier mixins.
- [x] Apply options flags before template handlers. Options visibility overrides template hidden attributes before OnLoad, including explicit false.
- [x] Run template OnLoad before initial OnShow. Hidden frames and frames under hidden parents do not receive an initial OnShow; later Show uses normal visibility dispatch.
- [ ] `forbidden=true` sets the frame flag; false does not clear restrictions inherited from its parent or existing loading scope. This is flag state, not security enforcement.

### Simulator validation policy — not native coercion/error evidence

- [x] Reject malformed options before registering a frame: non-table input, missing/invalid frame type, non-string name, invalid parent, non-boolean flags, nonfinite/fractional/out-of-range i32 ID, and non-dense or non-string inheritance arrays.
- [x] Reject embedded commas in inheritance entries rather than silently reinterpret an array element as several templates.
- [ ] Reject empty template names and ignore unknown option keys; these validation branches lack focused assertions.

## How it works

- [Frame creation and loading](../addon-loading-pipeline.md)
- [Template inheritance](../xml-template-system.md)
- [Widget state](../widget-system.md)

## Implementation inventory

- `src/lua_api/globals/create_frame/options.rs` — PTR argument decoding and validation.
- `src/lua_api/globals/create_frame/mod.rs` — shared allocation adapter, options flag state and initial show dispatch.
- `src/lua_api/globals/create_frame/template_chain.rs` — options-only flag reapplication before template OnLoad.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_create_frame_options.rs` — constructor identity, ordered templates, flags/lifecycle, malformed inputs, and retail positional regression.

## Known gaps (current cycle)

- [ ] Native validation, nil-parent defaults and lifecycle ordering require a real PTR probe; tests establish the simulator choices above only.
- [ ] Reentrant creation/Show/Hide inside OnLoad, template child event ordering, and handler-error recovery are not established by this slice.

## Out of scope

- `AllowedWhenUntainted`, secret arguments, protected calls and forbidden-aspect enforcement: no native/security proof is provided by this adapter.
- Native coercion or exact error strings, duplicate names, fractional IDs and inheritance edge precedence: generated type declarations do not specify them.
- Changes to positional CreateFrame lifecycle or unrelated XML loading behavior.
