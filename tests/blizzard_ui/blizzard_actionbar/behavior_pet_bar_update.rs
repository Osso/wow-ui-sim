//! Behavior pin: seeding `state.pet_actions[i]` and firing `PET_BAR_UPDATE`
//! repopulates the matching `PetActionButton{i+1}` icon and tooltip name via
//! `GetPetActionInfo`, and also unhides `PetActionBar` itself once
//! `PetHasActionBar()` flips to true.
//!
//! Source contract (`Interface/BlizzardUI/Blizzard_ActionBar/`):
//!
//! 1. `PetActionBar` is the `<Frame name="PetActionBar">` declared in
//!    `Mainline/PetActionBar.xml:33`, parented to UIParent and starting
//!    `hidden="true"`. It inherits `EditModeActionBarTemplate` and mixes in
//!    `PetActionBarMixin`. `KeyValue numButtons = 10` at xml:44 selects the
//!    fanout used by `ActionBar_OnLoad`.
//!
//! 2. `ActionBar_OnLoad` (`Shared/ActionBar.lua:3-44`) creates `numButtons`
//!    child buttons. The naming branch at `ActionBar.lua:23-24`,
//!    `if self == PetActionBar then buttonName = "PetActionButton"..i`, gives
//!    them the `PetActionButton1`..`PetActionButton10` globals the test
//!    probes. Each button is a `CheckButton` of template
//!    `PetActionButtonTemplate` (PetActionBar.xml:3) which inherits
//!    `SmallActionButtonTemplate`, which inherits `ActionButtonTemplate`,
//!    which declares `<Texture name="$parentIcon" parentKey="icon" />` at
//!    `Mainline/ActionButtonTemplate.xml:23`. So `PetActionButtonN.icon` is a
//!    Texture child accessible via `:GetTexture()` and `:IsShown()`.
//!
//! 3. `PetActionBarMixin:OnLoad` (`Shared/PetActionBar.lua:49-68`) registers
//!    `PET_BAR_UPDATE` at lua:55 (along with `UNIT_PET`, `PET_UI_UPDATE`,
//!    `UNIT_FLAGS`, `PET_BAR_UPDATE_COOLDOWN`, etc.), then calls
//!    `self:Update()` and conditionally `self:Show()` if `PetHasActionBar()`
//!    returns true. With cold-start `state.pet_actions = vec![default; 10]`
//!    (every slot has `has_action = false` per `state.rs:219`), the cold
//!    `PetHasActionBar()` reading is false (`pet_bar.rs:196-203` iterates
//!    every slot and short-circuits on `slot.has_action`), so the cold
//!    `PetActionBar:IsShown()` reading stays at the `hidden="true"` default
//!    from xml:33.
//!
//! 4. `PetActionBarMixin:OnEvent` at `PetActionBar.lua:70-91` routes
//!    `PET_BAR_UPDATE` to `self:Update()` and `self:Show()` when both
//!    `PetHasActionBar()` and `UnitIsVisible("pet")` are true (lua:72-77).
//!    The simulator returns `UnitIsVisible("pet") = true` unconditionally
//!    (`src/lua_api/globals/group_queries.rs:887-903`), so the second gate is
//!    permanently open and the test only needs to flip `PetHasActionBar()`
//!    via state seeding to trigger the success branch.
//!
//! 5. `PetActionBarMixin:Update` (`PetActionBar.lua:119-175`) iterates
//!    `i = 1..NUM_PET_ACTION_SLOTS` (10), reads
//!    `(name, texture, isToken, isActive, autoCastAllowed, autoCastEnabled,
//!    spellID) = GetPetActionInfo(i)`, and:
//!    - lua:126-128 (non-token branch) calls
//!      `petActionIcon:SetTexture(texture)` and
//!      `petActionButton.tooltipName = name`.
//!    - lua:156-165 calls `petActionIcon:Show()` when `texture` is non-nil,
//!      `petActionIcon:Hide()` otherwise. Because the cold-state `icon`
//!      starts hidden (no texture), an empty slot stays hidden post-Update.
//!    - lua:170-173 `if not PetHasActionBar() then self:Hide() end` — the
//!      test only flips one slot's `has_action`, so `PetHasActionBar()`
//!      returns true in the post-seed call and the bar's tail-end Hide
//!      gate does not fire. (The bar's literal `:IsShown()` is downstream
//!      of `EditModeActionBarMixin`'s `ShowOverride`/`IsShownOverride`
//!      indirection at `ActionBar.lua:266-306` plus the simulator's
//!      EditMode `UpdateSystemSettingVisibleSetting` resolution at
//!      `EditModeSystemTemplates.lua:1128-1139`. That visibility-resolution
//!      path is a separate contract from the icon-repopulation pin and is
//!      not asserted here — the icon and tooltipName writes alone prove
//!      the event chain reached `Update`.)
//!
//! 6. `GetPetActionInfo(index)` (`src/lua_api/globals/pet_bar.rs:91-106`)
//!    returns the empty 9-tuple `(nil, nil, false, false, false, false, nil,
//!    false, false)` when the indexed slot is out of range or its
//!    `has_action` flag is false. When seeded with `has_action = true`, it
//!    returns the populated 9-tuple from `push_pet_action_info` at lua:53-67:
//!    `(name, texture, is_token, is_active, auto_cast_allowed,
//!    auto_cast_enabled, spell_id, false, passive)`. The test fixture sets
//!    `is_token = false` so Update takes the non-token branch and the icon
//!    SetTexture call uses the raw texture path (not a `_G[token_name]`
//!    indirection that token-style entries would).
//!
//! Why the test seeds via `state.pet_actions` rather than calling
//! `CastPetAction` or another Lua API: there is no Lua-side function that
//! creates a pet action slot. The simulator's mutator surface
//! (`CastPetAction`, `TogglePetAutocast`, `CancelPetPossess` at
//! `pet_bar.rs:159-193`) only flips flags on already-populated slots. The
//! canonical write seam for "the player has a pet, here's slot 1" is
//! `state.pet_actions[i]` filled from the simulator's pet/combat model
//! (which the test fixture stands in for). This mirrors the pattern used by
//! `behavior_stance_select.rs:166-184` for `state.shapeshift_forms`.
//!
//! Why the test seeds slot 0 (Lua `PetActionButton1`) and explicitly checks
//! slot index 2 (`PetActionButton2`) is NOT updated: the cross-isolation
//! check confirms `Update` reads `GetPetActionInfo` per index rather than
//! broadcast-applying a single value. With 10 buttons, only the seeded slot
//! should pick up the texture; the other 9 stay empty/hidden. A regression
//! that copied slot 1's data across all buttons (e.g., a typo assigning
//! `actionButtons[1]` instead of `actionButtons[i]`) would still pass slot 1's
//! checks but fail the slot 2 isolation check.
//!
//! Why the test does NOT call `PetActionBar:Update()` directly to bypass
//! `PET_BAR_UPDATE`: the goal is to pin the *event chain*, not just the
//! Update method. A direct call would prove `Update` works in isolation but
//! would not catch a regression where `PetActionBar` stops registering
//! `PET_BAR_UPDATE` (lua:55) or where `OnEvent` (lua:72) drops the event
//! routing to Update. Firing the event is what proves the OnLoad event
//! registration and the OnEvent dispatch are still wired.
//!
//! Why the test uses Growl (spell 2649, "Interface/Icons/Ability_Physical_Taunt")
//! as the fixture: Growl is a real hunter pet ability with a known spell id
//! and texture path. The choice doesn't materially affect the test — any
//! `(name, texture, spell_id)` triple would work — but a real pet ability
//! makes the test data plausible if a future regression makes the test
//! inspect `SPELL_DB` or any name-based lookup table.
//!
//! The test pins the following observations:
//!   1. **`PetActionBar`, `PetActionButton1`, `PetActionButton2`, and
//!      `PetActionButton1.icon` exist as globals/children after harness
//!      settle.** A nil reading means the XML didn't load (TOC walk
//!      regressed), the `if self == PetActionBar` naming branch at
//!      `ActionBar.lua:23-24` regressed, or the `parentKey="icon"` Texture
//!      from `ActionButtonTemplate.xml:23` failed to attach.
//!   2. **Cold-state `PetActionButton1.tooltipName` is nil/empty.** Pinned
//!      by the cold-state `state.pet_actions[0].has_action = false` →
//!      `GetPetActionInfo(1)` returns the empty 9-tuple → `Update`'s lua:128
//!      writes nil. A non-empty cold reading means the test fixture didn't
//!      establish a clean baseline.
//!   3. **After seeding `state.pet_actions[0]` and firing `PET_BAR_UPDATE`,
//!      `PetActionButton1.tooltipName == "Growl"`.** This is the most direct
//!      pin of the event chain: it proves `PET_BAR_UPDATE` reached
//!      `PetActionBar:OnEvent`, the `PET_BAR_UPDATE` arm at lua:72 ran
//!      `self:Update()`, the loop body at lua:121-128 read
//!      `GetPetActionInfo(1)` (which sees `state.pet_actions[0]`), and the
//!      non-token branch wrote `petActionButton.tooltipName = name`.
//!   4. **`PetActionButton1.icon:GetTexture()` returns the seeded path's
//!      numeric fileDataID and `GetTextureFilePath()` preserves the path.**
//!      Pinned by `petActionIcon:SetTexture(texture)` at lua:127. This
//!      observation also proves `petActionButton.icon` resolves to the
//!      Texture declared at `ActionButtonTemplate.xml:23` (parentKey="icon").
//!   5. **`PetActionButton2.tooltipName` is nil/falsy and
//!      `PetActionButton2.icon:IsShown()` is false.** This is the
//!      cross-isolation check: only the seeded slot updates; the other 9
//!      stay empty. A reading where slot 2 picked up slot 1's texture means
//!      the loop is broadcasting instead of indexing per-slot.
//!
//! Regression candidates the assertions catch:
//!   - `PetActionBarMixin:OnLoad` stops registering `PET_BAR_UPDATE`:
//!     observations 3 and 4 fail (no OnEvent listener picks up the fire),
//!     1 and 2 still pass.
//!   - `PetActionBarMixin:OnEvent` drops the `PET_BAR_UPDATE` arm or the
//!     `PetHasActionBar() and UnitIsVisible("pet")` gate inverts:
//!     observations 3 and 4 fail; the `else` arm runs `self:Hide()` and
//!     skips Update entirely.
//!   - `Update` skips the per-button SetTexture/tooltipName writes (e.g., a
//!     refactor moves them into a guarded branch that doesn't fire on cold
//!     buttons): observations 3 and 4 fail.
//!   - `GetPetActionInfo` regresses to always returning the empty tuple
//!     (e.g., the `has_action` gate inverts): observations 3 and 4 fail;
//!     the non-token branch's `petActionIcon:SetTexture(nil)` clears the
//!     icon.
//!   - The loop bound regresses (e.g., `1..1` instead of
//!     `1..NUM_PET_ACTION_SLOTS`): observation 5 may pass spuriously
//!     (button 2 stays empty) but observations 3 and 4 still pin slot 1,
//!     so the test still detects most loop regressions on the seeded slot.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

