# Classic secrets authoring handoff

Verified source snapshot: 2026-10-03. **Authoring only; NOT applied, compiled, RED/GREEN tested, or accepted.**

## Active goal and boundaries
Audit prose-2026-04-17-213/214/215 against the current working tree; stage the smallest evidenced producer correction and real Classic-profile tests. Completion: source audit, unique exact edits, complete staged files, expected failures, profile commands, residual clause ledger. Exclusions: repository writes, dependency changes, Cargo/tests/git mutations, vendor changes, services, agents/models, operational execution. Only scratch staging and this handoff were written.

Current code differs from the task's older six-profile description: `ClientProfile` has **seven** variants, including WowForever. Cargo `client-retail` selects `profile-retail` + `retail-12-1-0`; PTR selects `retail-12-1-5`. Retail epoch features are rejected on the four official Classic markers by `src/client_profile.rs` compile errors. Proposal deliberately preserves Forever's existing separate secret/table-security behavior. No Forever-disablement or native 5.5.4 parity claim.

## Literal source rows
### prose-2026-04-17-213
> By and large, the addon security systems we put in place for Midnight will not be active in Classic. The secret value system is entirely disabled on Classic builds, and any API that returns secrets on Mainline will return non-secrets in Classic.

### prose-2026-04-17-214
> The new chat restrictions will also be inactive in Classic, and combat log events will work just like they do currently in Classic. Additional restrictions to various APIs added in Midnight (guild APIs, etc.) will also not be active in Classic. The existing pre-Midnight security systems (taint, restricted actions, etc.) will still be active of course. Brand new functionality and objects that were added in Midnight (duration objects, curves, etc. and their associated APIs) will be available in Classic.

### prose-2026-04-17-215
> One thing to note here is that Lua API documentation files do currently still reference secret values despite them not actually applying in Classic. We are working on fixing that, as well as doing an overall pass to make sure everything is working as expected in Classic. If you notice specific APIs that appear to not be working as expected please pass those along to us in the feedback channel.

Scout analysis consulted: `scout-prose-0331.md` sections 213/214/215 and `scout-remaining.md` corresponding rows. Both classify concrete Classic behavior as unmodeled, not narrative. Their older source line numbers do not substitute for this current-tree audit.

## 1. Secret production audit

“Classic” below means Wrath, Mists, Era, Anniversary. Compiled does not imply its wrapping branch is reachable. `unit_stats_restricted`, `cooldowns_restricted`, and `unit_auras_restricted` are explicit SimState inputs, default false; no automatic combat activation is inferred.

