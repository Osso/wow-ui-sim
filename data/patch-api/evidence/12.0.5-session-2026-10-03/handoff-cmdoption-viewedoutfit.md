# CmdOption / viewed-outfit authoring handoff

Authoring complete: 2026-10-03. **Not applied, compiled, tested, committed, or deployed.**

## Scope and artifacts

Pinned base: `aa29d7d7f06b14f33990c36ae736b98579b00b78` (`aa29d7d7f`), not the concurrently changing working tree. Wrote only this handoff and `scratchpad/staging/cmdoption-viewedoutfit/`. No repo/vendor writes, cargo/test execution, git mutations, agents, Supervisor, or model CLIs.

Staging root: `/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/staging/cmdoption-viewedoutfit/`.

- [Exact edits](staging/cmdoption-viewedoutfit/edits.json): 25 unique, non-overlapping old/new anchors against pinned base, tagged `state`, `producer`, `test`, or `spec`; three new-file entries with full-code paths/hashes.
- [Complete patch](staging/cmdoption-viewedoutfit/changes.patch): all changes, including full new files; reproduced below. Full resulting files also mirror repo paths in staging.
- Do not copy full staged existing files over another worker's changes. Reconcile `oldText` anchors against current files; if an anchor changed, compare pinned base and current ownership before integration. Applying the combined patch at once cannot demonstrate state/test-only RED.

| Tag | Paths | Purpose |
|---|---|---|
| state | `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` | Optional `viewed_transmog_outfit_id`, initialized None, epoch-gated |
| producer | `src/lua_api/globals/security/cmd_option.rs` | Public selected text + explicit unit; preserve internal default target |
| producer | `src/c_api/c_transmog_outfit_info.rs`, new `src/c_api/c_transmog_outfit_info/viewed.rs` | Register/query/change host-owned viewed selection and synchronous event |
| producer | `src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs` | Remove selected-epoch Lua viewed reader/key/helper; move reader into excluded legacy chunk |
| test | new `tests/cmd_option_selected_unit.rs`, new `tests/viewed_outfit_selection.rs` | Direct contracts, entire real vendor slash file/registered handler, host/event/security boundary |
| test | `tests/transmog_outfit_info.rs`, `tests/pending_transmog_cost.rs` | Replace viewed rawset fixtures with `ChangeViewedOutfit` |
| spec | `docs/specs/target-marker-macro-command.md`, `docs/specs/outfit-action-command.md` | Follow-up requirements, inferred policies, tests, exclusions; no new checked proof claims |

## (a) Selected unit: evidence, implementation, affected consumers

Review accepted: `data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md`, section 2. It identifies the missing second result as a pre-existing gap in vendor `/tm`, distinct from the already-tested simulator macro path.

Cache root: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns`.

`Blizzard_ChatFrameBase/Shared/SlashCommands.lua:1401–1404`:

```lua
local marker, target = SecureCmdOptionParse(msg);
if ( not target ) then
    target = "target";
