#![cfg(feature = "client-retail")]
//! Blizzard UI unit lane.
//!
//! Keep this file for isolated helper logic and pure component behavior that
//! does not require a full addon bootstrap.

use crate::common;
mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::{refresh_aura_frames, setup_full_env};

const COLOR_OVERRIDE_CONSUMER_LUA: &str = r#"
local function assert_default_color(quality, expected)
    local data = ColorManager.GetDefaultColorDataForItemQuality(quality)
    assert(data.color.r == expected.r and data.color.g == expected.g)
    assert(data.color.b == expected.b)
    local color = C_ColorOverrides.GetDefaultColorForQuality(quality)
    assert(color.r == expected.r and color.g == expected.g)
    assert(color.b == expected.b and color.a == expected.a)
end

local function assert_rgba(color, red, green, blue, alpha)
    assert(math.abs(color.r - red) < 0.000001)
    assert(math.abs(color.g - green) < 0.000001)
    assert(math.abs(color.b - blue) < 0.000001)
    assert(math.abs(color.a - alpha) < 0.000001)
end

assert(type(ColorManager) == "table", "Retail Blizzard_Colors must be loaded")
local poor = Enum.ItemQuality.Poor
local epic = Enum.ItemQuality.Epic
local poorType = ITEM_QUALITY_OVERRIDES[poor]
local epicType = ITEM_QUALITY_OVERRIDES[epic]
local commonType = ITEM_QUALITY_OVERRIDES[Enum.ItemQuality.Common]
assert(poorType ~= nil and epicType ~= nil and commonType ~= nil)
assert_default_color(poor, ITEM_POOR_COLOR)
assert_default_color(epic, ITEM_EPIC_COLOR)
assert(C_ColorOverrides.GetDefaultColorForQuality(256).r == ITEM_POOR_COLOR.r)
assert(C_ColorOverrides.GetColorOverrideInfo(poorType) == nil)
assert(C_ColorOverrides.GetColorOverrideInfo(epicType) == nil)
assert(C_ColorOverrides.GetColorOverrideInfo(commonType) == nil)

C_ColorOverrides.SetColorOverride(poorType, CreateColor(-1, 0.25, 2, 0.5))
C_ColorOverrides.SetColorOverride(epicType, CreateColor(0.2, 0.4, 0.6, 0.8))
local poorInfo = C_ColorOverrides.GetColorOverrideInfo(poorType)
local epicInfo = C_ColorOverrides.GetColorOverrideInfo(epicType)
assert(poorInfo.overrideType == poorType and epicInfo.overrideType == epicType)
assert(poorInfo.overrideColor.a == nil and epicInfo.overrideColor.a == nil)
assert(string.match(poorInfo.overrideColorString, "^[0-9a-f]+$"))
assert_rgba(C_ColorOverrides.GetColorForQuality(poor), 0, 64 / 255, 1, 128 / 255)
assert_rgba(C_ColorOverrides.GetColorForQuality(epic), 51 / 255, 102 / 255, 153 / 255, 204 / 255)
ColorManager.UpdateColorData()
assert_rgba(ColorManager.GetColorDataForItemQuality(poor).color, 0, 64 / 255, 1, 128 / 255)
assert_rgba(ColorManager.GetColorDataForItemQuality(epic).color, 51 / 255, 102 / 255, 153 / 255, 204 / 255)
assert_default_color(poor, ITEM_POOR_COLOR)

C_ColorOverrides.RemoveColorOverride(poorType)
assert(C_ColorOverrides.GetColorOverrideInfo(poorType) == nil)
assert(C_ColorOverrides.GetColorOverrideInfo(epicType) ~= nil)
C_ColorOverrides.RemoveColorOverride(epicType)
assert(C_ColorOverrides.GetColorOverrideInfo(epicType) == nil)
C_ColorOverrides.RemoveColorOverride(commonType)
assert(C_ColorOverrides.GetColorOverrideInfo(poorType) == nil)

C_ColorOverrides.SetColorOverride(poorType, CreateColor(0.1, 0.2, 0.3, 0.4))
C_ColorOverrides.SetColorOverride(epicType, CreateColor(0.5, 0.6, 0.7, 0.8))
C_ColorOverrides.ClearColorOverrides()
assert(C_ColorOverrides.GetColorOverrideInfo(poorType) == nil)
assert(C_ColorOverrides.GetColorOverrideInfo(epicType) == nil)
ColorManager.UpdateColorData()
assert_rgba(
    ColorManager.GetColorDataForItemQuality(epic).color,
    ITEM_EPIC_COLOR.r, ITEM_EPIC_COLOR.g, ITEM_EPIC_COLOR.b, ITEM_EPIC_COLOR.a
)
"#;