use wow_ui_sim::lua_api::state::PetActionSlot;

const ROOT: &str = "Blizzard_ActionBar";
const SEEDED_NAME: &str = "Growl";
const SEEDED_TEXTURE: &str = "Interface/Icons/Ability_Physical_Taunt";
const SEEDED_TEXTURE_FILE_DATA_ID: i64 = 132270;
const SEEDED_SPELL_ID: u32 = 2649;

#[test]
fn pet_bar_update_event_repopulates_button_icon_and_shows_bar_from_seeded_pet_actions() {
    wrath_action_case!(pet_update);

    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let cold_globals_exist: bool = env
            .eval(
                r#"
                return PetActionBar ~= nil
                    and PetActionButton1 ~= nil
                    and PetActionButton2 ~= nil
                    and PetActionButton1.icon ~= nil
                    and PetActionButton2.icon ~= nil
                "#,
            )
            .expect("pet bar global existence probe must run cleanly");
        assert!(
            cold_globals_exist,
            "After the startup-shape harness loads `{ROOT}`, `PetActionBar`, \
             `PetActionButton1`, `PetActionButton2`, and the `.icon` child \
             on each must exist. `PetActionBar` is the \
             `<Frame name=\"PetActionBar\">` declared at \
             Mainline/PetActionBar.xml:33. The buttons are created by \
             `ActionBar_OnLoad` (Shared/ActionBar.lua:3-44) via the \
             `if self == PetActionBar then buttonName = \"PetActionButton\"..i` \
             naming branch at lua:23-24. The `.icon` field is the \
             `<Texture parentKey=\"icon\">` declared at \
             Mainline/ActionButtonTemplate.xml:23 (inherited through \
             SmallActionButtonTemplate). A nil reading on any of them means \
             the XML didn't load, the naming branch regressed, or the \
             parentKey attachment regressed."
        );

        let cold_tooltip_clean: bool = env
            .eval(
                r#"
                return PetActionButton1.tooltipName == nil
                    or PetActionButton1.tooltipName == ""
                "#,
            )
            .expect("cold-state PetActionButton1.tooltipName probe must run cleanly");
        assert!(
            cold_tooltip_clean,
            "Cold-state `PetActionButton1.tooltipName` must be nil/empty. \
             `PetActionBarMixin:OnLoad` (PetActionBar.lua:63) calls \
             `self:Update()` once during the harness settle, but the \
             default `state.pet_actions` (state.rs:219) has every \
             `has_action = false`, so `GetPetActionInfo(1)` returns the \
             empty 9-tuple and `Update`'s non-token branch writes \
             `tooltipName = nil` (lua:128). A non-empty cold reading means \
             the harness is leaking pet-action data from somewhere — the \
             test can't observe the post-seed transition cleanly."
        );

        {
            let mut state = env.state().borrow_mut();
            state.pet_actions[0] = PetActionSlot {
                has_action: true,
                name: Some(SEEDED_NAME.to_string()),
                texture: Some(SEEDED_TEXTURE.to_string()),
                is_token: false,
                is_active: false,
                auto_cast_allowed: false,
                auto_cast_enabled: false,
                spell_id: Some(SEEDED_SPELL_ID),
                passive: false,
                cooldown: None,
                packed_action: 0,
            };
        }

        env.fire_event("PET_BAR_UPDATE")
            .expect("PET_BAR_UPDATE fire must dispatch cleanly to PetActionBar:OnEvent");

        let post_tooltip_name: Option<String> = env
            .eval("return PetActionButton1.tooltipName")
            .expect("PetActionButton1.tooltipName probe must run cleanly");
        assert_eq!(
            post_tooltip_name.as_deref(),
            Some(SEEDED_NAME),
            "After seeding `state.pet_actions[0]` (Lua slot 1) with \
             `name = \"{SEEDED_NAME}\"` and firing `PET_BAR_UPDATE`, \
             `PetActionButton1.tooltipName` must equal \"{SEEDED_NAME}\". \
             Pinned by `PetActionBarMixin:Update` at PetActionBar.lua:128 \
             (`petActionButton.tooltipName = name` in the non-token branch). \
             A wrong value means the event chain broke: either \
             `PetActionBar:OnLoad` stopped registering `PET_BAR_UPDATE` \
             (PetActionBar.lua:55), `OnEvent` dropped the PET_BAR_UPDATE arm \
             (lua:72), the `PetHasActionBar() and UnitIsVisible(\"pet\")` \
             gate (lua:73) inverted, or the per-button non-token write at \
             lua:128 regressed. Got: post_tooltip_name={post_tooltip_name:?}."
        );

        let (post_icon_file_data_id, post_icon_path): (i64, String) = env
            .eval(
                "return PetActionButton1.icon:GetTexture(), \
                 PetActionButton1.icon:GetTextureFilePath()",
            )
            .expect("PetActionButton1 icon identity probe must run cleanly");
        assert_eq!(
            post_icon_file_data_id, SEEDED_TEXTURE_FILE_DATA_ID,
            "After PET_BAR_UPDATE, `PetActionButton1.icon:GetTexture()` must \
             return the seeded texture's fileDataID. A mismatch means the \
             event chain broke or known texture identity stopped preserving \
             fileDataIDs. Got path: {post_icon_path:?}."
        );
        assert_eq!(
            post_icon_path, SEEDED_TEXTURE,
            "`PetActionButton1.icon:GetTextureFilePath()` must preserve the \
             path passed through `GetPetActionInfo` and `SetTexture`."
        );

        let other_slot_clean: bool = env
            .eval(
                r#"
                local empty_tooltip = (PetActionButton2.tooltipName == nil)
                    or (PetActionButton2.tooltipName == "")
                local icon_hidden = not PetActionButton2.icon:IsShown()
                return empty_tooltip and icon_hidden
                "#,
            )
            .expect("PetActionButton2 cross-isolation probe must run cleanly");
        assert!(
            other_slot_clean,
            "After PET_BAR_UPDATE, `PetActionButton2.tooltipName` must be \
             nil/empty and `PetActionButton2.icon:IsShown()` must be false. \
             The fixture only seeds `state.pet_actions[0]`; slot index 1 \
             (Lua slot 2) keeps its default `has_action = false`. \
             `GetPetActionInfo(2)` returns the empty 9-tuple \
             (pet_bar.rs:91-106 takes the `!slot.has_action` branch at \
             lua:102), so `Update`'s lua:128 sets `tooltipName = nil` and \
             lua:163-164 calls `petActionIcon:Hide()` because `texture` is \
             nil. A non-empty tooltipName or shown icon on button 2 means \
             the loop is broadcasting slot 1's data instead of indexing \
             per-slot, or `GetPetActionInfo` stopped gating on `has_action`."
        );
    });
    }
}
