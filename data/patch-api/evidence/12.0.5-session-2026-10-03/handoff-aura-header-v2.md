# Aura-header v2 handoff

Authoring complete; candidates are not compiled, runtime-verified, or accepted. No repository/vendor writes, cargo commands, git mutations, agents, or model CLIs were used.

`SCRATCH` = `/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad`.

Full code mirrors repository paths under `$SCRATCH/staging/aura-header-v2/`. Exact integration artifacts:

- `against-aa29d7d7f.patch`: complete patch against requested baseline, including new files and deletion.
- `existing-file-edits.json`: 17 unique, non-overlapping OLD/NEW replacements against aa29d7d7f; new-file tags and checksum-guarded deletion.
- `proof-ledger.json`: source/candidate checksums, existing-binary results, and verification limits.

Use exact edits, not whole existing-file snapshots: another worker owns concurrent tree changes. Every existing OLD anchor still matched once at final inspection. Live HEAD during existing-test runs was `ce418cbe46049c28ed7798c67a929a0dbb6e4354`, not aa29d7d7f. Existing-binary results are not proof that the staged candidate or that HEAD was compiled.

## 1. Aura root cause: addon-folder deduplication

Rows 03-25-112 / 03-31-151 remain tests-only candidates; no comparator change or acceptance credit is proposed.

The parked fixture did show the header. Its explicit second load was the failure:

1. `load_game_ui()` had already loaded `Blizzard_RestrictedAddOnEnvironment`.
2. Its cached retail TOC excludes `SecureAuraHeader.lua/xml` with `[AllowLoadGameType classic]`.
3. `load_addon_from_toc()` derives identity from `toc.addon_dir.file_name()`, not `## Title` (`src/loader/addon.rs`, `load_addon_internal`). It returns an empty successful LoadResult when that folder is already loaded.
4. The fixture changed Title but reused the same directory. Neither aura file loaded. Empty warnings did not establish successful loading. Header creation consequently had no actual aura-template scripts, so Show created no aura children.

This explains the recorded empty order without blaming the comparator, iteration, RegisterUnitEvent, or attribute primitives. Cached `SecureAuraHeader.lua:460–466` iterates GetUnitAuras and maps zero expiration to `math.huge`. `src/lua_api/globals/auras.rs::get_unit_auras` reads the seeded player buffs; nil maxAuraCount does not truncate them to zero.

**Corrected fixture:** append the two unannotated file entries to the real parsed TOC before its first load. No vendor file is changed:

```rust
fn load_restricted_environment_with_aura_header(env: &WowLuaEnv, toc: &std::path::Path) {
    let source = std::fs::read_to_string(toc).expect("read restricted environment TOC");
    let source = format!("{source}\nSecureAuraHeader.lua\nSecureAuraHeader.xml\n");
    let probe_toc = TocFile::parse(toc.parent().expect("TOC directory"), &source);
    let loaded = load_addon_from_toc(&env.loader_env(), &probe_toc)
        .expect("load actual secure aura files with restricted environment");
    assert!(loaded.lua_files > 0 && loaded.xml_files > 0);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}
```

The discovery loop invokes this only for the restricted-environment addon in aura fixtures, instead of the usual first load. The test also requires the actual Update function, an OnShow handler, initially hidden template, visible shown header, and UNIT_AURA registration. Probe errors are no longer cleared after construction; only unrelated preceding whole-UI startup errors are isolated.

`tests/secure_group_headers.rs` succeeds because it loads the restricted environment once, with SecureGroupHeaders already included for retail. Existing binary control: **2/2 PASS**, exit 0, 7.06 seconds.

No simulator aura-sort producer fix is justified. Corrected aura tests are expected GREEN, but were not executed.

## 2. Helper API and smallest supported lifecycle

Source rows 03-25-120 / 03-31-177 say helper functions no longer taint talent frame/player castbar. They do not specify loadout cast duration or universal asynchronous completion.

Cached `ClassTalentHelper.lua` explicitly says these utilities require the Class Talent Frame so the UI can manage change flows. Four registered command callbacks lazy-load PlayerSpells and invoke real specialization/loadout methods. Cached ClassTalents documentation declares the same four CallbackEvents. Direct simulator mutation bypasses those UI writes; an unchanged clean UI would be vacuous tests-only credit.

**Specialization:** real ActivateSpecByIndex/Name → content OnActivateClicked → C_SpecializationInfo.SetSpecialization. Existing `src/c_api/c_spec.rs::start_specialization_change` already creates the activation cast and pending_spec_change without changing active state. GUI completion already clears the cast, emits STOP/SUCCEEDED, then applies pending specialization. However, `apply_spec_change` previously changed only player index: talent/spec/config/hero state stayed stale.

Candidate moves the existing completion implementation to `src/lua_api/cast_completion.rs`, exposes `tick_casting`, and routes GUI ticking through that same function. Existing casting unit-test files remain in place and are referenced with path attributes. Delete old `src/iced_app/casting.rs` and its module declaration: there is no duplicate lifecycle or fallback.

At retail-12-0-5 completion, call `talents.switch_to_spec(spec_id)` together with player-index mutation before specialization notifications. Retain existing PLAYER_SPECIALIZATION_CHANGED, then publish no-payload ACTIVE_PLAYER_SPECIALIZATION_CHANGED. Cached UnitDocumentation declares that event; ClassTalentsFrame consumes it to refresh loadout options/tree state. Cast STOP/SUCCEEDED ordering is retained from existing simulator behavior; ordering of the added refresh after the unit notification is simulator policy, not proven native chronology. Earlier-epoch runtime completion behavior is preserved by cfg.

