# Retail 12.0.7 B23/B24/B27/B28 author handoff

Status: authoring complete; not integrated, compiled or executed.
Base requested: master 7702befe8. Only audit-cache writes authorized.

Goal: source/declaration/provider evidence for four batches; exact unique anchored edits separated into state/producer/test-support; staged behavioral Lua API tests and spec. Exclude blocked behavior rather than intentionally red tests. No cargo, builds, tests, simulator, git mutations, agents or model CLIs.

Proof ledger: no execution evidence. Static inspection only; integration must establish RED/GREEN and cached-consumer acceptance.

## Evidence and dispositions

Confirmed read-only `git rev-parse HEAD`: `7702befe8a567c225d9e8680594186e9689734f4`.
Source is the retained excerpt, not authenticated complete historical page. Cached root below is `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`; declarations may postdate 12.0.7. Quotes are local evidence, not native execution proof.

| Batch / row | Source quote | Cached declaration / consumer quote | Current provider | Required delta / disposition |
|---|---|---|---|---|
| B23 `prose-undated-018` | L18: "- AuraData.isFromPlayerOrPlayerPet true if aura came from player-controlled vehicle." | No exact generated AuraData field declaration located. `Blizzard_FrameXMLUtil/AuraUtil.lua:274`: `Player = "PLAYER", -- Include only auras that were cast by the player, or by the player's pet or vehicle`. `Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua:95`: `if (candidateFilters.isFromPlayerOrPlayerPet ~= nil) and (auraData.isFromPlayerOrPlayerPet ~= candidateFilters.isFromPlayerOrPlayerPet) then` | `src/lua_api/game_data.rs:88` source token, `:96` explicit marker; `src/lua_api/globals/auras.rs:272` collection, `:817` boolean publication | Add empty explicit controlled-vehicle source-token set; classify fresh cloned DTOs live. Preserve existing direct-player/pet marker. INFERRED token identity and live timing; no ownership acquisition or GUID alias claim. |
| B24 `prose-undated-019` | L19: "- Addons allowed to call C_UnitAuras.AddPrivateAuraAppliedSound during active M+ if player not in combat." | No original AddPrivate declaration in later cache. `Blizzard_Deprecated/Shared/Deprecated_12_1_0.lua:22–23`: `C_UnitAuras.AddPrivateAuraAppliedSound = function(sound)` / `return C_UnitAuras.AddAuraSound(Enum.UnitAuraSoundTrigger.Added, sound);`. `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:11–14`: `Name = "AddAuraSound",` / `HasRestrictions = true,` / `SecretArguments = "AllowedWhenUntainted",` | `src/c_api/private_aura_sounds/add.rs:167–180` denies insecure encounter OR M+ OR PvP; existing world/party/player booleans, registration and allocator are live state | Gate only M+ predicate by combat under 12.0.7; simultaneous encounter/PvP still deny (INFERRED). 32-case secure/tainted × encounter × M+ × PvP × combat matrix. Existing authentication unchanged. |
| B27 `source-context-004` | L4: "Undocumented: Removed IMPORTANT from AuraFilters." | `Blizzard_FrameXMLUtil/AuraUtil.lua:270–286` assigns a new `AuraUtil.AuraFilters` table, including `Important = "IMPORTANT",` at :286 | `src/lua_api/globals/auras.rs:156–175` initializes only when table absent, omits Important; parser :148–150 omits IMPORTANT | BLOCKED / EXCLUDED. Full cache overwrites simulator table. Do not patch vendor, inject post-load cleanup, change parser from a publication claim, or stage a knowingly failing full-load test. Need epoch-pinned vendor evidence/cache reconciliation first. No capability credit even though bootstrap omits it. |
| B28 `prose-undated-011` | L11: "- Added C_UIFileAsset namespace: GetFileID(asset), IsKnownFile(asset), IsLooseFile(asset)." | `Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:11–13,27–29,43–45`: three names; all `SecretArguments = "AllowedWhenUntainted"` | `src/c_api/c_ui_file_asset.rs:14–29` registers all three; GetFileID :33–74 is already authenticated | Namespace row PARTIAL candidate only: GetFileID already integrated, not reauthored; bounded registry predicates below. No claim of complete native catalog. |
| B28 `global api-C_UIFileAsset-IsKnownFile-050` | L50: "C_UIFileAsset.IsKnownFile" | Same cached file :29 `SecretArguments = "AllowedWhenUntainted",`; :39 `True if the asset is shipped with the client or refers to a known loose file. Existence or openability of loose files is not verified.` | `src/c_api/c_ui_file_asset.rs:76–79,92–117` classifies every positive converted number as shipped and checks filesystem for strings | Candidate: authenticate all args/extras first; read explicit known-shipped IDs and known-loose path membership live, false empty default, no IO. Normalization/type/domain/public result are INFERRED. |
| B28 `global api-C_UIFileAsset-IsLooseFile-051` | L51: "C_UIFileAsset.IsLooseFile" | Same cached file :45 `SecretArguments = "AllowedWhenUntainted",`; :55 `True if the asset refers to a loose file known to the client.` | `src/c_api/c_ui_file_asset.rs:81–84,119–176` checks selected addon filesystem, including extension probing | Candidate: explicit loose registry instead of filesystem existence; shipped asset precedence, no inferred extensions, positive integral u32 domain reused from GetFileID. INFERRED historical adoption and registry normalization. |

