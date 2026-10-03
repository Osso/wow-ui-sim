# Failing integration tests: read-only investigation

Inspected revision: ced356e95d3b992734c9f97c7fdabd787f06197f 2026-10-03T17:35:23-05:00 Track 12.0.5 audit run logs (renamed past the *.log ignore rule)

No repository edits, Cargo, test execution, agents, or model CLIs. Runtime failures supplied by caller; conclusions below use source/history evidence.

## 1. Console catalog — stale test (confirmed)

`tests/c_system_api.rs:53–66` asserts zero entries. `src/lua_api/globals/missing_surface/small_namespaces.rs:180` routes to `src/c_api/c_console.rs:11–19`, which enumerates `cvars.all_keys()` into fresh records. `src/lua_api/globals/register.rs:155` publishes the legacy global to the same producer.

Introducing commit: **2c78bff733124940692cc8a3d8cf72df93a53576**, **2026-09-22 18:35:32 -05:00**, **Expose modeled CVar console command catalog**. Its patch removes the empty-table producer and wires the modeled producer. `docs/specs/console-command-catalog.md:7–11` requires built-in and runtime-registered records. Cached `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ConsoleDocumentation.lua:59–65` declares a nonnullable array of ConsoleCommandInfo, not an empty result; declaration establishes shape, not native count/completeness.

Not CASC/font/host-dependent. Count varies with profile/registrations; nonempty catalog is intentional. Smallest correct fix: delete the obsolete empty test; tests/console_commands.rs already covers built-in records and registration lifecycle. If retaining a local replacement is preferred, full replacement follows (use instead of, not in addition to, equivalent existing coverage); do not hardcode1647.

```rust
#[test]
fn test_c_console_get_all_commands_contains_registered_cvar() {
    let env = env();
    let found: bool = env
        .eval(
            r#"
            C_CVar.RegisterCVar("RootCauseConsoleFixture", "1")
            for _, record in ipairs(C_Console.GetAllCommands()) do
                if string.lower(record.command) == "rootcauseconsolefixture" then
                    return record.commandType == Enum.ConsoleCommandType.Cvar
                end
            end
            return false
            "#,
        )
        .unwrap();
    assert!(found);
}
```

## 2. Item context — exact failing assertion/introducing regression unresolved

The producer is not skipping secret authentication. `src/c_api/c_tooltip_info_item_context.rs:22–31,36–45` authenticates all four stack arguments with `unwrap_secret` before parsing or lookup, including ignored quality. Cached rilua `6044544/src/table_security.rs:232–255` rejects authentic wrappers with “requires an untainted caller”; `src/api.rs:403–412` checks every live call-frame taint. Invalid public item/type inputs therefore cannot bypass a later secret gate in this producer.

Producer introduced by **23efb40c84837dfa9564a7d21bc6297842ee4df3**, **2026-10-02 13:31:02 -05:00**, **Implement exact item tooltip context producer**. Test introduced by **11d21cd28b1d14339825bd14530c3051d7e08f2c**, **2026-10-02 13:16:11 -05:00**, **Scaffold item tooltip context inputs and behavioral tests**. **f97d2c8edde8987edc9b06329878b0f1aca6d174**, **2026-10-02 13:41:59 -05:00**, **Split item-context color and error test predicates**, and **a920153ef827bab420aba1c4facad994b35691ba**, **2026-10-02 13:48:29 -05:00**, **Split item-context DTO identity assertions**, only split assertions. Producer and test have no subsequent committed changes through inspected HEAD.

Cargo pin history does NOT identify a later runtime change: **c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd**, **2026-10-01 00:43:11 -05:00**, **Adopt opaque secret-string formatting runtime**, pins rilua `6044544b960cd68b4b0c58bb3373412757c2caee` before these tests were introduced; Cargo.toml is unchanged between a920153ef and HEAD. Previous pin **6eace49d59669849cb6af02372a43fbe7426a895**, **2026-09-30 18:59:32 -05:00**, **Mint modeled scenario secrets without changing addon caller taint**, pins e47c1fe from77e0f76. Neither is a proved introducing regression here.

Important contrary evidence: `docs/specs/tooltip-item-context.md`, “Independent bounded acceptance”, records24 focused PASS at a920153ef on October2. `/tmp/patch-12.0.5-item-tooltip-context-final-artifact-proof.json` binds that recorded binary to hash99af35e6ac927ebe49fad0e698c63972f6e6f70786dd66c0deaaaae7cf6aebee. This is historical artifact evidence, not a newly executed proof. October3 `data/patch-api/evidence/12.0.5-session-2026-10-03/b97-green.log.txt:1219–1233` records the failure with empty traceback. B97 commit message also records these tooltip failures with ClearLines changes reverted. Thus “pre-dates today's work” is not established for this test by the available history.

`tests/tooltip_item_context.rs:148–155` labels every rejection/type/error-prefix/private-leak/gate assertion. A bare “assertion failed!” is therefore not evidence that authentication or gate ordering failed. Unlabelled assertions remain in `ITTainted` at136–143, caller-taint equality at156, and recovery's ITCheck/ITProperties/ITRGBA helpers at43–134. The provided log does not distinguish them. **It is impossible to honestly select one exact assertion from this error and source alone.** No CASC/font-dependent operation is involved in this API test; process/GC/VM state or build provenance remains possible but unproved.

No correct production/test semantic fix is justified yet. Proposal only: replace the probe helper with this diagnostic version, preserving its assertions and wrapping the existing script in xpcall to retain Lua traceback. This is diagnostic code, NOT a claimed fix; do not relax security/caller-trust assertions or repin rilua without the resulting evidence.