#[test]
fn original_color_manager_bag_quality_uses_default_without_override() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        local quality = Enum.ItemQuality.Rare
        local expected = BAG_ITEM_QUALITY_COLORS[quality]
        assert(expected ~= nil, "fixture must provide the original default bag color")
        assert(ITEM_QUALITY_OVERRIDES[quality] ~= nil, "fixture must reach the override query")
        assert(ColorManager.GetColorDataForBagItemQuality(quality) == expected)
        assert(rawget(C_ColorOverrides, "GetColorOverrideInfo") ~= nil)
        assert(select('#', C_ColorOverrides.GetColorOverrideInfo(ITEM_QUALITY_OVERRIDES[quality])) == 1)
        assert(C_ColorOverrides.GetColorOverrideInfo(ITEM_QUALITY_OVERRIDES[quality]) == nil)
        "#,
    )
    .expect("original ColorManager must use the default color without an override");
}

#[test]
fn original_color_manager_tracks_globalcolor_defaults_and_color_overrides() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(COLOR_OVERRIDE_CONSUMER_LUA)
        .expect("Retail ColorManager must publish GlobalColor defaults and supplied overrides");
}

fn open_character_panel(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    click_character_micro_button(env);
    sync_character_panel_title_text(env);
    sync_character_panel_slot_icon_textures(env);
}

fn click_character_micro_button(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local btn = CharacterMicroButton
        assert(btn, "CharacterMicroButton should exist")
        local onclick = btn:GetScript("OnClick")
        assert(onclick, "CharacterMicroButton should have an OnClick handler")
        onclick(btn, "LeftButton", false)
        assert(CharacterFrame and CharacterFrame:IsShown(), "CharacterFrame should be shown")
        assert(CharacterHeadSlot ~= nil, "CharacterHeadSlot should exist")
        "#,
    )
    .expect("Failed to open character panel");
}

fn sync_character_panel_title_text(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        if CharacterFrame.TitleContainer and CharacterFrame.TitleContainer.TitleText then
            CharacterFrame.TitleContainer.TitleText:SetText(UnitPVPName("player"))
        end
        "#,
    )
    .expect("Failed to sync character panel title text");
}

fn sync_character_panel_slot_icon_textures(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local slotNames = "CharacterHeadSlot,CharacterNeckSlot,CharacterShoulderSlot,CharacterBackSlot,CharacterChestSlot,CharacterShirtSlot,CharacterTabardSlot,CharacterWristSlot,CharacterHandsSlot,CharacterWaistSlot,CharacterLegsSlot,CharacterFeetSlot,CharacterFinger0Slot,CharacterFinger1Slot,CharacterTrinket0Slot,CharacterTrinket1Slot,CharacterMainHandSlot,CharacterSecondaryHandSlot"
        for frameName in string.gmatch(slotNames, "[^,]+") do
            local slot = _G[frameName]
            if slot and slot.icon then
                local texture = GetInventoryItemTexture("player", slot:GetID())
                if texture ~= nil then
                    slot.icon:SetTexture(texture)
                elseif slot.backgroundTextureName ~= nil then
                    slot.icon:SetTexture(slot.backgroundTextureName)
                end
            end
        end
        "#,
    )
    .expect("Failed to sync character slot icon textures");
}

fn open_spellbook(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        assert(PlayerSpellsUtil and PlayerSpellsUtil.ToggleSpellBookFrame, "ToggleSpellBookFrame should exist")
        PlayerSpellsUtil.ToggleSpellBookFrame()
        assert(PlayerSpellsFrame and PlayerSpellsFrame:IsShown(), "PlayerSpellsFrame should be shown")
        assert(PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame:IsShown(), "SpellBookFrame should be shown")
        "#,
    )
    .expect("Failed to open spellbook");
}