B28 candidate intentionally does not auto-populate registries from loader/filesystem. Host owns the explicit catalog and selected-root reconciliation. This gap prevents promoting B28 to complete coverage; integrate only as a bounded host-input candidate. Ordinary empty environment queries become false, not fabricated known IDs. Legacy filesystem-based tests must be epoch-limited, not silently made green by fallback.

## Anchored edits

Apply all OLD matches against original master bytes, not successively mutated files. Entries are non-overlapping per file. New test/spec files come from staging. For RED: apply state and test-support edits plus tests, withhold ALL producer edits. No passing proof is claimed.

### E01 — `state` — `src/lua_api/state/sim_state.rs`

OLD (unique against master):
```rust
    pub tiered_entrance_pde_id: u32,
```

NEW:
```rust
    pub tiered_entrance_pde_id: u32,
    /// INFERRED source-token ownership and live aura-query timing; empty by default.
    #[cfg(feature = "retail-12-0-7")]
    pub player_controlled_vehicle_sources: HashSet<String>,
    /// INFERRED explicit client catalog; GetFileID resolution does not imply membership.
    #[cfg(feature = "retail-12-0-7")]
    pub known_shipped_asset_ids: HashSet<u32>,
    /// INFERRED normalized lowercase slash paths; host owns selected-root reconciliation.
    #[cfg(feature = "retail-12-0-7")]
    pub known_loose_asset_paths: HashSet<String>,
```

### E02 — `state` — `src/lua_api/state.rs`

OLD (unique against master):
```rust
            tiered_entrance_pde_id: 0,
```

NEW:
```rust
            tiered_entrance_pde_id: 0,
            #[cfg(feature = "retail-12-0-7")]
            player_controlled_vehicle_sources: HashSet::new(),
            #[cfg(feature = "retail-12-0-7")]
            known_shipped_asset_ids: HashSet::new(),
            #[cfg(feature = "retail-12-0-7")]
            known_loose_asset_paths: HashSet::new(),
```

### E03 — `producer` — `src/lua_api/globals/auras.rs`

OLD (unique against master):
```rust
fn collect_unit_auras(state: &mut LuaState, unit: &str, filter: AuraFilter) -> Vec<AuraInfo> {
    let Ok(sim) = borrow_state(state) else {
        return Vec::new();
    };
    use crate::lua_api::globals::unit_api::parse_party_index;
    let auras: Vec<&AuraInfo> = if let Some(idx) = parse_party_index(unit) {
        if let Some(member) = sim.party_members.get(idx) {
            match filter {
                AuraFilter::Helpful => member.buffs.iter().collect(),
                AuraFilter::Harmful => member.debuffs.iter().collect(),
                AuraFilter::ExternalDefensive => return Vec::new(),
                AuraFilter::Maw => return Vec::new(),
            }
        } else {
            return Vec::new();
        }
    } else if unit == "player" {
        sim.player
            .buffs
            .iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect()
    } else if unit == "target" {
        let target_auras = target_fixture_auras();
        return target_auras
            .into_iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect();
    } else {
        return Vec::new();
    };
    auras.into_iter().cloned().collect()
}

```

