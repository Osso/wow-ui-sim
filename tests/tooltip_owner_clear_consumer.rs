//! Owner clearing, visibility and notification order through the original
//! Blizzard consumers.
//!
//! WM63 O17 §7 and WM63 L4 (Retail 12.1.5 / 70688) contract:
//! - a same-owner `SetOwner` clears content without an explicit Hide;
//! - a different-owner `SetOwner` hides the tooltip first;
//! - the clear notification dispatches only when the prior line count was
//!   nonzero, after the line state resets and before the new owner is
//!   recorded.
//!
//! The native notification's public script name is inferred, not verified.
//! These tests therefore assert dispatch count and order, and only use
//! `OnTooltipCleared` as the name the pinned consumers themselves bind.
//!
//! Consumers under test are the cached original sources:
//! `Blizzard_GameTooltip/Mainline/GameTooltip.xml` (`OnHide` and
//! `OnTooltipCleared` scripts) and
//! `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua`
//! (`InternalProcessInfo`, failed lookup hides, later reprocessing).

#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
use tooltip_full_env_helpers::setup_full_env;

/// The pinned consumer TOC selects the Mainline body for Retail/PTR.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn assert_mainline_tooltip_consumer_selected() {
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_GameTooltip"))
        .expect("profile-selected Blizzard_GameTooltip TOC");
    let body = std::fs::read_to_string(&toc).unwrap();
    assert!(
        body.contains("AllowLoadGameType: mainline"),
        "runtime selection guard: {} must be Mainline-only",
        toc.display()
    );
    assert!(
        body.contains("Mainline\\GameTooltip.xml"),
        "runtime selection guard: {} must load Mainline/GameTooltip.xml",
        toc.display()
    );
}

/// Same-owner `SetOwner` clears without hiding, notifies once with the old
/// owner still installed, and hides the used pooled FontString.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn tooltip_owner_clear_same_owner_clears_without_hide() {
    let env = setup_full_env();
    assert_mainline_tooltip_consumer_selected();
    env.exec(
        r#"
        OwnerClearA = CreateFrame("Frame", "OwnerClearA", UIParent)
        OwnerClearB = CreateFrame("Frame", "OwnerClearB", UIParent)
        -- Consumer handlers stay bound; the recorders run after them.
        GameTooltip:HookScript("OnTooltipCleared", function(self)
            _G.owner_clear_events[#_G.owner_clear_events + 1] =
                self:GetOwner() == OwnerClearA and "cleared-a" or "cleared-other"
        end)
        GameTooltip:HookScript("OnHide", function()
            _G.owner_clear_events[#_G.owner_clear_events + 1] = "hidden"
        end)

        GameTooltip:SetOwner(OwnerClearA, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "First" },
        } } }))
        assert(GameTooltip:IsShown())
        assert(GameTooltip.infoList, "consumer keeps its infoList while shown")
        local used = GameTooltip:GetLeftLine(1)
        assert(used:IsShown() and used:GetText() == "First")
        local info_list = GameTooltip.infoList

        _G.owner_clear_events = {}
        GameTooltip:SetOwner(OwnerClearA, "ANCHOR_NONE")

        assert(#_G.owner_clear_events == 1, "same-owner clear notifies exactly once")
        assert(_G.owner_clear_events[1] == "cleared-a",
            "notification dispatches with the old owner still installed")
        assert(GameTooltip:IsShown(), "same-owner SetOwner must not hide")
        assert(GameTooltip:NumLines() == 0, "same-owner SetOwner clears the lines")
        assert(not used:IsShown(), "clear hides the used left FontString")
        assert(used:GetText() == nil or used:GetText() == "",
            "clear resets the used FontString text")
        assert(GameTooltip.infoList == info_list,
            "clear must not run the consumer OnHide cleanup")
        "#,
    )
    .expect("WM63 O17 §7 same-owner clear");
}

/// Different-owner `SetOwner` hides first: the clear notification and the
/// original `OnHide` cleanup run in that order, then the new owner is stored.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn tooltip_owner_clear_reassignment_hides_before_owner_swap() {
    let env = setup_full_env();
    assert_mainline_tooltip_consumer_selected();
    env.exec(
        r#"
        OwnerClearA = CreateFrame("Frame", "OwnerClearA", UIParent)
        OwnerClearB = CreateFrame("Frame", "OwnerClearB", UIParent)
        GameTooltip:HookScript("OnTooltipCleared", function(self)
            _G.owner_clear_events[#_G.owner_clear_events + 1] =
                self:GetOwner() == OwnerClearA and "cleared-a" or "cleared-other"
        end)
        GameTooltip:HookScript("OnHide", function()
            _G.owner_clear_events[#_G.owner_clear_events + 1] = "hidden"
        end)

        GameTooltip:SetOwner(OwnerClearA, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "First" },
        } } }))
        assert(GameTooltip:IsShown() and GameTooltip.infoList)

        _G.owner_clear_events = {}
        GameTooltip:SetOwner(OwnerClearB, "ANCHOR_NONE")

        assert(#_G.owner_clear_events == 2, "reassignment clears once, then hides")
        assert(_G.owner_clear_events[1] == "cleared-a",
            "clear notification precedes the hide")
        assert(_G.owner_clear_events[2] == "hidden")
        assert(not GameTooltip:IsShown(), "reassignment hides the tooltip")
        assert(GameTooltip:GetOwner() == OwnerClearB, "new owner is stored")
        assert(GameTooltip:NumLines() == 0)
        assert(GameTooltip.infoList == nil,
            "original GameTooltip_OnHide cleanup cleared the handler info")
        "#,
    )
    .expect("WM63 O17 §7 reassignment hide and notification order");
}

