# Action loss-of-control cooldown info

Retail 12.0.5 exact plaintext row **239**, `api-C_ActionBar-GetActionLossOfControlCooldownInfo-239`, requires a meaningful action-slot snapshot before its output restriction can receive credit. Current action callback in `src/lua_api/globals/action_bar_api.rs` ignores its argument and fabricates an inactive five-field record. Existing typed inputs already suffice. See [audit SSOT](../wiki/investigations/patch-12-0-5-api-audit.md). This commit adds tests/spec only; no producer implementation or executed RED.

## What it must do

### Modeled snapshot

- [ ] Resolve existing `action_bars` slot → spell ID → `spell_loss_of_control` typed record. Slot17 →19750 returns `(312,237,1.25,true,true)`; slot19 →642 independently returns `(11,27,0.5,true,false)`. These are data fixtures, not native LoC observations.
- [ ] Return exactly one fresh ordinary public table with exactly `startTime`, `duration`, `modRate`, `isActive`, `shouldReplaceNormalCooldown`. Preserve f64 timing and convert f32 rate to f64; fixture rates are exactly representable, not evidence of arbitrary native precision.
- [ ] **Inferred absence policy:** valid unassigned/unknown positive slot and assigned spell without a record return `(0,0,1,false,false)`, preserving current action shape; do not borrow spell getter's nil-on-miss policy.
- [ ] **Inferred snapshot/flag-copy policy:** read host interval/flag replacements, mapping reassignment and clear immediately. Copy flags verbatim, including zero intervals with active/replacement flags. Do not calculate expiry, compare normal cooldowns, or derive active status.
- [ ] Reads and DTO mutation/replacement leave host maps and other DTOs unchanged; environments remain isolated. For an assigned unrestricted spell, existing real `C_Spell.GetSpellLossOfControlCooldownInfo` matches all five concrete action fields without changing spell API secrecy.

### Output security

- [ ] Apply existing `charge_state::cooldowns_are_restricted` predicate to three actual host-secret numeric values, including default zero timing/rate1. When false, same numbers are public. Do not substitute combat/stat policy or fake secrecy metadata.
- [ ] **Inferred table-versus-field/default restriction policy:** table stays public/accessibly ordinary; two copied flags remain ordinary booleans under restriction. Do not make booleans secret or declassify numeric wrappers.
- [ ] Secure and public-tainted callers preserve context. Tainted arithmetic on each restricted numeric field fails through existing VM authorization with nonempty errors that do not disclose payloads.
- [ ] Tainted copies retain actual numeric wrappers; host-only identity/allocation observations survive failures, secure recovery and forced GC. Turning restriction off produces a fresh public DTO while old secret fields remain opaque to addons.

### Slot authentication and validation

- [ ] Authenticate the documented argument position through native VM `unwrap_secret` **before** parsing or lookup. Public tainted assigned slots remain allowed; secure authentic secret NUM17/19 select the concrete records. Tainted secrets fail before known, unknown or missing-record lookup and preserve caller context; secure recovery remains usable.
- [ ] **Inferred strict domain:** accept finite integral positive u32 slots, including unknown positive slots; reject missing/nil, BOOL, STRING, table/frame, nonfinite/fractional, zero/negative and out-of-range arguments. Secure authenticated wrong types must fail, not select a default. Native `RequiresValidActionSlot` enforcement is unknown; the previous stub accepted anything.
- [ ] Invalid public/secure-secret numeric inputs yield nonempty contextual errors naming public API and argument1; error text does not expose private string/secret payloads. Input authentication and numeric output restriction are separate tests, not extra annotation-row credit.
- [ ] Publish one epoch125 retail/PTR C API handler in the real namespace; inverse-gate existing legacy handler for Forever/earlier profiles. Do not swap callbacks in tests, add generic declassification, or invent activation/catalog inputs.

## How it works