NEW:
```rust
fn collect_unit_auras(state: &mut LuaState, unit: &str, filter: AuraFilter) -> Vec<AuraInfo> {
    let Ok(sim) = borrow_state(state) else {
        return Vec::new();
    };
    use crate::lua_api::globals::unit_api::parse_party_index;
    let auras: Vec<&AuraInfo> = if let Some(idx) = parse_party_index(unit) {
        if let Some(member) = sim.party_members.get(idx) {
            match filter {
                AuraFilter::Helpful => member.buffs.iter().collect(),
                AuraFilter::Harmful => member.debuffs.iter().collect(),
                AuraFilter::ExternalDefensive => return Vec::new(),
                AuraFilter::Maw => return Vec::new(),
            }
        } else {
            return Vec::new();
        }
    } else if unit == "player" {
        sim.player
            .buffs
            .iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect()
    } else if unit == "target" {
        let target_auras = target_fixture_auras();
        let target_auras = target_auras
            .into_iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect();
        #[cfg(feature = "retail-12-0-7")]
        let target_auras =
            classify_vehicle_auras(target_auras, &sim.player_controlled_vehicle_sources);
        return target_auras;
    } else {
        return Vec::new();
    };
    let copied = auras.into_iter().cloned().collect();
    #[cfg(feature = "retail-12-0-7")]
    let copied = classify_vehicle_auras(copied, &sim.player_controlled_vehicle_sources);
    copied
}

/// INFERRED token identity and live ownership timing; never mutates stored AuraInfo.
#[cfg(feature = "retail-12-0-7")]
fn classify_vehicle_auras(
    auras: Vec<AuraInfo>,
    controlled_sources: &std::collections::HashSet<String>,
) -> Vec<AuraInfo> {
    auras
        .into_iter()
        .map(|mut aura| {
            let controlled_vehicle = controlled_sources.contains(&aura.source_unit);
            aura.is_from_player_or_player_pet |= controlled_vehicle;
            aura
        })
        .collect()
}

```

### E04 — `producer` — `src/c_api/private_aura_sounds/add.rs`

OLD (unique against master):
```rust
    let restricted = sim.world.encounter_in_progress
        || sim.mythic_plus.is_active
        || sim.private_aura_sound_registrations.pvp_match_active;
    // INFERRED HasRestrictions policy: no insecure registration in these contexts.
```

NEW:
```rust
    #[cfg(feature = "retail-12-0-7")]
    let mythic_restricted = sim.mythic_plus.is_active && sim.player.in_combat;
    #[cfg(not(feature = "retail-12-0-7"))]
    let mythic_restricted = sim.mythic_plus.is_active;
    let restricted = sim.world.encounter_in_progress
        || mythic_restricted
        || sim.private_aura_sound_registrations.pvp_match_active;
    // INFERRED: encounter/PvP restrictions still win over out-of-combat M+ permission.
```

### E05 — `test-support` — `tests/private_aura_sound_add_context.rs`

OLD (unique against master):
```rust
        set_context(&env, context.0, context.1, context.2);
        addon(&env, "DenySound()");
```

NEW:
```rust
        set_context(&env, context.0, context.1, context.2);
        // This existing denial test targets restricted M+, not the 12.0.7 exception.
        env.state().borrow_mut().player.in_combat = true;
        addon(&env, "DenySound()");
```

### E06 — `producer` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(!matches!(query_asset(state), Asset::Missing)));
    Ok(1)
}

fn c_ui_file_asset_is_loose_file(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(matches!(query_asset(state), Asset::Loose)));
    Ok(1)
}

enum Asset {
    Shipped,
    Loose,
    Missing,
}

fn query_asset(state: &LuaState) -> Asset {
    if file_id_from_asset_arg(state).is_some() {
        return Asset::Shipped;
    }
    let Some(path) = val_to_string(state, stack_val(state, 1)) else {
        return Asset::Missing;
    };
    if query_selected_addon_file(state, &path) {
        Asset::Loose
    } else {
        Asset::Missing
    }
}
```

NEW:
```rust
fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
    let asset = query_asset(state)?;
    state.push(Val::Bool(!matches!(asset, Asset::Missing)));
    Ok(1)
}

fn c_ui_file_asset_is_loose_file(state: &mut LuaState) -> LuaResult<u32> {
    let asset = query_asset(state)?;
    state.push(Val::Bool(matches!(asset, Asset::Loose)));
    Ok(1)
}

enum Asset {
    Shipped,
    Loose,
    Missing,
}

