//! Pins that the original Blizzard tooltip setters keep running the cached Lua
//! `TooltipDataHandler` pipeline instead of the simulator's Rust content
//! setters in `src/lua_api/frame/methods/widgets/tooltip/content.rs`.
//!
//! WM63-D56:26-39 records that order as cached Lua source: getter, `ClearLines`,
//! info insertion, line insertion with line post-callbacks, info then global
//! tooltip post-callbacks, `Show`, backdrop style. A consumed tooltip
//! pre-callback prevents later processing and `Show` without undoing the
//! earlier clear.
//!
//! `TooltipDataHandler.lua:521-610` installs the accessors on
//! `TooltipDataHandlerMixin` (SetItemByID at :525, SetSpellByID at :560,
//! SetUnit at :582), `GameTooltip.lua:951` derives `GameTooltipDataMixin` from
//! it, and `GameTooltip.xml:4,249` mixes it into the real `GameTooltip`. The
//! mixin entries are copied onto the frame table (`helpers.rs:323-339`,
//! `helpers.rs:337-362`), so they shadow the Rust registrations that live only
//! in the frame metatable (`tooltip.rs:145-152`).
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

const PIPELINE_LUA: &str = r#"
    GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

    local entries = {}

    local function record(stage)
        entries[#entries + 1] = stage
    end

    local function index_of(stage, from)
        for i = from or 1, #entries do
            if entries[i] == stage then
                return i
            end
        end
        return nil
    end

    local function count_of(stage)
        local count = 0
        for _, entry in ipairs(entries) do
            if entry == stage then
                count = count + 1
            end
        end
        return count
    end

    local originalClearLines = GameTooltip.ClearLines
    GameTooltip.ClearLines = function(self)
        record('clear-lines')
        return originalClearLines(self)
    end

    local originalShow = GameTooltip.Show
    GameTooltip.Show = function(self)
        record('show')
        return originalShow(self)
    end

    TooltipDataProcessor.AddLinePostCall(TooltipDataProcessor.AllTypes, function(tooltip, lineData)
        assert(tooltip == GameTooltip, 'line post-callback must receive the tooltip')
        assert(lineData ~= nil, 'line post-callback must receive the published row')
        record('line-post')
    end)

    TooltipDataProcessor.AddTooltipPostCall(TooltipDataProcessor.AllTypes, function(tooltip, tooltipData)
        assert(tooltip == GameTooltip, 'tooltip post-callback must receive the tooltip')
        assert(tooltip:GetPrimaryTooltipData() == tooltipData,
            'tooltip post-callback must receive the published data table')
        record('tooltip-post')
    end)

    local function expect_pipeline(label, getterName, call)
        entries = {}
        assert(call() == true, label .. ' must report a processed tooltip')

        local info = GameTooltip:GetProcessingTooltipInfo()
        assert(info ~= nil, label .. ' left no processing info')
        assert(info.getterName == getterName,
            label .. ' ran getter ' .. tostring(info.getterName) .. ' instead of ' .. getterName)

        assert(entries[1] == 'clear-lines', label .. ' must clear the tooltip before inserting rows')
        assert(count_of('clear-lines') == 1, label .. ' must clear the tooltip exactly once')

        local firstLinePost = index_of('line-post')
        assert(firstLinePost ~= nil, label .. ' ran no line post-callback')
        assert(count_of('tooltip-post') == 1, label .. ' must run one tooltip post-callback')

        local tooltipPost = index_of('tooltip-post')
        assert(tooltipPost > firstLinePost, label .. ' ran the tooltip post-callback before its rows')

        local show = index_of('show')
        assert(show ~= nil, label .. ' never showed the tooltip')
        assert(show > tooltipPost, label .. ' showed the tooltip before its post-callbacks')
    end

    expect_pipeline('SetItemByID', 'GetItemByID', function()
        return GameTooltip:SetItemByID(6948)
    end)
    expect_pipeline('SetSpellByID', 'GetSpellByID', function()
        return GameTooltip:SetSpellByID(19750)
    end)
    expect_pipeline('SetUnit', 'GetUnit', function()
        return GameTooltip:SetUnit('player')
    end)

    -- A consumed tooltip pre-callback stops later processing and Show without
    -- undoing the earlier clear or info insertion (WM63-D56:36).
    TooltipDataProcessor.AddTooltipPreCall(TooltipDataProcessor.AllTypes, function()
        return true
    end)
    GameTooltip:Hide()
    assert(GameTooltip:SetSpellByID(19750) == false, 'a consumed pre-call must fail the setter')
    assert(not GameTooltip:IsShown(), 'a consumed pre-call must prevent Show')
    assert(GameTooltip:NumLines() == 0, 'the earlier clear must still have removed the rows')
    assert(GameTooltip:GetPrimaryTooltipInfo() ~= nil, 'the info insertion must survive the pre-call')
"#;

#[test]
fn original_tooltip_setters_run_the_cached_lua_processinfo_pipeline() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(PIPELINE_LUA).expect(
        "original SetItemByID / SetSpellByID / SetUnit must process through the cached \
         TooltipDataHandler pipeline",
    );
}