- [Lua API architecture](../lua-api.md)
- [Client profile system](../wiki/systems/client-profiles.md)
- [Audit classification/accounting](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/lua_api/state/support_types.rs`: existing `LossOfControlInfo` fields `start_time:f64`, `duration:f64`, `mod_rate:f32`, `is_active:bool`, `should_replace_normal_cooldown:bool`; no new host type required.
- Existing `SimState.action_bars` and `spell_loss_of_control`: assigned-slot and spell-keyed explicit inputs; no new state fields planned.
- `src/lua_api/globals/action_bar_api.rs`: current fabricated inactive action callback; retained inverse legacy handler planned, not implemented here.
- `src/lua_api/globals/action_bar_api/registration.rs`: current `COOLDOWN_SLOT_METHODS` publication; epoch125 real C API registration/inverse legacy gate planned.
- `src/c_api/c_spell.rs`: existing real spell-level five-field DTO and nil-on-miss behavior, unchanged.
- `src/c_api/charge_state.rs`: existing restriction predicate and authentic VM secret-number construction; no charge entry/selection work.
- Planned epoch125 action LoC handler belongs in `src/c_api/`; no producer file created in this task.

## Tests asserting this spec

- `tests/action_loss_control_cooldown_info.rs`: 22 focused behavioral cases, grouped autodiscovery; cfg `all(retail-12-0-5, any(profile-retail,client-ptr))`. No new Cargo target, private fixture imports, fake API producer or callback replacement.
- `tests/c_spell_flyout_probes.rs`: existing lower-level spell DTO controls, unchanged; not action-path proof.
- `tests/action_cooldown_output_restriction.rs`: existing assertion style only, unchanged; row233 cannot supply row239 credit.

## Known gaps (current cycle)

- [ ] Main compiled RED, producer GREEN and independent acceptance pending. No build/test/check/lint/readability/ops/push or delegation performed in this tests/spec task. Formatting alone is not execution proof.
- [ ] Exact239 remains pending. Accounting remains **197 pending /150 bounded /14 partial /1 metadata =362 IDs,69 capabilities**; no accounting artifact modified. Rows295 and233 remain separately pending.
- [ ] `/tmp/patch-12.0.5-action-loss-control-provider-map.md` mislabeled row237 and retained old199/148/68 counts. Correct scope is239; plaintext237 is `GetActionDisplayCount`, not this function. No row237 bundle or credit.
- [ ] Native LoC conditions, spell/action mapping, secrecy behavior, valid-slot domain and automatic signals remain unknown. Absence, strict input domain, table/field restriction, restricted defaults and verbatim flag-copy are chosen inferences, not native exception facts.
- [ ] Deferred non-gating native probe: actual LoC spell/action slot; capture three numeric/two BOOL fields across empty/inactive/active/replacement states; secure/tainted read-copy and secret observations under cooldown restriction. No native probe required for this bounded cycle.

### Source grounding

Cached profile files under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `ActionBarFrameDocumentation.lua`, full `GetActionLossOfControlCooldownInfo` function: `RequiresValidActionSlot=true`, `SecretWhenCooldownsRestricted=true`, `SecretArguments="AllowedWhenUntainted"`; one required `actionID:luaIndex`; non-nil `SpellLossOfControlInfo` return.
- `SpellSharedDocumentation.lua:33–44`: three non-nil NUM fields without `NeverSecret`, two non-nil BOOL fields marked `NeverSecret=true`. Descriptions associate activity with timing and replacement with normal cooldowns; explicit host-fixture flags deliberately remain snapshots, not native-derived conditions.
- Exact239 registration delta: `SecretWhenActionCooldownRestricted` → `SecretWhenCooldownsRestricted`. Metadata is not proof of modeled LoC selection or exception behavior.

## Out of scope

Rows237/231/233/295; duration APIs, legacy LoC pair, `GetActionCharges`, generic cooldown/GCD/charge selection; new host state/catalog/activation/expiry/events; full-profile/UI compatibility; native parity or native gate. Protected `aura_duration.rs` body inspection and edits are forbidden in this task. No source/Cargo/model changes or spell-secrecy credit.