/// INFERRED public boolean, positive integral u32 domain and ignored authenticated extras.
#[cfg(feature = "retail-12-0-7")]
fn query_asset(state: &LuaState) -> LuaResult<Asset> {
    let asset = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    for value in state.stack.iter().take(state.top).skip(state.base + 1) {
        rilua::table_security::unwrap_secret(state, *value)?;
    }
    // No validation can mask secret denial in any original argument or extra.
    let file_id = resolve_authenticated_asset_id(state, asset)?;
    let path = val_to_string(state, asset);
    let sim = borrow_state(state)?;
    if file_id.is_some_and(|id| sim.known_shipped_asset_ids.contains(&id)) {
        return Ok(Asset::Shipped);
    }
    // INFERRED exact normalized path membership; no filesystem or extension probing.
    let is_loose = path.is_some_and(|path| {
        let normalized = path.replace('\\', "/").to_ascii_lowercase();
        sim.known_loose_asset_paths.contains(&normalized)
    });
    Ok(if is_loose { Asset::Loose } else { Asset::Missing })
}

#[cfg(not(feature = "retail-12-0-7"))]
fn query_asset(state: &LuaState) -> LuaResult<Asset> {
    if file_id_from_asset_arg(state).is_some() {
        return Ok(Asset::Shipped);
    }
    let Some(path) = val_to_string(state, stack_val(state, 1)) else {
        return Ok(Asset::Missing);
    };
    Ok(if query_selected_addon_file(state, &path) {
        Asset::Loose
    } else {
        Asset::Missing
    })
}
```

### E07 — `producer` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
fn file_id_from_asset_arg(state: &LuaState) -> Option<u32> {
    if let Some(file_id) = numeric_file_id_arg(state) {
        return Some(file_id);
    }
    let path = val_to_string(state, stack_val(state, 1))?;
    crate::limited_listfile::lookup_path(&path)
}

fn numeric_file_id_arg(state: &LuaState) -> Option<u32> {
    let file_id = u32::from_stack(state, 1).ok()?;
    (file_id > 0).then_some(file_id)
}

fn query_selected_addon_file(state: &LuaState, path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let Some(parts) = valid_addon_parts(&normalized) else {
        return false;
    };
    let root = {
        let Ok(sim) = borrow_state(state) else {
            return false;
        };
        sim.addons
            .iter()
            .find(|addon| addon.folder_name.eq_ignore_ascii_case(parts[2]))
            .and_then(|addon| addon.addon_dir.clone())
    };
    let Some(root) = root else {
        return false;
    };
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    texture_candidates(&parts[3..])
        .iter()
        .any(|candidate| read_selected_addon_file(&root, &canonical_root, candidate))
}

fn valid_addon_parts(path: &str) -> Option<Vec<&str>> {
    let parts: Vec<_> = path.split('/').collect();
    let valid_prefix = parts.len() >= 4
        && parts[0].eq_ignore_ascii_case("Interface")
        && parts[1].eq_ignore_ascii_case("AddOns");
    let valid_components = parts
        .iter()
        .all(|part| !part.is_empty() && *part != "." && *part != ".." && !part.contains(':'));
    (valid_prefix && valid_components).then_some(parts)
}

fn texture_candidates(relative: &[&str]) -> Vec<String> {
    let base = relative.join("/");
    let mut candidates = vec![base.clone()];
    if !relative.last().is_some_and(|name| name.contains('.')) {
        candidates.extend(["blp", "tga", "png"].map(|ext| format!("{base}.{ext}")));
    }
    candidates
}

fn read_selected_addon_file(root: &Path, canonical_root: &Path, candidate: &str) -> bool {
    let mut file = PathBuf::from(root);
    for part in candidate.split('/') {
        let Ok(entries) = std::fs::read_dir(&file) else {
            return false;
        };
        let Some(entry) = entries.filter_map(Result::ok).find(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(part))
        }) else {
            return false;
        };
        file.push(entry.file_name());
    }
    file.canonicalize()
        .is_ok_and(|resolved| resolved.starts_with(canonical_root) && resolved.is_file())
}
```

