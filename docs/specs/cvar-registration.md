# CVar registration

`RegisterCVar` and `C_CVar.RegisterCVar` register runtime defaults through `src/lua_api/globals/set_cvar_verb.rs`. See [Lua API architecture](../lua-api.md) for runtime context.

## What it must do

- [x] Preserve an explicit empty string as the default and current value of a previously unknown CVar, through global and namespace getters.
- [x] Keep omitted/nil defaults at the existing simulator value `"0"`, distinct from an explicit empty string.
- [x] Preserve the first registered default and existing overrides on re-registration.
- [x] Let ClassicCastBarForever's empty-value filtering retain its scale default `1`; continue accepting a stored nonempty scale `1.25`.

Cached Forever `CVarDocumentation.lua` declares a nullable string `value`. Wowless `wowless/modules/cvars.lua` retains the supplied value, including an empty string. These support the distinction but are not native-client probe evidence. Omitted/nil `"0"` and re-registration behavior remain existing simulator policy, not claims derived from Wowless.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/set_cvar_verb.rs`: shared global/namespace registration and name validation.
- `src/cvars.rs`: runtime defaults, existing-value preservation and override storage; unchanged by this fix.

## Tests asserting this spec

`tests/set_cvar_global.rs`:

- `register_cvar_preserves_explicit_empty_default_lifecycle`: both registration surfaces, current/default reads, re-registration and overrides.
- `register_cvar_nil_default_retains_zero_policy` and existing `register_cvar_makes_unknown_cvar_visible_with_zero_default`: nil/omitted defaults.
- `empty_cvar_registration_preserves_classic_castbar_scale_default`: fresh unknown CVar through the addon's read/register/filter/scale-selection sequence and strict frame scale setter.

Development proof: `/tmp/forever-addon-audit/empty-cvar-development-ledger.json`. Frozen pre-fix runtime fails both empty-default regressions and passes nil-default preservation. At `d1e2487f6`, grouped `set_cvar_global` tests pass 15/15, including all three new tests. Independent acceptance and unchanged-addon replay remain pending.

## Known gaps (current cycle)

- [ ] Obtain independent verification and unchanged-addon startup/workflow in isolated data storage.

## Out of scope

Native-client parity, new persistence rules, namespace relocation and scale-validation changes. No addon/vendor modification; full ClassicCastBar compatibility requires separate runtime evidence.