| Site / evidence | Compiled/registered in Classic | Actual activation / current result |
|---|---|---|
| `unit_misc.rs:54–80` identity predicate/output; `group_queries.rs:462–490` UnitName and UnitNameUnmodified | Yes, non-12.0.5 branches | Party names (and valid raid-name path in group_queries) are ordinary strings marked in `__sim_secret_values`. UnitFullName marks name and realm. **Classic issecretvalue can return true and canaccess* false today.** |
| `unit_misc.rs` UnitGUID | Yes | Non-retail branch hardcodes `secret=false`; GUID is ordinary. Mainline 12.0.5 wraps explicitly classified identity GUID/name strings. Classic GUID alone is not a RED reproduction. |
| `group_queries.rs:332–336` cached GetRaidRosterInfo name → mark_secret_value | Yes, all profiles | Cached names are marked unconditionally, including the local player row. **Classic secrecy bug independently of UnitName token heuristic.** |
| `security/secret_values.rs:72` mark_secret_value; `security/value_access.rs:23` __sim_mark_secret_value | Yes, all profiles | Registry producer has no profile guard. Direct marker helper plus text/attribute bootstrap calls can classify Classic strings secret. |
| `security/secret_values.rs` function_is_secret; `env_init/mod.rs` tainting_loadstring | Yes, all profiles | Insecure loadstring closure stamped `*** ForceTaint_Strong ***`; fallback shallow/deep checks equate that closure taint with secrecy. **Classic closure secret today; taint must remain while secret classification changes.** |
| `c_secrets.rs:67–79` push_stat_number; unit_stats, cooldown_probes, combat_stats, pet_stats and retail_inputs callers | Helper shared; individual real getters vary by profile/epoch | Wrapping compiled only with retail-12-0-5. Without epoch, helper pushes Val::Num even if unit_stats_restricted=true. Classic stat fixture expected already GREEN. C_Secrets.ShouldUnitStatsBeSecret is not registered in Classic. |
| `charge_state.rs:25` cooldowns_are_restricted; c_spell.rs:465, c_action_bar.rs:49, charge_state.rs:75, loss_of_control.rs:22 | Shared producer code compiled; API-specific registrations vary | Restriction predicate requires retail-12-0-5 AND Retail/PTR marker AND explicit flag. Always false on requested Classic builds. Numeric wrap branches compiled but unreachable from those outputs. Classic charge fixture expected already GREEN. |
| `c_action_bar_counts.rs`, `c_spell_counts.rs` number/string wraps; `c_spell_book.rs:409–424` cast count | Module/callback gated retail-12-0-5 | Absent Classic. unit_auras_restricted direct use is only in retail-gated GetSpellMaxCumulativeAuraApplications; setting the flag does not activate that producer in Classic. |
| `c_scenario_info.rs:76/83` number/string outputs; `real/unit_spell_target_name.rs:37` string output | Callback/registrations gated retail-12-0-5; target-name also Retail/PTR | Absent Classic. Scenario struct's existence is not a registered Classic producer. |
| `lua_duration_object/formatting.rs:70/121` host string/number; Cooldown countdown_formatter.rs:104 number | Modules/registration gated retail-12-0-5 | Absent Classic. Formatting cannot be enabled merely by dropping a registrar cfg: it depends on 12.0.5 abbreviated formatter modules. |
| `seconds_formatter.rs:165` host string and generic wrap_value | Shared constructor/private callbacks | Wrap branch needs actual VM-secret input. Official Classic has no Lua-native secretwrap registration; compatibility secretwrap returns its argument unchanged. Public ordinary inputs do not reach wrapping. Host-injected VM secrets can still propagate: excluded from this bounded producer correction. |
| `lua_duration_object/core.rs:100` generic wrap_secret | Shared core | Requires genuine secret timing/input slots; none of the audited official Classic API paths produce native wrappers. Host-injected wrappers can propagate, not declassified by this proposal. |
| `duration_text_binding.rs:343` generic wrap_secret | Module compiled, register exits before callbacks on Classic | Runtime ACTIVE/interval gate supports Forever and selected Retail/PTR epochs only. No Classic factory/callback activation. |
| `frame/methods/misc/secret.rs:164` wrap_host_secret_bool | Callback/register gated client-wowforever | Absent the four requested Classic profiles. This is why blindly treating every non-Retail profile as secret-disabled would break existing Forever policy. |
| `security/secret_values.rs` native secretwrap; `env_init/mod.rs:75–80` VM table-security registration | Retail native registration gated retail-12-1-0; VM registration gated Forever | No native secret wrapper global installed for official Classic by simulator. Debug compatibility passthrough secretwrap remains; no VM/dependency rewrite proposed. |

**Answer:** yes, Classic returns values classified secret today through string markers and tainted-loadstring closure classification. Audited stat/cooldown native wrapper branches are already inactive. The smallest fix must stop actual marker production, not lie in issecretvalue or weaken VM unwrap/security guards.

No new host wrapping facade is staged: redirecting every Retail-only wrapper through another facade adds churn without changing any current official Classic producer. ONE runtime profile predicate instead governs the two still-active secrecy producers. Mainline epoch gates and VM authenticity remain intact. A future shared native wrapper producer must consult the same capability before introduction; this is guidance, not claimed future-proof enforcement.

## 2. Duration/curve availability audit