end
```

The same file's ASSIST/FOCUS/role handlers use `if ( not target ) then target = action; end` (570–650). This establishes a meaningful absent-second-result path; blindly publishing an implicit `target` changes more than `/tm`.

**Evidence limit:** a complete cached Lua search found callers/allowlisting, not a parser implementation or API declaration of the second return. In particular, inspected RestrictedInfrastructure/SecureHandlers provide no definition; RestrictedEnvironment.lua:82 allowlists the name, and SecureStateDriver.lua:96 takes only the first result. Therefore **nil without an explicit selector is INFERRED from consumer behavior**, not native parity evidence. The cache does not settle exact native return arity either.

Implementation shares a single explicit-unit resolver. On success, push selected text and explicit unit (nil when absent). Preserve existing one-nil no-match/non-string behavior. Existing Rust `resolve_cmd_option_with_unit` still defaults to `target`; `resolve_cmd_option` still returns text only. First matching clause/predicate grammar are unchanged. `unit=` and last-selector-wins remain explicitly inferred existing policies. No unsupported bracket grammar expansion or secret-input policy change.

The real `/tm` test reads the **entire unchanged** cached SlashCommands file. A normal Lua chunk wrapper supplies addon-name/addon-table varargs; it does not replace registration or extract/copy the handler. It then invokes the file's registered `SlashCmdList[SLASH_COMMAND.TARGET_MARKER]` with distinct target/focus GUIDs and checks actual marker queries and event callbacks. Host fixture sets `game_rules.active_game_mode = 1` (cached Enum.GameMode.Standard); default host mode 0 registers no commands. No vendor API/enum/parser/marker patches or fallback cache path.

Affected existing consumers/tests:

| Consumer | Effect |
|---|---|
| `src/lua_api/globals/security/state_drivers.rs:104` | Still uses text-only resolver; no code/test change required |
| `tests/security_api.rs` six `test_securecmdoptionparse_*` cases | First-value expectations retained; String result conversion reads first result (`env_convert.rs`), so added unit slot does not alter those assertions |
| `tests/mouse_tm_commands.rs`, `tests/outfit_action_command.rs`, state-driver tests | Regression controls for shared resolver, unit destinations and first-value macro selection; no rewrites |
| Cached SlashCommands second-value consumers | STARTATTACK 348; CAST 365; USE 378; TARGET/TARGET_EXACT 479/489; ASSIST/FOCUS 570/584; MAINTANKON/OFF 601/614; MAINASSISTON/OFF 627/640; PET_ATTACK/PET_MOVE_TO 661/677; WORLD_MARKER 822; TARGET_MARKER 1401 |
| Other cached consumers | CastRandomManager.lua:83 and CastSequenceManager.lua:254 consume target; Mainline/SlashCommandsOverrides.lua:56 consumes second value; single-value callers remain unchanged |

Parser registration is shared across profiles. New behavioral tests are gated to `retail-12-0-5`; older-profile native parity is not established. Existing restricted-environment load tests mention/allowlist the parser but do not directly assert its second return. Full chat input/addon-load/alias/security-authority proof remains outside this test.

## (b) Viewed selection: evidence and transition

Review accepted: `b98-verify-outfit-formatter.md`, section 1, viewed/pending fixture discussion. Missing declared setter is the root gap; rawset fixtures do not earn state-backed producer credit.

Cached `Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:62–70` declares:

```lua
Name = "ChangeViewedOutfit",
Type = "Function",
SecretArguments = "AllowedWhenUntainted",
Arguments = { { Name = "outfitID", Type = "number", Nilable = false } },
```

No Returns block: setter returns zero values. Lines 175–182 declare one nonnil numeric query result. Lines 835–839 declare `VIEWED_TRANSMOG_OUTFIT_CHANGED`, `SynchronousEvent = true`, no payload. Documentation does not explicitly link event timing to a particular setter.

`Blizzard_Transmog/Blizzard_Transmog.lua:514` says: “Active outfit is the outfit the player is wearing out in the world, viewed is what is being viewed in the transmog frame.” Lines 525–526 request selection on initial opening even if unchanged. `Blizzard_TransmogTemplates.lua:140–159` passes `elementData.outfitID`, not a player-facing index, after pending-action checking.

New `viewed.rs` authenticates the argument before borrowing/mutating host state. Resolve against catalog **ID** using existing first-match lookup policy. Store only host `viewed_transmog_outfit_id`. Getter returns its public numeric ID (initial 0). Mutation ends its mutable borrow before dispatch so synchronous listeners can query updated state.

**INFERRED policies, not native claims:** strict number input; absent selection encodes 0; catalog misses preserve selection and emit no refresh; first matching duplicate ID wins; disabled flags do not impose invented eligibility; every valid request, including repeated selection, sends the payload-free synchronous viewed event; pending cost/slot/sheathe snapshots remain intact. Event association/repeat behavior follows the visible refresh consumer but is not explicitly specified by the declaration. No displayed/catalog-change event is invented. No availability, appearance application, persistence, pending-discard, displayed-outfit, or protected-authority policy is added.

Selected epoch has no `__currentlyViewedOutfitID` reader/writer. Its getter is Rust-only; test writes to the old field cannot affect results. Historical getter is relocated into the existing `#[cfg(not(feature = "retail-12-0-5"))]` chunk alongside its existing historical writers. This is compile-time epoch separation, not runtime fallback or dual state. Pending sheathe Lua storage remains the pre-existing separate compatibility subsystem, not a duplicate viewed-state store.

Fixtures retain their purposes: active toggle/clear independence in `transmog_outfit_info`, and query-read-only cost behavior in `pending_transmog_cost`. Both now call the real setter. Disabled catalog fixture still selects under the explicitly inferred non-eligibility policy; existing slot/cost/lock/catalog preservation assertions remain intact.

## Expected RED and downstream verification

No RED/GREEN was executed. These are static counterfactuals at pinned base, not observed failures.

| Test scope | Expected failure before producers |
|---|---|
| `cmd_option_selected_unit::cmd_option_returns_only_explicit_unit_from_the_selected_clause` | Successful parse returns one value, so exact two-slot check fails; selector-specific second value is nil |
| `cmd_option_selected_unit::cached_vendor_target_marker_handler_uses_selected_unit_and_default_target` | With Standard-mode fixture, real handler falls back to target: focus marker assertion fails, target receives the marker |
| `viewed_outfit_selection::viewed_outfit_reads_host_state_not_writable_lua_storage_and_is_isolated` | Old Lua getter reads injected 999 rather than initial host selection; later host-state/API assertions also discriminate |
| Remaining new viewed tests and both edited fixtures | Declared setter is absent, so first `ChangeViewedOutfit` call raises rather than performing mutation/event |

For compilable RED, integrate **state + tests only**, not producers or legacy removal. New viewed test directly references the new host field; tests alone on base fail compilation, which is not behavioral RED. Adding the optional field/None initializer is a transparent compile bridge and does not implement the setter/getter. New module and registration belong only to producer phase. Feature gate first line is present in all four staged test files. New Lua assertions use `select('#', ...)`; eval numeric assertions use f64, never u32. New wrapper newlines separate Lua statements, not newlines inside Lua quoted strings.

After producers, downstream main should run bounded integration filters `cmd_option_selected_unit`, `viewed_outfit_selection`, `transmog_outfit_info`, `pending_transmog_cost`, plus shared-parser controls `security_api`, `mouse_tm_commands`, `outfit_action_command`, relevant state-driver tests; local build-host only. Follow repo startup Lua-error and final format/check conventions after actual integration. Do not treat this authoring packet as GREEN or CI readiness. Spec checkboxes remain unchecked.

## Proof ledger and merge risk

| Proof | Scope/revision | Result |
|---|---|---|
| Read-only `git rev-parse`, `git show`, `git ls-tree` | Pinned `aa29d7d7f06b14f33990c36ae736b98579b00b78` | Base verified; each oldText appears exactly once, no overlaps; three new paths absent |
| Scratch reconstruction comparison | All nine edited files, current staging | Exact edit application equals full staged content |
| Standalone rustfmt, edition 2024, skip_children=true | Only staged Rust files; focused rerun after cmd-option/test edit | Exit 0, no output; never traversed repo module children |
| Static Lua/test/storage inspection | Four staged tests + selected-epoch bootstrap | First-line gates; no u32 eval; current-viewed key/getter absent from common chunk |
| Manual Rust readability audit | Changed functions/lines only | Named resolve/push/select stages, bounded functions/nesting, no warning suppression; no additional changes needed |
| Cargo, startup, runtime tests, CI, deployment | None | Not executed/claimed |