```rust
fn probe(env: &WowLuaEnv, script: &str) {
    let levels = env.state().borrow().item_tooltip_levels.clone();
    let item = catalog();
    let player_class = env.state().borrow().player.class_index;
    let diagnostic = format!(
        r#"
        local ok, err = xpcall(function()
            {script}
        end, function(err)
            return debug.traceback(tostring(err), 2)
        end)
        if not ok then error(err, 0) end
        "#
    );
    env.exec(&diagnostic).expect("actual GetItemByID contract");
    assert_eq!(
        env.state().borrow().item_tooltip_levels,
        levels,
        "read-only host inputs"
    );
    assert_eq!(env.state().borrow().player.class_index, player_class);
    assert_catalog_unchanged(&item);
    env.exec("ITProperties()").unwrap();
}
```

## 3. Tooltip viewport — hidden tooltip uses oversized estimate (identified)

Root cause is fixture lifecycle, not missing CASC or screen defaults. `tests/tooltip_text_layout.rs:157–162` sets owner and adds text but never calls Show. `src/lua_api/frame/methods/widgets/tooltip/owner.rs:99` hides on SetOwner. `line_data.rs:53–86` adds lines and refreshes geometry WITHOUT showing the tooltip. `src/iced_app/tooltip.rs:71–80` skips invisible tooltips, so constructing WowFontSystem does not mean this tooltip gets glyph-backed measurement.

Stored geometry instead comes from `src/lua_api/frame/methods/widgets/tooltip/sizing.rs:9–15,40–54,85–86`: first-line width =51 chars ×14 ×0.55 +24 padding =**416.7**, exceeding the explicitly configured400 viewport. `src/layout.rs:652–663` clamps POSITION only: width>=400 sets x=0 and retains width416.7. Therefore right edge416.7 fails `tests/tooltip_text_layout.rs:180`. The top-left sibling only checks nonnegative x/y at221–222, so passes under the same oversized condition.

Introducing behavior change: **1e9674bcdb9f04b53bb9e30726ebaa2186d66fb6**, **2026-09-26 02:05:53 -05:00**, **Correct tooltip owner and content visibility lifecycle**, changes SetOwner from visible=true to false. The earlier estimate producer is **25795d2f8a25d283a394d3a660b019c4ba857e1c**, **2026-05-18 18:29:08 -05:00**, **Size tooltips when content changes**. The test and position-only oversize policy originate in **71370107e0f9a196d73bbbd9439ea4dfbc696379**, **2026-04-09 04:03:11 -05:00**, **Clamp tooltip layout to viewport**. No execution/bisection performed; September26 is the directly identified lifecycle boundary, not a newly measured first bad build.

Environment check disproves the suggested absent-install premise on this host: **/mnt/c/World of Warcraft/Data/data exists**, and is an asset-resolver discovery candidate. WOW_INSTALL_PATH/WOW_DATA_PATH/WOW_SIM_CASC are unset. `font.rs:401–415` logs casc=true for a requested mode; fallback happens only when no WoW families load. Font choice CAN affect a correctly shown tooltip's natural width, but cannot explain this hidden-tooltip416.7 estimate. Screen defaults likewise cannot explain it: test writes400x300.

Smallest semantic correction: explicitly Show before renderer sizing; additionally ensure the fixture fits before demanding all edges stay inside. Full replacement below derives sufficient viewport dimensions from measured geometry, avoiding arbitrary available-font assumptions while still anchoring past the right/bottom edges. Do not change production clamping to truncate tooltip dimensions.

```rust
#[test]
fn test_tooltip_layout_is_clamped_to_viewport_edges() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ClampOwner", UIParent)
        owner:SetSize(10, 10)
        owner:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:AddLine("A tooltip line wide enough to overflow the viewport")
        GameTooltip:AddLine("Second line to ensure some height")
        GameTooltip:Show()
        "#,
    )
    .unwrap();
    update_tooltip_sizes(&env);
    let (screen_width, screen_height) = {
        let mut state = env.state().borrow_mut();
        let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let frame = state.widgets.get(id).unwrap();
        assert!(frame.visible);
        assert!(frame.width > 0.0 && frame.height > 0.0);
        let width = 400.0_f32.max(frame.width + 50.0);
        let height = 300.0_f32.max(frame.height + 50.0);
        state.screen_width = width;
        state.screen_height = height;
        state.widgets.clear_all_layout_rects();
        (width, height)
    };
    env.exec(&format!(
        "ClampOwner:ClearAllPoints(); \
         ClampOwner:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', {}, {})",
        screen_width - 5.0,
        -(screen_height - 5.0),
    ))
    .unwrap();
    update_tooltip_layout(&env);
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let rect = state.widgets.get(id).unwrap().layout_rect.unwrap();
    assert!(rect.width < screen_width && rect.height < screen_height);
    assert!(rect.x >= 0.0 && rect.y >= 0.0);
    assert!((rect.x + rect.width - screen_width).abs() <= 0.1);
    assert!(rect.y + rect.height <= screen_height + 0.1);
}
```

## Proof limits

Source/history/artifact inspection only. No repo files changed; no Cargo/test binary/git mutation/model or agent invocation. Console diagnosis confirmed; hidden-tooltip estimate explains viewport failure deterministically; item-context exact assertion/root cause remains unresolved, and a semantic fix would be speculation. No blanket claim that all three failures pre-date October3 is justified.