| Registration | Current Classic availability / gate | Proposal |
|---|---|---|
| `register.rs:229` register_lua_duration_object; shared module/core | C_DurationUtil.CreateDuration, CreateManualClock, GetCurrentTime, lifecycle/timing/curve-evaluation core already registered | No change required. Behavioral test uses manual clock, real duration state, numeric/color curves and duration EvaluateElapsedDuration. |
| `env_init/mod.rs:62`, c_curve_util::register | CreateCurve and CreateColorCurve already registered all profiles | No change required. No speculative un-gating. |
| c_curve_util BOOLEAN_SELECTION_LUA | retail-12-0-0 gate; EvaluateColorFromBoolean / EvaluateColorValueFromBoolean absent | Residual associated-API gap; not tested or claimed. |
| C_Spell.GetSpellCooldownDuration / C_SpellBook.GetSpellBookItemCooldownDuration | retail-12-0-0 registration and callback gates | Residual gap. Isolated capability extraction could be a follow-up, not proven by constructor test. |
| C_Spell.GetSpellChargeDuration / C_SpellBook.GetSpellBookItemChargeDuration; C_UnitAuras.GetAuraDuration | retail-12-0-5 gates, including aura_duration module | Residual gap. Do not activate entire retail epoch on Classic: compile_error and unrelated patch behavior. |
| C_ActionBar.GetActionCooldownDuration / GetActionChargeDuration / GetActionLossOfControlCooldownDuration | Shared registration in action_bar_api/registration.rs, no epoch gate on the method list | Already published; not covered by staged constructor test. |
| Duration FormatElapsedDuration / FormatRemainingDuration / FormatTotalDuration; Cooldown SetCountdownFormatter / GetCountdownFormatter | retail-12-0-5 module and registry gates; formatting imports retail-only abbreviated_number_formatter | Residual gap. Removing a single cfg is insufficient without exposing 12.0.5 formatter behavior. No such refactor staged. |
| C_DurationUtil.CreateDurationTextBinding | Runtime ACTIVE/profile/interface gate exits for four Classic profiles | Residual gap; no gate change staged. |

Core objects/curves do not need registration changes. Full “and their associated APIs” remains incomplete. This audit is not evidence that all absent associated APIs are inseparable; it identifies which grouped formatting paths are unsafe to un-gate wholesale and leaves the rest unproven.

## 3. Proposed changes

| Tag | Staged mirrored path | Intent |
|---|---|---|
| state | src/client_profile.rs | `ClientProfile::uses_secret_values()` is false for Wrath/Mists/Era/Anniversary; true for Retail/PTR/Forever. Runtime ACTIVE usage, no new cfg walls. |
| producer | src/lua_api/globals/security/secret_values.rs | Guard marker allocation and closure secrecy classification. Do not clear closure/slot taint, alter protected-state guard, or hide authentic VM secrets. |
| producer-test | tests/security_api.rs | Seven existing tests now assert profile-specific secrecy/access while retaining old Mainline/Forever expectations. |
| state/producer-test | tests/classic_secret_policy.rs | Seven new Classic-gated public-behavior fixtures. build.rs automatically discovers the top-level file in the integration target; Cargo.toml need not change. |

No repo specification/wiki/coverage status edited or staged. This is a bounded proposal, not whole-row acceptance. No direct unit_misc edit needed: its Classic identity output calls the centrally guarded marker. No SimState restriction flag mutation required.

### Expected RED / existing-GREEN matrix (not executed)

| New test | Original current tree prediction | What its result would prove |
|---|---|---|
| identity_names_and_guid_are_ordinary | RED first non-secret UnitName assertion; GUID expected ordinary already | Exact seeded party name/realm/GUID payload, arity, concatenation, access and caller taint |
| raid_roster_name_is_ordinary | RED non-secret cached roster assertion | Separate unconditional roster marker producer |
| restricted_stat_input_keeps_plain_numbers | GREEN by current no-epoch push_stat_number path | Explicit true stat input, five exact numbers, addon arithmetic and taint |
| restricted_cooldown_input_keeps_plain_numbers | GREEN by existing runtime cfg predicate | Exact charge DTO under true cooldown flag; actual addon arithmetic and unchanged flag |
| duration_and_curve_constructors_work | Expected GREEN; constructor wiring exists; runtime success not observed | Concrete manual-clock duration, numeric and color curve interpolation, clock advance/copy |
| loadstring_remains_tainted_not_secret | RED non-secret closure assertion | Removing secrecy classification does not erase closure/global-slot taint |
| tainted_protected_action_still_denied | Expected GREEN by existing shared guard; runtime success not observed | Insecure in-combat protected width write dropped, exact ADDON_ACTION_BLOCKED event, secure write succeeds |

Existing expectation changes in tests/security_api.rs: test_secure_map_rejects_secret_keys_and_values; test_issecretvalue_tainted; test_canaccessvalue_tainted; test_canaccessallvalues_one_tainted; test_party_roster_name_is_secret_value; test_party_full_name_marks_name_and_realm_secret; test_table_containing_party_identity_is_not_accessible. Their Classic expectations were inconsistent with rows 213/215; Mainline/Forever expectations retained. Existing stat-restriction tests are gated via tests/character_stats.rs; cooldown_restriction tests gate themselves to 12.0.5 Retail/PTR. Do not un-gate those secret assertions for Classic.