**Loadout:** real LoadConfigByName/Index → LoadConfigInternal(autoApply=true) → existing LoadConfig. Vendor distinguishes NoChangesNecessary, Ready, and LoadInProgress. Only LoadInProgress starts the ongoing commit UI. Current simulator instead mutates seeded loadout immediately, publishes ACTIVE_COMBAT_CONFIG_CHANGED, and returns Ready. This packet retains that bounded existing model and tests its actual callback/state/event completion; it does NOT relabel Ready as LoadInProgress or invent a commit cast.

A native delayed loadout commit/castbar lifecycle remains unmodeled and uncredited. Declaring every loadout asynchronous is not supported by these sources. Specialization completion is fully authored; universal loadout commit semantics are not claimed.

## 3. State / producer / tests

**STATE:** no new fields or second selection/sort model. Reuse player.pending_spec_change, casting, existing talents/config state, and player.buffs.

**PRODUCER:** `src/c_api/class_talent_commands.rs` preserves the parked four secure command-event delegates. Epoch-selected registration replaces retail direct SwitchTo* mutation. Existing secure-stack guard restores caller taint on callback success/error; tainted addon observer closures remain tainted. Ordinary primitive arguments only are tested; secret/combat argument parity is not credited.

**PRODUCER integration:** existing-file replacements affect `src/c_api/mod.rs`, the class_talents registration/old-handler cfgs, `src/lua_api/mod.rs`, `src/iced_app/mod.rs`, and `src/iced_app/update.rs`. New shared completion file contains the moved helpers plus retail completion fix.

**TEST / DOC:** full revised six-test module at `tests/secure_aura_header_helpers.rs`; staged contract at `docs/specs/secure-aura-header-helper-delegation.md`. New test file begins with required retail cfg. No u32 env.eval result, forbidden raw-string interior, or nonexistent EventQueue method is introduced.

**Round-B99 dependency:** none required for this slice's new state/schema. Candidates and exact edits originate from aa29d7d7f, not concurrent snapshots. Current dispatcher additionally contains B99 aura-entry bookkeeping; this packet neither changes nor relies on it. Preserve unrelated B99 edits when integrating. Do not reapply original parked snapshot blindly.

## 4. Existing tests: old versus new meaning

| Existing test | Old meaning | New meaning |
|---|---|---|
| hero_talents::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state | Bare environment; four commands immediately mutate state | Retail calls real cached helpers. Protection named loadout 202 → Holy cast completion/default 101 → Holy indexed loadout 102 → Retribution cast completion/default 301. Retains config names, last-selected IDs and hero-subtree assertions. Asserts unchanged pre-deadline state, actual cast identity/events, coherent completed observer state and no duplicate completion. Earlier-epoch test remains unchanged. |
| admin_spec_talent_api::test_trait_config_mapping_tracks_active_loadout | Uses UI command as bare-env setup | Uses existing LoadConfig provider to establish another config; retains every mapping assertion. |
| hero_talents::test_non_selectable_hero_nodes_do_not_show_selectable_glow | Protection selected through an immediate UI command | Seeds coherent Protection player/talent state before loading UI; glow assertions unchanged. |
| hero_talents rendering: test_class_talent_edges_render_below_visible_talent_buttons, test_button_frame_level_change_relevels_connected_edges_on_update, test_hero_spec_content_spec_image_anchors_to_spec_name | Same UI-command setup | Same pre-load coherent fixture; rendering assertions unchanged. |

The existing hero test now calls a shared assertion function in the revised helper test module; its guarantees are not removed or replaced with unchanged-state success. Standalone headless-capable helper completion test also covers actual specialization transition.

## 5. Expected RED and proof limits

| Case | Expected failure without required change | Proof |
|---|---|---|
| Old aura fixture | Empty child order | Observed original b99-red.log; source proves skipped second load. |
| Corrected aura fixture | No known behavioral RED; existing cached sorting should pass | Authored, not run. |
| Helper delegation/taint tests without producer registration | No actual UI activation/Ready-field overwrite/observer error; pending specialization absent | Original five-test RED exists; revised expectations are predictions. |
| Delegation without secure-stack boundary | Existing-slot UI writes tainted; caller/observer assertions expose laundering | Source reasoning, not mutation execution. |
| Delegation with old completion body | Player index changes while config/hero/talent state stays old; ACTIVE_PLAYER_SPECIALIZATION_CHANGED missing | Newly authored completion assertions; not executed. |

A tests-only checkout against aa29d7d7f lacks the new shared tick API and is not a runnable behavioral RED checkpoint. Compile the shared extraction/exposure first if separating lifecycle plumbing from behavioral producer changes. No such build was performed here.

Allowed existing-binary runs: secure_group_headers 2/2 PASS; old hero immediate-mutation test 1/1 PASS, exit 0, 0.27 seconds. Logs: `$SCRATCH/aura-v2-existing-headers.log` and `$SCRATCH/aura-v2-existing-hero.log`. These controls do not validate new code.

Standalone rustfmt parser/format runs on staged Rust exited 0. Manual readability and exact-edit reconstruction completed; all final OLD anchors match current tree. No compilation, revised-test execution, full-suite execution, independent acceptance, startup smoke, or native-client comparison was performed.

**Merge risk today:** uncompiled candidate; corrected vendor fixtures may expose additional primitive gaps; delayed loadout commit/castbar path remains unmodeled. No full-row acceptance credit should be recorded yet. Main integrator must compile/run revised helper tests, rewritten existing hero test, affected fixture tests and relocated cast tests before claiming success.
