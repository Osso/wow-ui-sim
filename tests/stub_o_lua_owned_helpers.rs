//! O retirement boundaries plus preservation through original Retail TOC consumers.
//! Boundary tests are retirement REDs; owner-loaded tests are preservation, not
//! evidence that the old fallback survived a successful original-owner load.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn seconds_formatter_has_no_synthetic_pre_owner_publication() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(rawget(_G, 'SecondsFormatter') == nil, 'synthetic SecondsFormatter before original owner')
        assert(rawget(_G, 'SecondsFormatterMixin') == nil, 'synthetic formatter mixin before original owner')
        assert(rawget(_G, 'SecondsFormatterConstants') == nil, 'synthetic formatter constants before original owner')
    "#).unwrap();
}

#[test]
fn static_model_info_has_no_synthetic_pre_owner_publication() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(rawget(_G, 'StaticModelInfo') == nil, 'synthetic StaticModelInfo before original owner')
    "#).unwrap();
}

#[test]
fn original_auction_consumer_preserves_owner_formatter_behavior() {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    // SharedXML.toc:26 owns TimeUtil; AuctionHouseUI.toc:9 mainline gate,
    // :11 selects the Shared consumer. Load the LoD addon explicitly.
    let owners = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXML"], &[]);
    assert!(owners.iter().any(|n| n == "Blizzard_SharedXML"));
    let consumers =
        load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_AuctionHouseUI"], &[]);
    assert!(consumers.iter().any(|n| n == "Blizzard_AuctionHouseUI"));
    env.exec(r#"
        local function original(fn, path)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        original(SecondsFormatterMixin.Init, 'Blizzard_SharedXML/TimeUtil.lua')
        original(SecondsFormatterMixin.Format, 'Blizzard_SharedXML/TimeUtil.lua')
        original(AuctionHouseUtil.FormatTimeLeft, 'Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUtil.lua')
        assert(SecondsFormatter.Abbreviation.None == 1 and SecondsFormatter.Interval.Minutes == 2)
        local f = CreateFromMixins(SecondsFormatterMixin)
        f:Init()
        assert(f:GetDesiredUnitCount(120) == 2 and f:GetStripIntervalWhitespace() == false)
        f:SetDesiredUnitCount(3)
        f:SetMinInterval(SecondsFormatter.Interval.Hours)
        assert(f:GetDesiredUnitCount(120) == 3 and f:GetMinInterval(120) == SecondsFormatter.Interval.Hours)
        local unset = CreateFromMixins(SecondsFormatterMixin)
        assert(unset:GetDefaultAbbreviation() == nil and unset:GetApproximationSeconds() == nil)
        assert(unset:CanRoundUpLastUnit() == nil and unset:CanRoundUpIntervals() == nil)
        -- Real row consumer rounds 90 seconds up to two minutes, one-letter
        -- localized form. Sold selects a known color without WoW-token inputs.
        local expected = MINUTE_ONELETTER_ABBR
        if TIME_UTIL_WHITE_SPACE_STRIPPABLE then expected = expected:gsub(' ', '') end
        expected = GRAY_FONT_COLOR:WrapTextInColorCode(expected:format(2))
        assert(AuctionHouseUtil.FormatTimeLeft(90, {status=Enum.AuctionStatus.Sold}) == expected)
        assert(AuctionHouseUtil.TimeLeftTooltipFormatter:GetDesiredUnitCount(90) == 2)
    "#).unwrap();
}

#[test]
fn original_azerite_consumer_preserves_static_actor_orchestration() {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    // SharedXMLGame.toc:4 Game, :12 mainline owner; AzeriteUI.toc:13-14
    // loads original mixin and XML. Actor doubles are observation scaffolding,
    // not invented native ModelScene backing data or a 3D-render claim.
    let owners = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXMLGame"], &[]);
    assert!(owners.iter().any(|n| n == "Blizzard_SharedXMLGame"));
    let consumers = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_AzeriteUI"], &[]);
    assert!(consumers.iter().any(|n| n == "Blizzard_AzeriteUI"));
    env.exec(r#"
        local function original(fn, path)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        original(StaticModelInfo.SetupModelScene, 'Blizzard_SharedXMLGame/StaticModelInfo.lua')
        original(AzeriteEmpoweredItemPowerMixin.SetupModelScene, 'Blizzard_AzeriteUI/AzeriteEmpoweredItemPowerMixin.lua')
        local entry = StaticModelInfo.CreateModelSceneEntry(55, 123, 456)
        assert(entry.modelSceneID == 55 and entry.effectFileID1 == 123 and entry.effectFileID2 == 456)
        assert(entry.sceneID == nil and entry.displayID == nil)
        local function scene()
            local actor = {
                SetModelByFileID = function(self, id) self.fileID = id end,
                SetAnimation = function(self, ...) self.animation = {...} end,
            }
            return {
                actor = actor,
                Hide = function(self) self.hidden = true end,
                SetFromModelSceneID = function(self, id, force) self.id=id; self.force=force end,
                GetActorByTag = function(self, tag) self.tag=tag; return self.actor end,
            }
        end
        local view = {ClickEffect=scene(), CanSelectEffect=scene()}
        -- Call original consumer, not a handwritten copy of SetupModelScene.
        AzeriteEmpoweredItemPowerMixin.SetupModelScene(view, true)
        assert(view.ClickEffect.hidden and view.CanSelectEffect.hidden)
        assert(view.ClickEffect.id == 219 and view.CanSelectEffect.id == 222)
        assert(view.ClickEffect.force == true and view.CanSelectEffect.force == true)
        assert(view.clickEffectActor == view.ClickEffect.actor)
        assert(view.ClickEffect.tag == 'effect' and view.CanSelectEffect.tag == 'effect')
        assert(view.ClickEffect.actor.fileID == 1983548 and view.CanSelectEffect.actor.fileID == 1983980)
        local animation = view.ClickEffect.actor.animation
        assert(#animation == 4)
        for i=1,4 do assert(animation[i] == 0) end
        assert(view.CanSelectEffect.actor.animation == nil)
    "#).unwrap();
}