### Parent-owned proof commands — NOT RUN
After applying staged files (not authorized in this handoff), run the new module once per official Classic profile:

```text
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-mists --test --test integration classic_secret_policy:: -- --nocapture
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-wrath --test --test integration classic_secret_policy:: -- --nocapture
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-era --test --test integration classic_secret_policy:: -- --nocapture
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-anniversary --test --test integration classic_secret_policy:: -- --nocapture
```

Changed existing tests under Mists and a Mainline 12.0.5 control build:

```text
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,client-mists --test --test integration security_api:: -- --nocapture
python3 scripts/build-host.py --build-host local --no-default-features --features sound,gui,casc,profile-retail,retail-12-0-5 --test --test integration security_api:: -- --nocapture
```

Matched Retail secret-output fixtures already exist: tests/character_stats/stat_restriction.rs seeds armor=1234; tests/cooldown_restriction.rs seeds the same spell19750 charge values (1/3/12/40/2) and asserts genuine wrapper metadata/payload; tests/security_api.rs seeds explicit party GUID classification on Mainline. Their previous proof is not reused as proof of this proposal; relevant filtered parent-owned runs and post-change Rust checks remain required. No Cargo feature need be added for the new tests.

## 4. Remaining clauses / proof level

| Row | Bounded authored coverage | Still unproven |
|---|---|---|
| 213 | Classic marker/closure source correction; representative name/GUID/stat/charge tests staged | No compiled execution; no all-API/all-Classic/native-5.5.4 proof; no blanket Midnight-security parity; host-injected authentic wrappers, unexamined value producers, frame secret/aspect state not disabled. Forever out of requested scope. |
| 214 | Shared pre-Midnight taint/protected-state guard unchanged; explicit taint denial/recovery test; existing duration/curve constructors tested in authored fixture | Chat restrictions and guild restrictions across entire API surface; combat log event payload/lifecycle compatibility; all restricted actions; all associated APIs, formatters and binding factories; native parity. Existing SendChatMessage appends logs and GuildInvite mutates roster without new guards, but source observation alone is NOT full chat/guild proof. No broad policy switch staged. |
| 215 | Same bounded secret-production fixture coverage as 213 | No whole-Classic runtime acceptance. Vendor documentation correction/feedback/planned sweep are upstream narrative, not repository runtime completion; vendor docs untouched. |

**Merging risk today:** uncompiled draft; minimal producer logic preserves Retail/PTR/Forever branches by inspection, but new fixtures and seven adjusted expectations have no execution evidence. Scope must stay bounded: this is not a complete secret-system disable switch or a completed row214 implementation.

## 5. Authoring proof ledger

- Read-only code/prose audit against current working tree; no git metadata used as current source truth.
- Every replacement anchor counted exactly once against original file; all source hashes rechecked immediately before handoff generation; staged files reconstructed from those exact replacements and compared byte-for-byte.
- `rustfmt --edition 2024 <scratch>/staging/classic-secrets/tests/classic_secret_policy.rs`: exit0, empty stdout/stderr; new-file syntax/formatting only, not type-check or behavior proof.
- Raw Lua-string boundaries inspected; select uses single-quoted '#'; no env.eval::<u32> authored (new tests use env.exec).
- Manual changed-line readability review: small exhaustive profile match, early producer guard, unchanged taint mechanism, concrete independent behavioral fixtures; no warning suppressions added. No automated lint/readability/type gates executed.
- No Cargo/tests/builds, git writes, repo writes, VM/dependency changes, agents/models or service mutations.

### Original file SHA-256 snapshots
- `src/client_profile.rs`: `d9458b2ed11563edfba4bce17a9bf8004a007791341a0387cedc8b5ae3d61b29`
- `src/lua_api/globals/security/secret_values.rs`: `f1b6e14741da88cc80d622d7b2c1973279658b8eda8b7d48fdfea67f9d60a87e`
- `tests/security_api.rs`: `dc61c18c21b33e001271250fc973cfee3e61f80be67d0067e71506bf1abd6fce`