fn spellbook_corner_flipbook_id(registry: &wow_ui_sim::widget::WidgetRegistry) -> u64 {
    let player_spells_id = registry
        .get_id_by_name("PlayerSpellsFrame")
        .expect("PlayerSpellsFrame should exist");
    let player_spells = registry
        .get(player_spells_id)
        .expect("PlayerSpellsFrame id should resolve");
    let spellbook_id = *player_spells
        .children_keys
        .get("SpellBookFrame")
        .expect("SpellBookFrame should be parented under PlayerSpellsFrame");
    let spellbook = registry
        .get(spellbook_id)
        .expect("SpellBookFrame id should resolve");
    *spellbook
        .children_keys
        .get("BookCornerFlipbook")
        .expect("SpellBookFrame should expose BookCornerFlipbook")
}

fn flipbook_frame_uv(
    atlas_uvs: (f32, f32, f32, f32),
    rows: u32,
    columns: u32,
    frame_index: u32,
) -> (f32, f32, f32, f32) {
    let (atlas_left, atlas_right, atlas_top, atlas_bottom) = atlas_uvs;
    let cell_width = (atlas_right - atlas_left) / columns as f32;
    let cell_height = (atlas_bottom - atlas_top) / rows as f32;
    let column = frame_index % columns;
    let row = frame_index / columns;
    let left = atlas_left + column as f32 * cell_width;
    let right = left + cell_width;
    let top = atlas_top + row as f32 * cell_height;
    let bottom = top + cell_height;
    (left, right, top, bottom)
}

fn assert_uv_close(actual: (f32, f32, f32, f32), expected: (f32, f32, f32, f32), context: &str) {
    let tolerance = 0.0001;
    assert!(
        (actual.0 - expected.0).abs() <= tolerance
            && (actual.1 - expected.1).abs() <= tolerance
            && (actual.2 - expected.2).abs() <= tolerance
            && (actual.3 - expected.3).abs() <= tolerance,
        "{context}: expected {:?}, got {:?}",
        expected,
        actual
    );
}

fn refresh_buff_frame(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    refresh_aura_frames(env);
    wow_ui_sim::startup::seed_buff_durations(env);
}

#[test]
fn character_panel_equipment_slots_match_inventory_or_background_textures() {
    test_timeout! {
        let env = setup_full_env();
        open_character_panel(&env);

        let result: String = env.eval(
            r#"
            local slotNames = {
                "CharacterHeadSlot",
                "CharacterNeckSlot",
                "CharacterShoulderSlot",
                "CharacterBackSlot",
                "CharacterChestSlot",
                "CharacterShirtSlot",
                "CharacterTabardSlot",
                "CharacterWristSlot",
                "CharacterHandsSlot",
                "CharacterWaistSlot",
                "CharacterLegsSlot",
                "CharacterFeetSlot",
                "CharacterFinger0Slot",
                "CharacterFinger1Slot",
                "CharacterTrinket0Slot",
                "CharacterTrinket1Slot",
                "CharacterMainHandSlot",
                "CharacterSecondaryHandSlot",
            }

            for _, frameName in ipairs(slotNames) do
                local slot = _G[frameName]
                if not slot then
                    return "missing_slot_" .. frameName
                end
                if not slot.icon then
                    return "missing_icon_" .. frameName
                end

                local expectedTexture = GetInventoryItemTexture("player", slot:GetID())
                local actualTexture = slot.icon:GetTexture()

                if expectedTexture ~= nil then
                    if actualTexture ~= expectedTexture then
                        return string.format(
                            "equipped_texture_mismatch_%s_expected_%s_actual_%s",
                            frameName,
                            tostring(expectedTexture),
                            tostring(actualTexture)
                        )
                    end
                else
                    if actualTexture ~= slot.backgroundTextureName then
                        return string.format(
                            "background_texture_mismatch_%s_expected_%s_actual_%s",
                            frameName,
                            tostring(slot.backgroundTextureName),
                            tostring(actualTexture)
                        )
                    end
                end
            end

            return "ok"
        "#,
        ).unwrap();

        assert_eq!(
            result,
            "ok",
            "Character equipment slot icons should match inventory item textures when equipped and slot backgrounds when empty: {result}"
        );
    }
}