NEW:
```rust
#[cfg(not(feature = "retail-12-0-7"))]
fn file_id_from_asset_arg(state: &LuaState) -> Option<u32> {
    if let Some(file_id) = numeric_file_id_arg(state) {
        return Some(file_id);
    }
    let path = val_to_string(state, stack_val(state, 1))?;
    crate::limited_listfile::lookup_path(&path)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn numeric_file_id_arg(state: &LuaState) -> Option<u32> {
    let file_id = u32::from_stack(state, 1).ok()?;
    (file_id > 0).then_some(file_id)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn query_selected_addon_file(state: &LuaState, path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let Some(parts) = valid_addon_parts(&normalized) else {
        return false;
    };
    let root = {
        let Ok(sim) = borrow_state(state) else {
            return false;
        };
        sim.addons
            .iter()
            .find(|addon| addon.folder_name.eq_ignore_ascii_case(parts[2]))
            .and_then(|addon| addon.addon_dir.clone())
    };
    let Some(root) = root else {
        return false;
    };
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    texture_candidates(&parts[3..])
        .iter()
        .any(|candidate| read_selected_addon_file(&root, &canonical_root, candidate))
}

#[cfg(not(feature = "retail-12-0-7"))]
fn valid_addon_parts(path: &str) -> Option<Vec<&str>> {
    let parts: Vec<_> = path.split('/').collect();
    let valid_prefix = parts.len() >= 4
        && parts[0].eq_ignore_ascii_case("Interface")
        && parts[1].eq_ignore_ascii_case("AddOns");
    let valid_components = parts
        .iter()
        .all(|part| !part.is_empty() && *part != "." && *part != ".." && !part.contains(':'));
    (valid_prefix && valid_components).then_some(parts)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn texture_candidates(relative: &[&str]) -> Vec<String> {
    let base = relative.join("/");
    let mut candidates = vec![base.clone()];
    if !relative.last().is_some_and(|name| name.contains('.')) {
        candidates.extend(["blp", "tga", "png"].map(|ext| format!("{base}.{ext}")));
    }
    candidates
}

#[cfg(not(feature = "retail-12-0-7"))]
fn read_selected_addon_file(root: &Path, canonical_root: &Path, candidate: &str) -> bool {
    let mut file = PathBuf::from(root);
    for part in candidate.split('/') {
        let Ok(entries) = std::fs::read_dir(&file) else {
            return false;
        };
        let Some(entry) = entries.filter_map(Result::ok).find(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(part))
        }) else {
            return false;
        };
        file.push(entry.file_name());
    }
    file.canonicalize()
        .is_ok_and(|resolved| resolved.starts_with(canonical_root) && resolved.is_file())
}
```

### E08 — `producer` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
```

NEW:
```rust
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_bridge::FromStack;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
```

### E09 — `producer` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
use std::path::{Path, PathBuf};
```

NEW:
```rust
#[cfg(not(feature = "retail-12-0-7"))]
use std::path::{Path, PathBuf};
```

### E10 — `test-support` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
        let env = WowLuaEnv::new().expect("env");
        let result: String = env
```

NEW:
```rust
        let env = WowLuaEnv::new().expect("env");
        #[cfg(feature = "retail-12-0-7")]
        env.state().borrow_mut().known_shipped_asset_ids.insert(136243);
        let result: String = env