## 6. Exact replacements against these snapshots

Full files are in `/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/staging/classic-secrets`. Do not overwrite changed parent work wholesale; apply these unique OLD→NEW replacements to the verified snapshot. NEW snippets contain full replacement code, no ellipses.

### Edit 1 [state] `src/client_profile.rs`

OLD (unique):
```rust
impl ClientProfile {
    /// Forever Classic identity
```

NEW:
```rust
impl ClientProfile {
    /// Disable Midnight secrets on the four official Classic profiles.
    /// Forever retains its separate existing compatibility policy.
    pub const fn uses_secret_values(self) -> bool {
        match self {
            Self::Retail | Self::Ptr | Self::WowForever => true,
            Self::Wrath | Self::Mists | Self::Era | Self::Anniversary => false,
        }
    }

    /// Forever Classic identity
```

### Edit 2 [producer] `src/lua_api/globals/security/secret_values.rs`

OLD (unique):
```rust
pub(crate) fn mark_secret_value(state: &mut LuaState, value: Val) {
    let Some(key)
```

NEW:
```rust
pub(crate) fn mark_secret_value(state: &mut LuaState, value: Val) {
    if !crate::client_profile::ACTIVE.uses_secret_values() {
        return;
    }
    let Some(key)
```

### Edit 3 [producer] `src/lua_api/globals/security/secret_values.rs`

OLD (unique):
```rust
    rilua::stdlib::taint::get_closure_taint(state, func_ref).as_deref()
        == Some(LOADSTRING_SECRET_TAINT_MARKER)
```

NEW:
```rust
    crate::client_profile::ACTIVE.uses_secret_values()
        && rilua::stdlib::taint::get_closure_taint(state, func_ref).as_deref()
            == Some(LOADSTRING_SECRET_TAINT_MARKER)
```

### Edit 4 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(result, "loadstring result should be secret");
```

NEW:
```rust
    assert_eq!(
        result,
        wow_ui_sim::client_profile::ACTIVE.uses_secret_values(),
        "loadstring secrecy follows the profile without removing closure taint"
    );
```

### Edit 5 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(!result, "loadstring result should not be accessible");
```

NEW:
```rust
    assert_eq!(
        result,
        !wow_ui_sim::client_profile::ACTIVE.uses_secret_values(),
        "loadstring secret access follows the profile"
    );
```

### Edit 6 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(!result, "mixed values should fail access check");
```

NEW:
```rust
    assert_eq!(
        result,
        !wow_ui_sim::client_profile::ACTIVE.uses_secret_values(),
        "mixed secret access follows the profile"
    );
```

### Edit 7 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(secret, "party roster identity should be secret");
    assert!(
        !accessible,
        "party roster identity should not be directly accessible"
    );
```

NEW:
```rust
    let expected_secret = wow_ui_sim::client_profile::ACTIVE.uses_secret_values();
    assert_eq!(secret, expected_secret, "party roster identity secrecy");
    assert_eq!(accessible, !expected_secret, "party roster identity access");
```

### Edit 8 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(name_secret, "party full-name identity should be secret");
    assert!(realm_secret, "party realm identity should be secret");
    assert!(
        !all_accessible,
        "secret full-name fields should block bulk access"
    );
```

NEW:
```rust
    let expected_secret = wow_ui_sim::client_profile::ACTIVE.uses_secret_values();
    assert_eq!(name_secret, expected_secret, "party full-name secrecy");
    assert_eq!(realm_secret, expected_secret, "party realm secrecy");
    assert_eq!(all_accessible, !expected_secret, "party bulk access");
```

### Edit 9 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(
        !accessible,
        "tables containing secret identities should be secret"
    );
```

NEW:
```rust
    assert_eq!(
        accessible,
        !wow_ui_sim::client_profile::ACTIVE.uses_secret_values(),
        "nested party identity access follows the profile"
    );
```

### Edit 10 [producer-test] `tests/security_api.rs`

OLD (unique):
```rust
    assert!(
        key_error.contains("attempted to store a secret key in a SecureMap"),
        "secret key should be rejected, got: {key_error}"
    );
    assert!(
        value_error.contains("attempted to store a secret value in a SecureMap"),
        "secret value should be rejected, got: {value_error}"
    );
```

