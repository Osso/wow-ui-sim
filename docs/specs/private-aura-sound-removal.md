# Private aura sound removal

Batch49 covers only exact row `global api-C_UnitAuras-RemovePrivateAuraAppliedSound-403` (`- HasRestrictions`) in the [12.0.5 source register](../../data/patch-api/sources/12.0.5-register.json). C API-owned host inputs live in `src/c_api/private_aura_sounds.rs`; fixtures describe a bounded removal transition. This commit publishes neither removal method and contains no producer body. Architecture reference: [C API boundary](../wiki/systems/lua-api.md).

## Evidence and inference

The source row removes restrictions from the legacy remover. March 31 source prose (line168) separately retains conditional restrictions on Add; this slice does not change Add. Later cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua` names `RemoveAuraSound`, declares a required numeric `auraSoundID`, no returns, and `SecretArguments = "AllowedWhenUntainted"`. This is cached later-client evidence, **not native 12.0.5 verification**.

Cached Retail `Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua` gates on `GetCVarBool("loadDeprecationFallbacks")` and assigns `C_UnitAuras.RemovePrivateAuraAppliedSound = C_UnitAuras.RemoveAuraSound`. The simulator [CVar defaults](../../src/cvars.yaml) set that CVar to `1`. Executing this actual file after bootstrap can erase legacy-only publication. Thus later producer publication must expose legacy under `retail-12-0-5` and modern under `retail-12-1-0`, using the same live ID set. Modern publication is required solely for alias durability, not an expanded audio feature.

The representation (`HashSet<u32>`), inclusive ID domain, missing/repeated-ID behavior, strict validation and conservative secret rejection are **INFERRED simulator policies**, not native semantics. Zero-result removal is supported by later cached documentation but remains unverified for native 12.0.5. Native secret acceptance/access permissions, native error wording, lifecycle and side effects are unknown. No native probe gate applies to this approved bounded model.

## What it must do

### Inputs and transition — INFERRED

- [ ] Start each environment with an empty host-declared live ID set; never fabricate acquired IDs.
- [ ] Accept only an actual public finite integral `u32` NUMBER, including `0` and `4294967295`; no string coercion or spell-alias lookup.
- [ ] Remove an existing host-seeded ID immediately and return exactly zero Lua values; preserve all other IDs.
- [ ] Treat unknown and repeatedly removed valid IDs as zero-result no-ops.
- [ ] Observe host replacement immediately; isolate environments and preserve caller tables, spell aliases, cooldown associations and classification flags.
- [ ] Reject missing, nil, STRING (including `'101'`), bool, plain table, actual FrameTable, function, thread, NaN, infinities, negative, fractional and out-of-range inputs with contextual public errors, no ID mutation and successful subsequent public recovery.

### Security — bounded policy, not native permission parity

- [ ] Ordinary public tainted calls remove live IDs, preserve caller taint and require no caller/combat gate, reflecting the literal restriction-removal row.
- [ ] **INFERRED:** Authenticate secret arguments through actual VM security before type/key access; reject them even in secure context without unwrapping or declassifying their payloads.
- [ ] **INFERRED:** Rooted actual secret NUMBERs (known/unknown), STRINGs (numeric hit/unknown), and a wrapped real frame remain secret and identity-stable through GC and rejection in secure and stamped tainted closures; errors remain public, state unchanged, taint preserved and public recovery works.

### Publication durability

- [ ] Later producer publishes legacy at `retail-12-0-5`; modern exists only from `retail-12-1-0`. Both remove from the same ID set with zero results.
- [ ] Under `retail-12-1-0`, executing the complete actual unmodified cached Shared deprecated file after bootstrap with default CVar `1` leaves legacy removal functional, including actual public tainted removal and host-observed state transition.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/private_aura_sounds.rs` — default-empty public host input struct and live ID set only; no register/provider/body.
- `src/c_api/mod.rs` — public module gated by `retail-12-0-5`.
- `src/lua_api/state/sim_state.rs` — per-environment gated field beside classification/cooldown inputs.
- `src/lua_api/state.rs` — gated empty default initializer.

## Tests asserting this spec

`tests/private_aura_sound_removal.rs`: 13 grouped fixtures (11 at `retail-12-0-5`, two additional publication fixtures at `retail-12-1-0`), discovered by the existing generated integration harness; no new Cargo target.

The publication fixture reads `$HOME/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua` in full and calls `WowLuaEnv::exec_named` with the actual path as chunk name after `WowLuaEnv::new()` bootstrap. Missing cache fails explicitly. No copied alias snippet, synthetic vendor function, namespace substitution, API replacement or fallback is used.

Prerequisites asserted before loading: default `GetCVarBool('loadDeprecationFallbacks')`, real `C_UnitAuras` and `C_SpecializationInfo` tables, existing `Enum.UnitAuraSoundTrigger.Added`, and existing `Enum.CustomAuraButtonDispelTypeTextureStyle.BorderWithIcon`/`PreserveAsset`. The other vendor functions are only defined, not invoked; they need no Add or paper-doll implementation. Its final inspect-specialization assignment may read nil without preventing this chunk's removal alias. Fixture frame arguments use actual `CreateFrame('Frame')` behavior and table representation, never a userdata-shape guess.

No tests/build/check/gates were run at this inputs stage. Expected preproducer failures are absent generic namespace methods; **compiled RED is parent-owned and not yet established**. All requirements remain unchecked.

## Known gaps (current cycle)

- [ ] Parent compiled RED before any removal producer or API publication.
- [ ] Later minimal producer and parent-owned GREEN/independent verification; this input commit does not establish passing behavior or row acceptance.
- [ ] Add acquisition remains unmodeled; host-seeded removal does not establish Add-to-Remove lifecycle.

## Out of scope

Add providers, audio/playback, catalog, listeners, counters, sound records, native encounter matrices, native secret permissions/error parity, CVar0 coverage, other API rows, coverage accounting and full-page acceptance. None are needed for this host-seeded remover or required later alias durability.