Merge risk today: uncompiled/unexecuted packet; second-result nil/arity native contract and event miss/repeat/pending policies lack native evidence. Entire real slash file may expose environment dependencies at execution; no synthetic fallback is provided. Do not overwrite concurrent work or extend B98 earned coverage until new behavioral proof is recorded. No known source-level new fallback/dual viewed state remains.

## Full-code patch

The exact anchor manifest remains authoritative for conflict-aware application. This appendix includes complete new producer/test files and every existing-file edit; unchanged surrounding code lives in the full mirrored files.

```diff
--- a/src/lua_api/state/sim_state.rs
+++ b/src/lua_api/state/sim_state.rs
@@ -180,6 +180,9 @@
     /// Explicit applied selection; independent of viewed/pending outfit metadata.
     #[cfg(feature = "retail-12-0-5")]
     pub active_transmog_outfit_id: Option<i64>,
+    /// Explicit catalog-backed viewed selection; independent of active/pending state.
+    #[cfg(feature = "retail-12-0-5")]
+    pub viewed_transmog_outfit_id: Option<i64>,
     /// Global setting only; no per-outfit or pending-situation behavior.
     pub outfit_situations_enabled: bool,
     /// Explicit filter values only; native defaults and set filtering are unmodeled.
--- a/src/lua_api/state.rs
+++ b/src/lua_api/state.rs
@@ -174,6 +174,8 @@
             transmog_outfit_catalog: crate::c_api::c_transmog_outfit_info::OutfitCatalog::default(),
             #[cfg(feature = "retail-12-0-5")]
             active_transmog_outfit_id: None,
+            #[cfg(feature = "retail-12-0-5")]
+            viewed_transmog_outfit_id: None,
             // Simulator initial policy; native default is unverified.
             outfit_situations_enabled: false,
             transmog_set_filters: HashMap::new(),
--- a/src/c_api/c_transmog_outfit_info.rs
+++ b/src/c_api/c_transmog_outfit_info.rs
@@ -7,6 +7,8 @@
 
 #[cfg(feature = "retail-12-0-5")]
 mod actions;
+#[cfg(feature = "retail-12-0-5")]
+mod viewed;
 #[cfg(feature = "retail-12-0-5")]
 pub(crate) use actions::run_outfit_command;
 
@@ -41,6 +43,8 @@
     catalog::register(state, namespace)?;
     #[cfg(feature = "retail-12-0-5")]
     actions::register(state, namespace)?;
+    #[cfg(feature = "retail-12-0-5")]
+    viewed::register(state, namespace)?;
     #[cfg(all(
         feature = "retail-12-0-5",
         any(feature = "profile-retail", feature = "client-ptr")
--- a/src/lua_api/globals/security/cmd_option.rs
+++ b/src/lua_api/globals/security/cmd_option.rs
@@ -1,5 +1,5 @@
 //! `SecureCmdOptionParse(options)` — returns the first option whose bracketed
-//! condition list matches current simulator state.
+//! condition list matches current simulator state, plus its explicit unit selector.
 
 use rilua::vm::state::LuaState;
 use rilua::{LuaResult, Val, runtime_error};
@@ -24,16 +24,27 @@
     };
     let selected = {
         let sim = borrow_state(state)?;
-        resolve_cmd_option(&text, &sim).map(str::to_string)
-    };
-    match selected {
-        Some(value) => {
-            let result = Val::Str(state.gc.intern_string(value.as_bytes()));
-            state.push(result);
-        }
-        None => state.push(Val::Nil),
-    }
-    Ok(1)
+        resolve_cmd_option_with_explicit_unit(&text, &sim)
+            .map(|(value, unit)| (value.to_owned(), unit.map(str::to_owned)))
+    };
+    Ok(push_cmd_option_results(state, selected))
+}
+
+fn push_cmd_option_results(
+    state: &mut LuaState,
+    selected: Option<(String, Option<String>)>,
+) -> u32 {
+    let Some((value, unit)) = selected else {
+        state.push(Val::Nil);
+        return 1;
+    };
+    let value = Val::Str(state.gc.intern_string(value.as_bytes()));
+    state.push(value);
+    let unit = unit.map_or(Val::Nil, |unit| {
+        Val::Str(state.gc.intern_string(unit.as_bytes()))
+    });
+    state.push(unit);
+    2
 }
 
 pub(crate) fn resolve_cmd_option<'a>(
@@ -48,11 +59,19 @@
     text: &'a str,
     sim: &crate::lua_api::SimState,
 ) -> Option<(&'a str, &'a str)> {
+    resolve_cmd_option_with_explicit_unit(text, sim)
+        .map(|(value, unit)| (value, unit.unwrap_or("target")))
+}
+
+fn resolve_cmd_option_with_explicit_unit<'a>(
+    text: &'a str,
+    sim: &crate::lua_api::SimState,
+) -> Option<(&'a str, Option<&'a str>)> {
     text.split(';')
         .filter_map(parse_cmd_option_clause)
         .find_map(|clause| {
             clause.matches(sim).then(|| {
-                let unit = clause.conditions.map_or("target", condition_unit);
+                let unit = clause.conditions.and_then(explicit_condition_unit);
                 (clause.value, unit)
             })
         })
@@ -104,11 +123,14 @@
 }
 
 fn condition_unit(conditions: &str) -> &str {
+    explicit_condition_unit(conditions).unwrap_or("target")
+}
+
+fn explicit_condition_unit(conditions: &str) -> Option<&str> {
     // INFERRED: preserve the existing parser's last-selector-wins policy.
     conditions
         .rsplit(',')
         .find_map(|condition| parse_unit_override(condition.trim()))
-        .unwrap_or("target")
 }
 
 fn parse_unit_override(condition: &str) -> Option<&str> {
--- a/src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs
+++ b/src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs
@@ -1,13 +1,11 @@
 //! Temporary `C_TransmogOutfitInfo` slot/outfit defaults.
 //!
 //! Outfit locks are state-backed in `lua_api::globals::transmog_outfit_info`.
-//! Slot metadata, sheathe categories, and viewed metadata remain compatibility
-//! defaults. Retail 12.0.5 applied selection belongs to the catalog-backed C API.
+//! Slot metadata and sheathe categories remain compatibility defaults.
+//! Retail 12.0.5 active/viewed selection belongs to the catalog-backed C API.
 
 const TRANSMOG_OUTFIT_SLOT_DEFAULTS_LUA: &str = r#"
 C_TransmogOutfitInfo = C_TransmogOutfitInfo or __wow_namespace()
-
-local CURRENTLY_VIEWED_OUTFIT_ID_KEY = "__currentlyViewedOutfitID"
 local PENDING_SHEATHE_CATEGORIES_KEY = "__pendingSheatheCategories"
 local VALID_SHEATHE_SLOT_TRANSMOG_ID = 190001
 
@@ -20,15 +18,6 @@
     end
 
     return fallback
-end
-
-local function outfitIDValue(key)
-    local value = rawget(C_TransmogOutfitInfo, key)
-    if type(value) == "number" then
-        return value
-    end
-
-    return 0
 end
 
 local function numberOrNumericString(value)
@@ -98,12 +87,6 @@
     return slots
 end
 
-if rawget(C_TransmogOutfitInfo, "GetCurrentlyViewedOutfitID") == nil then
-    function C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()
-        return outfitIDValue(CURRENTLY_VIEWED_OUTFIT_ID_KEY)
-    end
-end
-
 if rawget(C_TransmogOutfitInfo, "GetAllTransmogOutfitOptionSheatheCategoryInfo") == nil then
     function C_TransmogOutfitInfo.GetAllTransmogOutfitOptionSheatheCategoryInfo(slotTransmogID)
         if numberOrNumericString(slotTransmogID) ~= VALID_SHEATHE_SLOT_TRANSMOG_ID then
@@ -176,6 +159,12 @@
 local function resetOutfitState()
     setOutfitIDs(0)
     C_TransmogOutfitInfo[PENDING_SHEATHE_CATEGORIES_KEY] = {}
+end
+
+if rawget(C_TransmogOutfitInfo, "GetCurrentlyViewedOutfitID") == nil then
+    function C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()
+        return outfitIDValue(CURRENTLY_VIEWED_OUTFIT_ID_KEY)
+    end
 end
 
 if rawget(C_TransmogOutfitInfo, "GetActiveOutfitID") == nil then
--- a/tests/transmog_outfit_info.rs
+++ b/tests/transmog_outfit_info.rs
@@ -30,8 +30,7 @@
         return "change_to_outfit_failed"
     end
 
-    -- ChangeViewedOutfit is declared but unimplemented here; seed the viewed-outfit storage directly.
-    rawset(C_TransmogOutfitInfo, "__currentlyViewedOutfitID", 7)
+    C_TransmogOutfitInfo.ChangeViewedOutfit(7)
     assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 7, "viewed metadata fixture should be queryable")
 
     C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(16, 2, Enum.TransmogOutfitSlotOptionSheatheCategory.Side)
--- a/tests/pending_transmog_cost.rs
+++ b/tests/pending_transmog_cost.rs
@@ -105,8 +105,7 @@
         C_TransmogOutfitInfo.ChangeToOutfit(7, false)
         assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91, 'catalog index 7 should select outfit 91')
         assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 0, 'selection should not set viewed metadata')
-        -- ChangeViewedOutfit is declared but unimplemented here; seed the viewed-outfit storage directly.
-        rawset(C_TransmogOutfitInfo, '__currentlyViewedOutfitID', 91)
+        C_TransmogOutfitInfo.ChangeViewedOutfit(91)
         C_TransmogOutfitInfo.SetPendingTransmogSheatheCategory(16, 2, 2)
         "#,
     )
@@ -120,7 +119,7 @@
     env.exec(
         r#"
         assert(C_TransmogOutfitInfo.GetActiveOutfitID() == 91, 'cost reads should preserve active selection')
-        assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 91, 'cost reads should preserve explicitly seeded viewed metadata')
+        assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 91, 'cost reads should preserve API-selected viewed outfit')
         local pending = rawget(C_TransmogOutfitInfo, '__pendingSheatheCategories')
         assert(pending['16:2'] == 2, 'cost reads should preserve pending sheathe category')
         local count = 0
--- a/docs/specs/target-marker-macro-command.md
+++ b/docs/specs/target-marker-macro-command.md
@@ -9,6 +9,7 @@
 - [ ] Unconditional `/tm 2` assigns marker 2; `/tm 0` clears it. Selected `@focus`, `target=focus` and `unit=focus` selectors drive both predicate evaluation and the destination unit.
 - [ ] **INFERRED simulator policy:** invalid, malformed, unselected or out-of-range decimal integer commands are atomic no-ops. A later valid command still succeeds. Destination absence follows the existing marker API's no-op policy; no synthetic unit is created.
 - [ ] Preserve existing marker assignment/movement and synchronous notification semantics. Their equivalence to native repeated-assignment/toggle behavior remains unproven.
+- [ ] `SecureCmdOptionParse` returns the selected text and explicit unit selector to the unchanged cached `/tm` handler. **INFERRED from cached consumers:** successful clauses without selectors return nil as the second value, not an invented target; `@unit`, `target=unit`, and the existing `unit=unit` alias return the selected unit. A rejected clause cannot leak its selector into a later match. Preserve existing first-value predicates, Rust resolver default destination, and no-selection return shape. Native parser implementation/return-arity documentation is not present in the inspected cache.
 
 ## How it works
 
@@ -26,6 +27,8 @@
 
 `tests/mouse_tm_commands.rs` — one auto-included `integration` module, feature-gated for `retail-12-0-5`. Assertions cover marker queries, actual stored unit icons, queued events and synchronous Lua event callbacks.
 
+`tests/cmd_option_selected_unit.rs` — direct text/unit results with and without selectors, rejected-clause isolation, and the entire unchanged cached `SlashCommands.lua` followed by its registered `TARGET_MARKER` callback. This exercises the vendor handler and existing marker/event APIs, not full chat input or addon loading. Follow-up is scratch-authored and unexecuted; prior B98 proof below does not cover it.
+
 ## Development proof and independent bounded acceptance — 2026-10-03
 
 Commit `a4cce2db1`. RED: 0 PASS / 5 FAIL. GREEN: 5/5 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.
@@ -41,6 +44,6 @@
 
 ## Out of scope
 
-This bounded numeric slice does not cover native `!`/`~` prefixes, toggle parity, slash aliases, Lua `tonumber`'s broader coercion, adjacent-bracket alternatives, unsupported macro predicates, or a fully loaded chat slash-handler path. Existing parser predicates are reused, not certified wholesale (notably placeholder pet/vehicle existence and coarse group matching are not native proof). `SecureCmdOptionParse`'s missing second Lua return is not changed; the shared Rust selector supplies the destination to this command.
+This bounded numeric slice does not cover native `!`/`~` prefixes, toggle parity, slash aliases, Lua `tonumber`'s broader coercion, adjacent-bracket alternatives, unsupported macro predicates, or a fully loaded chat addon/input path. Existing parser predicates are reused, not certified wholesale (notably placeholder pet/vehicle existence and coarse group matching are not native proof). The follow-up supplies `SecureCmdOptionParse`'s explicit second return to the real cached handler; the shared Rust resolver retains its default destination for simulator macro dispatch. Explicit-selector nil policy and existing last-selector-wins/`unit=` behavior are bounded simulator policies, not a native grammar certification.
 
 Row `prose-2026-03-25-099` is skipped, not counted covered: cached declarations contain `IsMouseOver` but no exact `IsUnderMouse` declaration. No invented alias or geometry policy is supplied.
--- a/docs/specs/outfit-action-command.md
+++ b/docs/specs/outfit-action-command.md
@@ -5,6 +5,9 @@
 ## What it must do
 
 - [ ] Keep per-environment host-owned applied selection independent of catalog, viewed and pending metadata. `GetActiveOutfitID()` observes that selection, not writable Lua namespace fields. **INFERRED:** initial absence and numeric zero for absent selection.
+- [ ] `ChangeViewedOutfit(outfitID)` resolves the catalog ID, not a player-facing index, and updates separate host-owned viewed selection. `GetCurrentlyViewedOutfitID()` reads that selection as one public numeric result; the mutation returns no values. Apply `AllowedWhenUntainted` validation before mutation; ordinary addon arguments do not cleanse taint. No selected-epoch Lua storage reader/writer or fallback remains. **INFERRED:** strict number input, initial zero, first matching catalog ID, unknown IDs preserve selection, and disabled flags do not define eligibility.
+- [ ] Deliver `VIEWED_TRANSMOG_OUTFIT_CHANGED` synchronously after a valid viewed request, with no payload and already-updated selection. **INFERRED:** this event's association with the setter and refresh on repeated valid selection; the cache declares synchronous delivery/no payload but not the precise trigger policy. Invalid/missing IDs emit no event. Do not invent displayed-outfit or catalog-change notifications.
+- [ ] **INFERRED:** viewed selection preserves active selection, pending cost, viewed-slot snapshots and pending sheathe metadata. Active change/clear continues to preserve viewed selection. No native pending-discard policy is claimed.
 - [ ] `ChangeToOutfit(index, allowRemoveOutfit)` resolves the catalog's published player-facing index, not its ID or vector position. A repeated selection clears only when removal is allowed; a different selection applies. `ClearOutfit()` removes selection idempotently. Both mutation APIs return zero values.
 - [ ] Reuse the unchanged vendor `SecureActionButton_OnClick` outfit dispatch: `outfit-index`, `action=change/toggle/clear`, and default toggle. No simulator reimplementation of vendor action selection.
 - [ ] Apply existing VM `AllowedWhenUntainted` validation to both ChangeToOutfit arguments before mutation. Ordinary addon arguments remain permitted and must not cleanse caller taint. **INFERRED:** strict number/bool argument types; missing catalog index preserves selection; duplicate indices choose first catalog entry, matching lookup policy.
@@ -17,9 +20,10 @@
 
 ## Implementation inventory
 
-- `src/c_api/c_transmog_outfit_info/actions.rs`: native selection query/transitions and slash-command bridge.
+- `src/c_api/c_transmog_outfit_info/actions.rs`: active selection query/transitions and slash-command bridge.
+- `src/c_api/c_transmog_outfit_info/viewed.rs`: host-backed viewed query, catalog-ID transition and synchronous refresh.
 - `src/c_api/c_transmog_outfit_info.rs`: patch-gated registration/export; catalog and unrelated settings unchanged.
-- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: optional active ID and empty initialization.
+- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: separate optional active/viewed IDs and empty initialization.
 - `src/lua_api/globals/security/{cmd_option.rs,mod.rs}`: share existing pure condition resolver without changing parser behavior.
 - `src/lua_api/globals/spell_macro_verbs.rs`: `/outfit` route and only that command's empty-argument allowance.
 - `src/lua_api/workarounds/temporary/transmog_outfit_slot_defaults.rs`: remove selected-epoch Lua selection defaults. Historical pre-12.0.5 code remains compile-time isolated, never available as a fallback in the modeled epoch. Existing historical combined lifecycle test is limited to its matching epoch.
@@ -27,6 +31,8 @@
 ## Tests asserting this spec
 
 `tests/outfit_action_command.rs`, auto-included in the grouped integration target: host snapshot/environment isolation, sparse-index transitions, missing/invalid inputs, secret authentication, conditional branches, macro toggles/clear, and the complete unchanged cached vendor SecureTemplates file followed by its real click-handler dispatch. No test invokes a copied/extracted handler or fabricated secure action.
+
+Scratch follow-up tests: `tests/viewed_outfit_selection.rs` asserts host/public query state, environment isolation, catalog-ID versus index, synchronous event payload/order, repeat/miss behavior, pending/active preservation, and authenticated secret/tainted ordinary calls. `tests/transmog_outfit_info.rs` and `tests/pending_transmog_cost.rs` select viewed fixtures through `ChangeViewedOutfit`, not raw storage writes. This follow-up is unexecuted; previous B98 proof below does not cover it.
 
 ## Development proof and independent bounded acceptance — 2026-10-03
 
@@ -45,4 +51,4 @@
 
 ## Out of scope
 
-Catalog population/create/delete/reorder, native wardrobe availability, 3D/transmog rendering, events, persistence, pending/viewed outfit transitions, protected-frame authority and older-profile parity. Prose/signatures do not define these policies; inventing them would be dishonest. No complete-row/full-page acceptance is claimed before the bounded tests are executed and accepted.
+Catalog population/create/delete/reorder, native wardrobe availability, 3D/transmog rendering, displayed-outfit transitions, unrelated events, persistence, pending edit/discard lifecycle, protected-frame authority and older-profile parity. Cached `TransmogOutfitInfoDocumentation.lua:62–70,175–182,835–839` declares the viewed setter/query and payload-free synchronous refresh event; `Blizzard_Transmog.lua:514–527` distinguishes active from viewed, and `Blizzard_TransmogTemplates.lua:140–159` selects by ID. Exact native miss/repeat/pending policies remain unproven and are marked INFERRED above. No complete-row/full-page acceptance is claimed before the bounded tests are executed and accepted.
--- /dev/null
+++ b/src/c_api/c_transmog_outfit_info/viewed.rs
@@ -0,0 +1,56 @@
+//! Catalog-backed viewed outfit selection, separate from applied/pending state.
+
+use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
+use crate::lua_api::methods::{borrow_state, borrow_state_mut};
+use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
+use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
+use rilua::{LuaResult, Val};
+
+pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
+    table_set_rust_fn_static(
+        state,
+        namespace,
+        "GetCurrentlyViewedOutfitID",
+        get_currently_viewed_outfit_id,
+    )?;
+    table_set_rust_fn_static(state, namespace, "ChangeViewedOutfit", change_viewed_outfit)
+}
+
+fn get_currently_viewed_outfit_id(state: &mut LuaState) -> LuaResult<u32> {
+    // INFERRED: numeric zero represents initial absence, like the active query.
+    let id = borrow_state(state)?.viewed_transmog_outfit_id.unwrap_or(0);
+    state.push(Val::Num(id as f64));
+    Ok(1)
+}
+
+fn change_viewed_outfit(state: &mut LuaState) -> LuaResult<u32> {
+    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
+    let Val::Num(id) = value else {
+        return Err(rilua::runtime_error(
+            "ChangeViewedOutfit requires a numeric outfitID",
+        ));
+    };
+    if select_viewed_outfit(state, id)? {
+        // INFERRED association: each valid request refreshes the viewed outfit,
+        // including repeat selection when the vendor first opens the frame.
+        // Declaration specifies synchronous delivery and no payload.
+        dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_CHANGED", &[])?;
+    }
+    Ok(0)
+}
+
+fn select_viewed_outfit(state: &mut LuaState, requested_id: f64) -> LuaResult<bool> {
+    let mut sim = borrow_state_mut(state)?;
+    let id = sim
+        .transmog_outfit_catalog
+        .entries
+        .iter()
+        .find(|entry| entry.outfit_id as f64 == requested_id)
+        .map(|entry| entry.outfit_id);
+    // INFERRED: catalog misses preserve state and emit no refresh. Like catalog
+    // lookup, first matching ID wins; isDisabled is not an eligibility policy.
+    let Some(id) = id else { return Ok(false) };
+    // INFERRED: changing viewed selection does not discard pending snapshots.
+    sim.viewed_transmog_outfit_id = Some(id);
+    Ok(true)
+}
--- /dev/null
+++ b/tests/cmd_option_selected_unit.rs
@@ -0,0 +1,98 @@
+#![cfg(feature = "retail-12-0-5")]
+
+use wow_ui_sim::lua_api::WowLuaEnv;
+
+#[test]
+fn cmd_option_returns_only_explicit_unit_from_the_selected_clause() {
+    let env = WowLuaEnv::new().unwrap();
+    env.exec(
+        r#"
+        local function check(text, expectedValue, expectedUnit)
+            local function inspect(...)
+                assert(select('#', ...) == (expectedValue == nil and 1 or 2))
+                local value, unit = ...
+                assert(value == expectedValue)
+                assert(unit == expectedUnit)
+            end
+            inspect(SecureCmdOptionParse(text))
+        end
+        check('plain', 'plain', nil)
+        check('[nocombat] 2', '2', nil)
+        check('[] 3', '3', nil)
+        check('[@focus] 4', '4', 'focus')
+        check('[target=focus] 5', '5', 'focus')
+        check('[unit=focus] 6', '6', 'focus')
+        check('[@target] 7', '7', 'target')
+        check('[@focus,combat] 1; fallback', 'fallback', nil)
+        check('[combat] 1; [@player,exists] 8', '8', 'player')
+        check('[@player,@focus] 1', '1', 'focus')
+        check('[unknown] 1', nil, nil)
+        check('', nil, nil)
+        "#,
+    )
+    .unwrap();
+    env.state().borrow_mut().player.in_combat = true;
+    env.exec(
+        r#"
+        local value, unit = SecureCmdOptionParse('[@focus,combat] 1; fallback')
+        assert(value == '1' and unit == 'focus')
+        "#,
+    )
+    .unwrap();
+}
+
+#[test]
+fn cached_vendor_target_marker_handler_uses_selected_unit_and_default_target() {
+    let env = WowLuaEnv::new().unwrap();
+    env.exec("TargetUnit('player')").unwrap();
+    {
+        let state = env.state();
+        let mut sim = state.borrow_mut();
+        let mut focus = sim.current_target.clone().unwrap();
+        focus.guid = "Creature-0-1-2-3-448-000002".into();
+        focus.name = "CmdOption focus fixture".into();
+        sim.current_focus = Some(focus);
+        // Enum.GameMode.Standard = 1; default mode 0 registers no vendor commands.
+        sim.game_rules.active_game_mode = 1;
+    }
+    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
+    let source =
+        std::fs::read_to_string(ui.join("Blizzard_ChatFrameBase/Shared/SlashCommands.lua"))
+            .expect("profile-scoped cached vendor SlashCommands.lua required");
+    // Supply the addon's normal chunk varargs; do not replace registration or handlers.
+    env.exec(&format!(
+        "local function loadVendor(...)\n{source}\nend\nloadVendor('Blizzard_ChatFrameBase', {{ SecureCmdList = {{}} }})"
+    ))
+    .expect("load entire unchanged cached SlashCommands.lua");
+    env.exec(
+        r#"
+        local handler = SlashCmdList[SLASH_COMMAND.TARGET_MARKER]
+        assert(type(handler) == 'function')
+        markerEvents = 0
+        local listener = CreateFrame('Frame')
+        listener:RegisterEvent('RAID_TARGET_UPDATE')
+        listener:SetScript('OnEvent', function(_, event)
+            assert(event == 'RAID_TARGET_UPDATE')
+            markerEvents = markerEvents + 1
+        end)
+        handler('[@focus,exists] 5')
+        assert(GetRaidTargetIndex('focus') == 5)
+        assert(GetRaidTargetIndex('target') == nil)
+        assert(markerEvents == 1)
+        handler('[target=focus,exists] 6')
+        assert(GetRaidTargetIndex('focus') == 6)
+        assert(GetRaidTargetIndex('target') == nil)
+        handler('[nocombat] 2')
+        assert(GetRaidTargetIndex('target') == 2)
+        assert(GetRaidTargetIndex('focus') == 6)
+        assert(markerEvents == 3)
+        handler('[@focus,combat] 7')
+        assert(GetRaidTargetIndex('focus') == 6 and markerEvents == 3)
+        handler('[@focus] 0')
+        assert(GetRaidTargetIndex('focus') == nil)
+        assert(GetRaidTargetIndex('target') == 2 and markerEvents == 4)
+        "#,
+    )
+    .unwrap();
+    assert!(env.state().borrow().lua_errors.is_empty());
+}
--- /dev/null
+++ b/tests/viewed_outfit_selection.rs
@@ -0,0 +1,165 @@
+#![cfg(feature = "retail-12-0-5")]
+
+use wow_ui_sim::c_api::c_transmog_outfit_info::{OutfitEntry, PendingTransmogCost};
+use wow_ui_sim::lua_api::WowLuaEnv;
+
+fn seeded_env() -> WowLuaEnv {
+    let env = WowLuaEnv::new().unwrap();
+    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![
+        OutfitEntry {
+            outfit_id: 91,
+            name: "Raid".into(),
+            situation_categories: vec![],
+            icon: 135_771,
+            is_event_outfit: false,
+            is_disabled: false,
+            player_facing_outfit_index: 1,
+        },
+        OutfitEntry {
+            outfit_id: 305,
+            name: "Travel".into(),
+            situation_categories: vec![],
+            icon: 132_489,
+            is_event_outfit: false,
+            is_disabled: false,
+            player_facing_outfit_index: 7,
+        },
+    ];
+    env
+}
+
+#[test]
+fn viewed_outfit_reads_host_state_not_writable_lua_storage_and_is_isolated() {
+    let first = seeded_env();
+    let second = seeded_env();
+    first
+        .exec(
+            r#"
+        local function inspect(...)
+            assert(select('#', ...) == 1)
+            local id = ...
+            assert(type(id) == 'number' and id == 0 and not issecretvalue(id))
+        end
+        inspect(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID())
+        C_TransmogOutfitInfo.__currentlyViewedOutfitID = 999
+        assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 0)
+        "#,
+        )
+        .unwrap();
+    first.state().borrow_mut().viewed_transmog_outfit_id = Some(305);
+    assert_eq!(
+        first
+            .eval::<f64>("return C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()")
+            .unwrap(),
+        305.0
+    );
+    assert_eq!(
+        second
+            .eval::<f64>("return C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()")
+            .unwrap(),
+        0.0
+    );
+    first
+        .exec("C_TransmogOutfitInfo.ChangeViewedOutfit(91)")
+        .unwrap();
+    assert_eq!(first.state().borrow().viewed_transmog_outfit_id, Some(91));
+    assert_eq!(second.state().borrow().viewed_transmog_outfit_id, None);
+}
+
+#[test]
+fn viewed_outfit_id_transition_is_synchronous_and_preserves_active_and_pending() {
+    let env = seeded_env();
+    env.state().borrow_mut().pending_transmog_cost = Some(PendingTransmogCost {
+        cost: 123_450,
+        modifier_flags: 14,
+    });
+    env.exec(
+        r#"
+        local api = C_TransmogOutfitInfo
+        api.ChangeToOutfit(7, false)
+        api.SetPendingTransmogSheatheCategory(16, 2, 2)
+        local pending = rawget(api, '__pendingSheatheCategories')
+        local events = {}
+        local listener = CreateFrame('Frame')
+        listener:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_CHANGED')
+        listener:RegisterEvent('TRANSMOG_DISPLAYED_OUTFIT_CHANGED')
+        listener:RegisterEvent('TRANSMOG_OUTFITS_CHANGED')
+        listener:SetScript('OnEvent', function(_, event, ...)
+            assert(event == 'VIEWED_TRANSMOG_OUTFIT_CHANGED')
+            assert(select('#', ...) == 0)
+            events[#events + 1] = api.GetCurrentlyViewedOutfitID()
+        end)
+        assert(select('#', api.ChangeViewedOutfit(91)) == 0)
+        assert(api.GetCurrentlyViewedOutfitID() == 91)
+        assert(#events == 1 and events[1] == 91)
+        assert(api.GetActiveOutfitID() == 305)
+        assert(rawget(api, '__pendingSheatheCategories') == pending and pending['16:2'] == 2)
+        assert(select('#', api.ChangeViewedOutfit(305)) == 0)
+        assert(#events == 2 and events[2] == 305)
+        -- INFERRED: valid repeated requests refresh, including initial UI selection.
+        api.ChangeViewedOutfit(305)
+        assert(#events == 3 and events[3] == 305)
+        for _, id in ipairs({1, 7, 999, -1, 0, 91.5, math.huge, 0/0}) do
+            assert(select('#', api.ChangeViewedOutfit(id)) == 0)
+            assert(api.GetCurrentlyViewedOutfitID() == 305 and #events == 3)
+        end
+        for _, value in ipairs({'91', false, {}}) do
+            assert(not pcall(api.ChangeViewedOutfit, value))
+            assert(api.GetCurrentlyViewedOutfitID() == 305 and #events == 3)
+        end
+        assert(not pcall(api.ChangeViewedOutfit))
+        api.ClearOutfit()
+        assert(api.GetActiveOutfitID() == 0 and api.GetCurrentlyViewedOutfitID() == 305)
+        assert(rawget(api, '__pendingSheatheCategories') == pending)
+        assert(#events == 3)
+        "#,
+    )
+    .unwrap();
+    assert_eq!(env.state().borrow().viewed_transmog_outfit_id, Some(305));
+    assert_eq!(
+        env.state().borrow().pending_transmog_cost,
+        Some(PendingTransmogCost {
+            cost: 123_450,
+            modifier_flags: 14,
+        })
+    );
+    let empty = WowLuaEnv::new().unwrap();
+    empty.exec("C_TransmogOutfitInfo.ChangeViewedOutfit(91); assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 0)").unwrap();
+}
+
+#[test]
+fn viewed_outfit_authenticates_secrets_before_mutation_and_preserves_caller_taint() {
+    let env = seeded_env();
+    env.exec(
+        r#"
+        local api = C_TransmogOutfitInfo
+        local id = secretwrap(91)
+        collectgarbage('collect')
+        api.ChangeViewedOutfit(id)
+        assert(api.GetCurrentlyViewedOutfitID() == 91)
+        local count = 0
+        local listener = CreateFrame('Frame')
+        listener:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_CHANGED')
+        listener:SetScript('OnEvent', function() count = count + 1 end)
+        local function denied()
+            api.ChangeViewedOutfit(secretwrap(305))
+        end
+        debug.setobjecttaint(denied, 'ViewedOutfitProbe')
+        assert(not pcall(denied))
+        assert(api.GetCurrentlyViewedOutfitID() == 91 and count == 0)
+        assert(not pcall(api.ChangeViewedOutfit, secretwrap('305')))
+        assert(api.GetCurrentlyViewedOutfitID() == 91 and count == 0)
+        local function ordinary()
+            assert(debug.getstacktaint() == 'ViewedOutfitProbe' and not issecure())
+            api.ChangeViewedOutfit(305)
+            assert(debug.getstacktaint() == 'ViewedOutfitProbe' and not issecure())
+        end
+        debug.setobjecttaint(ordinary, 'ViewedOutfitProbe')
+        ordinary()
+        assert(api.GetCurrentlyViewedOutfitID() == 305 and count == 1)
+        assert(issecure() and debug.getstacktaint() == nil)
+        "#,
+    )
+    .unwrap();
+    assert_eq!(env.state().borrow().viewed_transmog_outfit_id, Some(305));
+}
```