```

### E11 — `test-support` — `src/loader/tests/wow_api_globals/startup_globals.rs`

OLD (unique against master):
```rust
fn test_patch_12_0_7_safe_global_bridges() {
    let env = WowLuaEnv::new().unwrap();
```

NEW:
```rust
fn test_patch_12_0_7_safe_global_bridges() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().known_shipped_asset_ids.insert(136243);
```

### E12 — `test-support` — `tests/ui_file_assets.rs`

OLD (unique against master):
```rust
//! Addon-local file queries must use the TOC directory selected by the real loader.
```

NEW:
```rust
//! Legacy pre-12.0.7 filesystem contract. 12.0.7 host registry tests live in
//! patch_12_0_7_b23_b28.rs; loader registry acquisition is not modeled there.
#![cfg(not(feature = "retail-12-0-7"))]
```

### E13 — `producer` — `src/c_api/c_ui_file_asset.rs`

OLD (unique against master):
```rust
//! Patch 12.0.7 exposes a small lookup surface for UI file assets. The
//! simulator recognizes shipped IDs from the bundled listfile and loose files
//! within the loader-selected addon directory.
```

NEW:
```rust
//! Under 12.0.7, GetFileID resolves bundled paths while known/loose queries
//! read explicit host catalogs. Catalog acquisition is not modeled.
//! Earlier epochs retain their separate loader-selected filesystem contract.
```

## Deliverables and integration boundary

- `staging/p1207-b23-b28/tests/patch_12_0_7_b23_b28.rs` — full new file, 13 behavioral Rust tests through WowLuaEnv. Three vehicle cases, four sound cases, six asset cases. Sound matrix covers 32 fresh environments. Strict 12.0.7 selects 12 cases; later 12.1.0 selects the additional cached-forwarding case; strict earlier 12.0.5 selects three sound cases with old-policy expectations. Existing integration generation discovers the file; do not add a second Cargo test binary or manual module duplicate.
- `staging/p1207-b23-b28/docs/specs/patch-12-0-7-vehicle-sound-assets.md` — full new spec, following explicit-stat-inputs section structure. Every requirement remains unchecked because tests were not run.
- 13 anchored edits: state E01/E02; producers E03/E04/E06/E07/E08/E09/E13; test-support E05/E10/E11/E12. Apply against original master; producer withholding is explicit for RED. New files are test/spec additions, not repository mutations.

### Existing-test changes and exclusions

E05 keeps the old denial test meaningful by setting combat true for its M+ case. E10/E11 explicitly configure the shipped ID queried by the existing path tests. E12 preserves all three old selected-root/physical-existence tests under the older epoch, rather than keeping old filesystem semantics as a new-epoch fallback. New-epoch tests deliberately replace that contract with host membership, including a physically present unregistered file and a registered file after physical deletion. This does NOT establish loader-selected-root acquisition; retain the gap rather than claim equivalent loader proof.

B27 is fully excluded. Its source row remains pending; no parser change, Important-table cleanup or intentionally failing full-load test is staged. Bootstrap absence alone cannot close it because the actual cache creates a new table containing the entry. No exact generated AuraFilters removal declaration exists in this cache.

B28 is a bounded candidate, not complete asset subsystem work. Historical 12.0.7 declaration pinning, automatic catalog population before addon load, and selected-root identity/reconciliation are blocked/unmodeled. Host catalog entries are authoritative; callers must not assume existing filesystem-based registration survives this change automatically. Do not integrate B28 as a drop-in loader fix or promote its source rows without explicitly accepting that bounded host-input scope. No GetFileID edit or ledger promotion is proposed.

Vehicle ownership uses exact host-declared source tokens, not GUID aliases or lifetime-stable identities. The published sourceUnit recipient-dependent behavior is pre-existing and out of scope; classification reads the original stored AuraInfo source token. Existing explicitly true direct-player/pet flags are preserved, not recomputed from tokens.

Sound policy alone changes; allocation, field authentication, removal and secure bypass remain as implemented. Cached 12.1 forwarding uses the same check_context path through AddAuraSound; the extra staged test executes the actual cached deprecated chunk rather than substituting a wrapper. Simultaneous encounter/PvP rules are INFERRED and must not be represented as native evidence.

## Static proof ledger — 2026-10-04

| Inspection / command | Scope | Result / limits |
|---|---|---|
| `git rev-parse HEAD` (read-only) | requested base | `7702befe8a567c225d9e8680594186e9689734f4` |
| Python source-ID membership inspection | coverage JSON, six requested row IDs | all present; ledger not modified |
| Python exact OLD-span inspection using read-only `git show 7702befe8:<path>` | E01–E13 | each OLD matches exactly once; per-file spans non-overlapping; no edits applied to repository |
| `rustfmt --edition 2024 <staged test path>` | new test file | exit 0; formatting only, no compiler/behavior proof. Subsequent change is one Lua assertion inside an existing raw string, which does not change Rust formatting. |
| Manual Rust readability pass | all proposed NEW code and staged test helpers | producer helper is pure; catalog IO removed for new epoch; short new functions, no warning suppression. Existing collector body retained rather than adjacent refactor. No metrics, lint or type check executed. |
| Static argument-order and feature inspection | E03/E04/E06 | vehicle and registry inputs gated; M+ exception gated; asset authentication precedes type/domain/catalog validation and includes every extra; original GetFileID untouched |

No cargo, tests, builds, simulator, agents, model CLIs, git mutation, or repository writes. No execution claim, RED/GREEN claim, complete-row credit, or native parity claim. Integration must perform targeted acceptance and cached-consumer checks after applying the chosen bounded scope; this author handoff itself is complete.