NEW:
```rust
    if wow_ui_sim::client_profile::ACTIVE.uses_secret_values() {
        assert!(
            key_error.contains("attempted to store a secret key in a SecureMap"),
            "secret key should be rejected, got: {key_error}"
        );
        assert!(
            value_error.contains("attempted to store a secret value in a SecureMap"),
            "secret value should be rejected, got: {value_error}"
        );
    } else {
        assert_eq!(key_error, "", "Classic tainted function key is not secret");
        assert_eq!(value_error, "", "Classic tainted function value is not secret");
    }
```

## 7. New file [state/producer-test] `tests/classic_secret_policy.rs`

Create only when absent. Full code:

```rust
//! Bounded Classic secret disablement; no native-client or whole-row claim.
#![cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]

use wow_ui_sim::c_api::charge_state::SpellChargeState;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn classic_secret_policy_identity_names_and_guid_are_ordinary() {
    let env = WowLuaEnv::new().expect("Classic identity environment");
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    env.state().borrow_mut().party_members[0].name = "ClassicAlice".to_owned();
    env.exec(
        r#"
        local function checkIdentity()
            local name = UnitName('party1')
            local fullName, realm = UnitFullName('party1')
            local guid = UnitGUID('party1')
            assert(not issecretvalue(name), 'Classic UnitName must be ordinary')
            assert(not issecretvalue(fullName) and not issecretvalue(realm))
            assert(not issecretvalue(guid))
            assert(name == 'ClassicAlice' and fullName == 'ClassicAlice')
            assert(realm == 'SimRealm' and guid == 'Player-0000-00000002')
            assert(canaccessallvalues(name, fullName, realm, guid))
            assert(canaccesstable({name = name, guid = guid}))
            assert(name .. ':' .. guid == 'ClassicAlice:Player-0000-00000002')
            assert(select('#', UnitGUID('party1')) == 1)
        end
        checkIdentity()
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicSecretPolicyAddon')
            checkIdentity()
            assert(debug.getstacktaint() == 'ClassicSecretPolicyAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicSecretPolicyAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("ordinary payload, arity, nested access and unchanged caller taint");
}

#[test]
fn classic_secret_policy_raid_roster_name_is_ordinary() {
    let env = WowLuaEnv::new().expect("Classic raid environment");
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    env.state().borrow_mut().player.name = "ClassicRaidLeader".to_owned();
    env.exec(
        r#"
        local name = GetRaidRosterInfo(1)
        assert(not issecretvalue(name), 'Classic roster name must be ordinary')
        assert(name == 'ClassicRaidLeader')
        assert(canaccessvalue(name) and canaccesstable({name = name}))
        "#,
    )
    .expect("raid producer uses ordinary names in Classic");
}

#[test]
fn classic_secret_policy_restricted_stat_input_keeps_plain_numbers() {
    let env = WowLuaEnv::new().expect("Classic stat environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.stats.armor = 1234;
        state.unit_stats_restricted = true;
    }
    env.exec(
        r#"
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicStatAddon')
            local base, effective, total, positive, negative = UnitArmor('player')
            assert(select('#', UnitArmor('player')) == 5)
            assert(base == 1234 and effective == 1234 and total == 1234)
            assert(positive == 0 and negative == 0 and base + 1 == 1235)
            for _, value in ipairs({base, effective, total, positive, negative}) do
                assert(type(value) == 'number' and not issecretvalue(value))
                assert(canaccessvalue(value))
            end
            assert(debug.getstacktaint() == 'ClassicStatAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicStatAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("restricted host flag cannot make Classic UnitArmor outputs secret");
    assert!(env.state().borrow().unit_stats_restricted);
}

#[test]
fn classic_secret_policy_restricted_cooldown_input_keeps_plain_numbers() {
    let env = WowLuaEnv::new().expect("Classic cooldown environment");
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = true;
        state.spell_charges.insert(
            19750,
            SpellChargeState {
                current_charges: 1,
                max_charges: 3,
                recharge_start: 12.0,
                recharge_duration: 40.0,
                charge_mod_rate: 2.0,
            },
        );
    }
    env.exec(
        r#"
        local function addonCaller()
            local info = C_Spell.GetSpellCharges(19750)
            assert(type(info) == 'table')
            assert(select('#', C_Spell.GetSpellCharges(19750)) == 1)
            assert(info.currentCharges == 1 and info.maxCharges == 3)
            assert(info.cooldownStartTime == 12 and info.cooldownDuration == 40)
            assert(info.chargeModRate == 2 and info.cooldownDuration / info.chargeModRate == 20)
            for _, value in pairs(info) do
                assert(type(value) == 'number' and not issecretvalue(value))
            end
            assert(canaccesstable(info))
            assert(debug.getstacktaint() == 'ClassicCooldownAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicCooldownAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("restricted host flag cannot make Classic charge outputs secret");
    assert!(env.state().borrow().cooldowns_restricted);
}

#[test]
fn classic_secret_policy_duration_and_curve_constructors_work() {
    let env = WowLuaEnv::new().expect("Classic duration environment");
    env.exec(
        r#"
        local clock = C_DurationUtil.CreateManualClock(104)
        local duration = C_DurationUtil.CreateDuration()
        duration:SetClock(clock)
        duration:SetTimeFromStart(100, 10)
        assert(duration:GetTotalDuration() == 10)
        assert(duration:GetElapsedDuration() == 4)
        assert(duration:GetRemainingDuration() == 6)
        assert(not duration:HasSecretValues())
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 0)
        curve:AddPoint(10, 20)
        assert(curve:Evaluate(4) == 8)
        assert(duration:EvaluateElapsedDuration(curve) == 8)
        local colorCurve = C_CurveUtil.CreateColorCurve()
        colorCurve:AddPoint(0, CreateColor(0, 0, 0, 1))
        colorCurve:AddPoint(10, CreateColor(1, 0, 0, 1))
        local r, g, b, a = colorCurve:EvaluateUnpacked(5)
        assert(r == 0.5 and g == 0 and b == 0 and a == 1)
        clock:AdvanceTime(2)
        assert(duration:GetElapsedDuration() == 6)
        assert(duration:Copy():GetTotalDuration() == 10)
        "#,
    )
    .expect("existing Classic duration/curve registration has real public behavior");
}

#[test]
fn classic_secret_policy_loadstring_remains_tainted_not_secret() {
    let env = WowLuaEnv::new().expect("Classic closure environment");
    env.exec(
        r#"
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicLoadstringAddon')
            local fn = assert(loadstring('ClassicTaintedSlot = 17; return debug.getstacktaint()'))
            assert(not issecretvalue(fn), 'Classic tainted closure must not be secret')
            assert(canaccessvalue(fn) and canaccessallvalues(fn, 17))
            assert(canaccesstable({fn = fn}))
            assert(fn() ~= nil, 'loadstring must retain taint')
            local secure, owner = issecurevariable('ClassicTaintedSlot')
            assert(not secure and owner ~= nil, 'slot taint must remain')
            assert(debug.getstacktaint() == 'ClassicLoadstringAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicLoadstringAddon')
        addonCaller()
        assert(debug.getstacktaint() == nil)
        assert(ClassicTaintedSlot == 17)
        "#,
    )
    .expect("secret classification changes without erasing pre-Midnight taint");
}

#[test]
fn classic_secret_policy_tainted_protected_action_still_denied() {
    let env = WowLuaEnv::new().expect("Classic protected action environment");
    env.exec(
        r#"
        local blocked = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_BLOCKED')
        listener:SetScript('OnEvent', function(_, _, _, action)
            blocked[#blocked + 1] = action
        end)
        local protected = CreateFrame('Frame', 'ClassicSecretProtectedFrame', UIParent)
        protected:SetSize(40, 20)
        A_Admin.SetFrameProtected('ClassicSecretProtectedFrame', true)
        A_Admin.SetInCombat(true)
        local function addonCaller()
            assert(debug.getstacktaint() == 'ClassicProtectedAddon')
            protected:SetWidth(90)
            assert(protected:GetWidth(true) == 40)
            assert(debug.getstacktaint() == 'ClassicProtectedAddon')
        end
        debug.setobjecttaint(addonCaller, 'ClassicProtectedAddon')
        addonCaller()
        assert(#blocked == 1 and blocked[1] == 'ClassicSecretProtectedFrame:SetWidth()')
        assert(debug.getstacktaint() == nil)
        protected:SetWidth(70)
        assert(protected:GetWidth(true) == 70)
        assert(#blocked == 1)
        "#,
    )
    .expect("insecure combat write denied, exact blocked event, secure write allowed");
}
```
