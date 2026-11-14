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

#[cfg(all(feature = "client-wowforever", feature = "optional-tests"))]
use wow_ui_sim::lua_api::WowLuaEnv;

/// The profile-selected consumer TOC loads the tooltip XML body, and the same
/// file list selects it at runtime (Retail: `_Mainline` TOC; PTR: unified TOC).
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn assert_tooltip_consumer_selected() {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_GameTooltip"))
        .expect("profile-selected Blizzard_GameTooltip TOC");
    let body = std::fs::read_to_string(&toc).unwrap();
    let expected_toc = if ACTIVE == ClientProfile::Retail {
        "Blizzard_GameTooltip_Mainline.toc"
    } else {
        "Blizzard_GameTooltip.toc"
    };
    assert_eq!(
        toc.file_name().and_then(|name| name.to_str()),
        Some(expected_toc),
        "runtime selection guard"
    );
    assert!(
        body.contains("GameTooltip.xml"),
        "runtime selection guard: {} must load the tooltip XML",
        toc.display()
    );
}

/// The original XML-bound `OnHide` handler is the live script, so these tests
/// drive the cached consumer, not a substitute.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
const CONSUMER_BINDING_CHECK: &str = r#"
    local on_hide = GameTooltip:GetScript("OnHide")
    assert(type(on_hide) == "function", "original OnHide handler is bound")
    local source = debug.getinfo(on_hide, "S").source
    assert(source:find("GameTooltip", 1, true), source)
"#;

/// Same-owner `SetOwner` clears without hiding, notifies once with the old
/// owner still installed, and hides the used pooled FontString.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn tooltip_owner_clear_same_owner_clears_without_hide() {
    let env = setup_full_env();
    assert_tooltip_consumer_selected();
    env.exec(CONSUMER_BINDING_CHECK).unwrap();
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
        local inserted = CreateFrame("Frame", nil, UIParent)
        inserted:Show()
        GameTooltip.insertedFrames = { inserted }

        _G.owner_clear_events = {}
        GameTooltip:SetOwner(OwnerClearA, "ANCHOR_NONE")

        assert(GameTooltip.insertedFrames == nil,
            "original OnTooltipCleared cleanup clears inserted frames")
        assert(not inserted:IsShown(),
            "original OnTooltipCleared cleanup hides inserted frames")
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
    assert_tooltip_consumer_selected();
    env.exec(CONSUMER_BINDING_CHECK).unwrap();
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
    assert_tooltip_consumer_selected();
    env.exec(CONSUMER_BINDING_CHECK).unwrap();
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
/// The pooled line FontString is not asserted: Forever's line FontStrings come
/// from its tooltip XML, and only Wrath/Retail/PTR record them for clear-time
/// reset (no Forever native evidence covers that reset).
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

        tip:SetOwner(owner_a, "ANCHOR_NONE")
        assert(tip:IsShown(), "same owner retains visibility")
        assert(tip:NumLines() == 0, "same-owner SetOwner clears lines")
        assert(tip:GetLeftLine(1) == nil, "cleared tooltip exposes no line")

        tip:AddLine("Second")
        tip:SetOwner(owner_b, "ANCHOR_NONE")
        assert(not tip:IsShown(), "different owner hides the tooltip")
        assert(tip:GetOwner() == owner_b, "new owner is stored")
        assert(tip:NumLines() == 0, "reassignment clears lines")
        "#,
    )
    .expect("Forever owner reassignment lifecycle");
}

/// R88n.5: Forever ClearLines empties cached text for active rows, but it does
/// not visit unused cached rows that callers still hold by FontString reference.
#[cfg(all(feature = "client-wowforever", feature = "optional-tests"))]
#[test]
fn tooltip_clear_lines_forever_clears_only_active_cached_text() {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    env.exec(
        r#"
        local tip = GameTooltip
        tip:AddLine("first")
        tip:AddDoubleLine("second left", "second right")
        local first = tip:GetLeftLine(1)
        local second_left = tip:GetLeftLine(2)
        local second_right = tip:GetRightLine(2)
        local function empty(line)
            local text = line:GetText()
            return text == nil or text == ""
        end

        tip:ClearLines()
        assert(empty(first), "active left line text is cleared")
        assert(empty(second_left), "active second-row left text is cleared")
        assert(empty(second_right), "active second-row right text is cleared")

        tip:AddLine("only active row")
        local active = tip:GetLeftLine(1)
        second_left:SetText("inactive left")
        second_right:SetText("inactive right")
        tip:ClearLines()
        assert(empty(active), "the active row is cleared")
        assert(second_left:GetText() == "inactive left", "unused left cache entry is untouched")
        assert(second_right:GetText() == "inactive right", "unused right cache entry is untouched")
        "#,
    )
    .expect("Forever ClearLines active cached text contract");
}

/// PRESERVATION: the profile-selected TooltipDataHandler still clears old
/// active rows and publishes the replacement through its original ProcessInfo.
#[cfg(all(feature = "client-wowforever", feature = "optional-tests"))]
#[test]
fn tooltip_clear_lines_forever_preserves_original_processor_reprocessing() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let toc = wow_ui_sim::loader::find_toc_file(&addons.join("Blizzard_SharedXMLGame"))
        .expect("profile-selected Blizzard_SharedXMLGame TOC");
    let toc_body = std::fs::read_to_string(&toc).unwrap();
    assert!(toc_body.contains("## AllowLoad: Game"), "selected Game TOC");
    assert!(
        toc_body.contains("Tooltip\\TooltipDataHandler.lua"),
        "selected TOC loads TooltipDataHandler.lua: {}",
        toc.display()
    );

    env.exec(
        r#"
        assert(TooltipDataHandlerMixin and type(TooltipDataHandlerMixin.ProcessInfo) == "function",
            "original TooltipDataHandler mixin is loaded")
        assert(GameTooltip.ProcessInfo == TooltipDataHandlerMixin.ProcessInfo,
            "GameTooltip uses the original ProcessInfo method")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "old first" },
            { leftText = "old second" },
        } } }))
        local cached_first = GameTooltip:GetLeftLine(1)
        assert(cached_first:GetText() == "old first")

        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { leftText = "replacement" },
        } } }))
        assert(GameTooltip:NumLines() == 1, "original processor replaces the old rows")
        assert(GameTooltip:GetLeftLine(1) == cached_first, "active FontString remains pooled")
        assert(cached_first:GetText() == "replacement", "replacement content is visible")
        "#,
    )
    .expect("original Forever TooltipDataHandler reprocessing");
}