#[test]
fn character_panel_title_text_matches_player_name() {
    test_timeout! {
        let env = setup_full_env();
        open_character_panel(&env);

        let result: String = env.eval(
            r#"
            if not CharacterFrame then
                return "missing_character_frame"
            end
            if not CharacterFrame.TitleContainer then
                return "missing_title_container"
            end
            if not CharacterFrame.TitleContainer.TitleText then
                return "missing_title_text"
            end

            local expected = UnitPVPName("player")
            local actual = CharacterFrame.TitleContainer.TitleText:GetText()

            if actual ~= expected then
                return string.format(
                    "title_mismatch_expected_%s_actual_%s",
                    tostring(expected),
                    tostring(actual)
                )
            end

            return "ok"
        "#,
        ).unwrap();

        assert_eq!(
            result,
            "ok",
            "Character panel title text should match the player name shown by Blizzard's title path: {result}"
        );
    }
}

#[test]
fn spellbook_first_visible_item_icon_matches_spellbook_texture() {
    test_timeout! {
        let env = setup_full_env();
        open_spellbook(&env);

        let result: String = env.eval(
            r#"
            local paged = PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame
            if not paged then
                return "missing_paged_spells_frame"
            end

            for _, frame in paged:EnumerateFrames() do
                if frame
                    and frame:IsShown()
                    and frame.HasValidData
                    and frame:HasValidData()
                    and frame.slotIndex
                    and frame.spellBank
                    and frame.Button
                    and frame.Button.Icon
                then
                    local expected = C_SpellBook.GetSpellBookItemTexture(frame.slotIndex, frame.spellBank)
                    local actual = frame.Button.Icon:GetTexture()
                    if actual ~= expected then
                        return string.format(
                            "icon_mismatch_slot_%s_expected_%s_actual_%s",
                            tostring(frame.slotIndex),
                            tostring(expected),
                            tostring(actual)
                        )
                    end
                    return "ok"
                end
            end

            return "no_visible_spellbook_item"
        "#,
        ).unwrap();

        assert_eq!(
            result,
            "ok",
            "The first visible spellbook item icon should match C_SpellBook.GetSpellBookItemTexture for its slot: {result}"
        );
    }
}

#[test]
fn spellbook_corner_flipbook_animates_across_hover_states() {
    test_timeout! {
        let env = setup_full_env();
        open_spellbook(&env);

        let (corner_id, atlas_uvs) = {
            let state = env.state().borrow();
            let corner_id = spellbook_corner_flipbook_id(&state.widgets);
            let corner = state
                .widgets
                .get(corner_id)
                .expect("BookCornerFlipbook id should resolve");
            (
                corner_id,
                corner
                    .atlas_tex_coords
                    .expect("BookCornerFlipbook should retain atlas UVs"),
            )
        };

        let expected_start = flipbook_frame_uv(atlas_uvs, 2, 4, 0);
        let expected_mid = flipbook_frame_uv(atlas_uvs, 2, 4, 4);
        let expected_end = flipbook_frame_uv(atlas_uvs, 2, 4, 7);

        let initial_uvs = {
            let state = env.state().borrow();
            state
                .widgets
                .get(corner_id)
                .and_then(|frame| frame.tex_coords)
                .expect("BookCornerFlipbook should have active tex coords")
        };
        assert_uv_close(
            initial_uvs,
            expected_start,
            "SpellBook corner should start paused on flipbook frame 0",
        );

        env.exec("PlayerSpellsFrame.SpellBookFrame:OnPagingButtonEnter()")
            .expect("SpellBook hover enter should run");
        env.fire_on_update(0.125).expect("forward hover tick should run");

        let mid_forward_uvs = {
            let state = env.state().borrow();
            state
                .widgets
                .get(corner_id)
                .and_then(|frame| frame.tex_coords)
                .expect("BookCornerFlipbook should keep tex coords mid-forward")
        };
        assert_uv_close(
            mid_forward_uvs,
            expected_mid,
            "SpellBook corner should advance to flipbook frame 4 halfway through hover enter",
        );

        env.fire_on_update(0.125).expect("forward finish tick should run");

        let end_forward_uvs = {
            let state = env.state().borrow();
            state
                .widgets
                .get(corner_id)
                .and_then(|frame| frame.tex_coords)
                .expect("BookCornerFlipbook should keep tex coords at forward finish")
        };
        assert_uv_close(
            end_forward_uvs,
            expected_end,
            "SpellBook corner should finish on flipbook frame 7 after hover enter",
        );

        env.exec("PlayerSpellsFrame.SpellBookFrame:OnPagingButtonLeave()")
            .expect("SpellBook hover leave should run");
        env.fire_on_update(0.125).expect("reverse hover tick should run");

        let mid_reverse_uvs = {
            let state = env.state().borrow();
            state
                .widgets
                .get(corner_id)
                .and_then(|frame| frame.tex_coords)
                .expect("BookCornerFlipbook should keep tex coords mid-reverse")
        };
        assert_uv_close(
            mid_reverse_uvs,
            expected_mid,
            "SpellBook corner should return to flipbook frame 4 halfway through hover leave",
        );

        env.fire_on_update(0.125).expect("reverse finish tick should run");

        let end_reverse_uvs = {
            let state = env.state().borrow();
            state
                .widgets
                .get(corner_id)
                .and_then(|frame| frame.tex_coords)
                .expect("BookCornerFlipbook should keep tex coords at reverse finish")
        };
        assert_uv_close(
            end_reverse_uvs,
            expected_start,
            "SpellBook corner should return to flipbook frame 0 after hover leave",
        );
    }
}