/// Empty clears do not notify, and the consumer reprocesses after the
/// failed-lookup Hide that `TooltipDataHandler.lua` performs.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn tooltip_owner_clear_failed_lookup_reprocessing_notifies_once() {
    let env = setup_full_env();
    assert_mainline_tooltip_consumer_selected();
    env.exec(
        r#"
        OwnerClearA = CreateFrame("Frame", "OwnerClearA", UIParent)
        OwnerClearB = CreateFrame("Frame", "OwnerClearB", UIParent)
        GameTooltip:HookScript("OnTooltipCleared", function(self)
            _G.owner_clear_events[#_G.owner_clear_events + 1] =
                self:GetOwner() == OwnerClearA and "cleared-a" or "cleared-other"
        end)
        GameTooltip:HookScript("OnHide", function()
            _G.owner_clear_events[#_G.owner_clear_events + 1] = "hidden"
        end)

        GameTooltip:SetOwner(OwnerClearA, "ANCHOR_NONE")
        _G.owner_clear_events = {}
        GameTooltip:ClearLines()
        assert(#_G.owner_clear_events == 0,
            "an empty clear does not dispatch the notification")

        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "Looted" },
        } } }))
        assert(GameTooltip:IsShown() and GameTooltip.infoList)

        _G.owner_clear_events = {}
        -- TooltipDataHandler.lua:264-270: a non-append lookup that yields no
        -- data hides the tooltip.
        assert((GameTooltip:ProcessInfo({
            getterFunction = function() return nil end,
        })) == false)

        assert(#_G.owner_clear_events == 2,
            "failed lookup clears its content once, then hides")
        assert(_G.owner_clear_events[1] == "cleared-a",
            "the hide clears before the consumer OnHide handler runs")
        assert(_G.owner_clear_events[2] == "hidden")
        assert(not GameTooltip:IsShown() and GameTooltip:NumLines() == 0)
        assert(GameTooltip.infoList == nil, "failed lookup clears handler info")
        assert(GameTooltip:GetOwner() == nil, "Hide releases the owner")

        -- TooltipDataHandler.lua:272-307: the next non-append info re-owns,
        -- clears and publishes fresh content.
        GameTooltip:SetOwner(OwnerClearB, "ANCHOR_NONE")
        _G.owner_clear_events = {}
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "Again" },
        } } }))
        assert(GameTooltip:IsShown() and GameTooltip:NumLines() == 1)
        assert(GameTooltip:GetLeftLine(1):GetText() == "Again")
        assert(GameTooltip:GetOwner() == OwnerClearB)
        "#,
    )
    .expect("TooltipDataHandler reprocessing after a failed lookup");
}

/// Forever keeps the inferred owner-reassignment visibility lifecycle: the
/// same owner keeps the tooltip shown and clears its lines, a different owner
/// hides through the ordinary lifecycle before recording the new owner.
///
/// The pooled FontString's own shown flag is not asserted: only Retail/PTR
/// (WM63 O17 §7) and Wrath reset it, and no Forever native evidence covers it.
#[cfg(feature = "client-wowforever")]
#[test]
fn tooltip_owner_clear_forever_owner_reassignment_lifecycle() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        local owner_a = CreateFrame("Frame", "OwnerClearA", UIParent)
        local owner_b = CreateFrame("Frame", "OwnerClearB", UIParent)
        local tip = GameTooltip
        assert(tip, "original GameTooltip must exist")
        tip:SetOwner(owner_a, "ANCHOR_NONE")
        tip:AddLine("First")
        tip:Show()
        assert(tip:IsShown() and tip:NumLines() == 1)
        local used = tip:GetLeftLine(1)

        tip:SetOwner(owner_a, "ANCHOR_NONE")
        assert(tip:IsShown(), "same owner retains visibility")
        assert(tip:NumLines() == 0, "same-owner SetOwner clears lines")
        assert(used:GetText() == nil, "cleared content drops the used FontString text")

        tip:AddLine("Second")
        tip:SetOwner(owner_b, "ANCHOR_NONE")
        assert(not tip:IsShown(), "different owner hides the tooltip")
        assert(tip:GetOwner() == owner_b, "new owner is stored")
        assert(tip:NumLines() == 0, "reassignment clears lines")
        "#,
    )
    .expect("Forever owner reassignment lifecycle");
}