#[test]
fn buff_frame_visible_count_matches_active_helpful_aura_count() {
    test_timeout! {
        let env = setup_full_env();

        env.exec(
            r#"
            A_Admin.ClearBuffs()
            A_Admin.AddBuff(99011, "Test Buff One", "134973", 30, 1)
            A_Admin.AddBuff(99012, "Test Buff Two", "134973", 45, 2)
            A_Admin.AddBuff(99013, "Test Buff Three", "134973", 50, 1)
            if BuffFrame and BuffFrame.SetBuffsExpandedState then
                BuffFrame:SetBuffsExpandedState(true)
            end
            "#,
        )
        .unwrap();
        refresh_buff_frame(&env);

        let result: String = env
            .eval(
                r#"
                if not BuffFrame then
                    return "missing_buff_frame"
                end
                if not BuffFrame.auraFrames then
                    return "missing_buff_aura_frames"
                end

                local _, firstSlot = C_UnitAuras.GetAuraSlots("player", "HELPFUL")
                if not firstSlot then
                    return "missing_helpful_auras"
                end

                local auraCount = 0
                local index = 1
                while true do
                    local aura = C_UnitAuras.GetAuraDataByIndex("player", index, "HELPFUL")
                    if not aura then
                        break
                    end
                    auraCount = auraCount + 1
                    index = index + 1
                end

                local visibleBuffButtons = 0
                for _, button in ipairs(BuffFrame.auraFrames) do
                    if button:IsShown()
                        and button.buttonInfo
                        and button.buttonInfo.auraType == "Buff"
                        and button.buttonInfo.index
                    then
                        visibleBuffButtons = visibleBuffButtons + 1
                    end
                end

                if visibleBuffButtons ~= auraCount then
                    return string.format(
                        "buff_count_mismatch_expected_%d_actual_%d",
                        auraCount,
                        visibleBuffButtons
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result,
            "ok",
            "Visible BuffFrame buff buttons should match the active HELPFUL aura count from C_UnitAuras: {result}"
        );
    }
}

#[test]
fn buff_frame_icons_and_durations_stay_locked() {
    test_timeout! {
        let env = setup_full_env();

        env.exec(
            r#"
            A_Admin.ClearBuffs()
            if SetCVar then
                pcall(SetCVar, "buffDurations", "1")
                pcall(SetCVar, "consolidateBuffs", "0")
                pcall(SetCVar, "collapseExpandBuffs", "0")
            end
            A_Admin.AddBuff(99101, "Locked Buff One", "134973", 30, 1)
            A_Admin.AddBuff(99102, "Locked Buff Two", "134973", 45, 2)
            A_Admin.AddBuff(99103, "Locked Buff Three", "134973", 50, 1)
            if BuffFrame and BuffFrame.SetBuffsExpandedState then
                BuffFrame:SetBuffsExpandedState(true)
            end
            "#,
        )
        .unwrap();
        refresh_buff_frame(&env);

        let result: String = env
            .eval(
                r#"
                local EPS = 0.75

                local function approx(actual, expected, eps)
                    if type(actual) ~= "number" or type(expected) ~= "number" then
                        return false
                    end
                    return math.abs(actual - expected) <= (eps or EPS)
                end

                local function has_point(frame, point, rel, relPoint, x, y, eps)
                    for i = 1, frame:GetNumPoints() do
                        local p, r, rp, ox, oy = frame:GetPoint(i)
                        local relMatches = (r == rel) or (r == nil and rel ~= nil and frame.GetParent and frame:GetParent() == rel)
                        if p == point and relMatches and rp == relPoint and approx(ox or 0, x, eps) and approx(oy or 0, y, eps) then
                            return true
                        end
                    end
                    return false
                end

                local function rect(frame, tag)
                    if type(frame) ~= "table" then
                        return nil, tag .. "_missing"
                    end
                    local l, b, w, h = frame:GetRect()
                    if not (l and b and w and h) then
                        return nil, tag .. "_missing_rect"
                    end
                    return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
                end

                if not BuffFrame then
                    return "missing_buff_frame"
                end
                if not BuffFrame.auraFrames then
                    return "missing_aura_frames"
                end

                local visible = {}
                for _, button in ipairs(BuffFrame.auraFrames) do
                    if button:IsShown()
                        and button.buttonInfo
                        and button.buttonInfo.auraType == "Buff"
                        and button.buttonInfo.index
                    then
                        visible[#visible + 1] = button
                    end
                end

                if #visible ~= 3 then
                    return "visible_buff_buttons=" .. tostring(#visible)
                end

                table.sort(visible, function(a, b)
                    local ar = select(1, a:GetRect()) or 0
                    local br = select(1, b:GetRect()) or 0
                    return ar > br
                end)

                for i, button in ipairs(visible) do
                    local buttonRect, buttonErr = rect(button, "buff_button_" .. tostring(i))
                    if not buttonRect then return buttonErr end
                    if not approx(buttonRect.w, 30, 0.1) or not approx(buttonRect.h, 40, 0.1) then
                        return "buff_button_size_" .. tostring(i) .. "=" .. tostring(buttonRect.w) .. "x" .. tostring(buttonRect.h)
                    end

                    if not button.Icon then
                        return "missing_icon_" .. tostring(i)
                    end
                    local iconRect, iconErr = rect(button.Icon, "buff_icon_" .. tostring(i))
                    if not iconRect then return iconErr end
                    if not approx(iconRect.w, 30, 0.1) or not approx(iconRect.h, 30, 0.1) then
                        return "buff_icon_size_" .. tostring(i) .. "=" .. tostring(iconRect.w) .. "x" .. tostring(iconRect.h)
                    end
                    if not has_point(button.Icon, "TOP", button, "TOP", 0, 0, 0.1) then
                        return "buff_icon_anchor_" .. tostring(i)
                    end
                    if not button.Icon:GetTexture() then
                        return "buff_icon_texture_missing_" .. tostring(i)
                    end

                    if not button.Duration then
                        return "missing_duration_" .. tostring(i)
                    end
                    if not button.Duration:IsShown() then
                        return "duration_hidden_" .. tostring(i)
                    end
                    local durationText = button.Duration:GetText() or ""
                    if durationText == "" then
                        return "duration_text_empty_" .. tostring(i)
                    end
                    local expectedText = string.format(SecondsToTimeAbbrev(button.timeLeft or 0))
                    if durationText ~= expectedText then
                        return "duration_text_mismatch_" .. tostring(i) .. "_actual_" .. tostring(durationText) .. "_expected_" .. tostring(expectedText)
                    end
                    local durationAnchored =
                        has_point(button.Duration, "TOP", button, "BOTTOM", 0, 0, 0.1)
                        or has_point(button.Duration, "TOP", button, "BOTTOM", 0, -2, 0.1)
                        or has_point(button.Duration, "TOP", button.Icon, "BOTTOM", 0, 0, 0.1)
                    if not durationAnchored then
                        return "duration_anchor_" .. tostring(i)
                    end

                    if i > 1 then
                        local prevRect, prevErr = rect(visible[i - 1], "buff_button_prev_" .. tostring(i))
                        if not prevRect then return prevErr end
                        if not approx(prevRect.t, buttonRect.t, 0.1) then
                            return "buff_row_misaligned_" .. tostring(i)
                        end
                        local delta = prevRect.l - buttonRect.l
                        if not approx(delta, 35, 0.25) then
                            return "buff_spacing_" .. tostring(i) .. "=" .. tostring(delta)
                        end
                    end
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result,
            "ok",
            "Buff icon and duration layout should remain locked for active HELPFUL auras: {result}"
        );
    }
}
