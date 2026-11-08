//! Behavior pin: the search-filter routing chain in
//! `Blizzard_AchievementUI.lua`.
//!
//! Two distinct edges are pinned across two tests so that the
//! query→filter→data-provider pipeline can be read top-to-bottom and so
//! each test body stays comfortably under the readability budget.
//!
//! 1. **Typing edge** — see
//!    `search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length`.
//!    `AchievementFrameSearchBox_OnTextChanged` (lua:3377-3390) gates on
//!    `MIN_CHARACTER_SEARCH = 3` (declared in `Constants.lua:296`). Below
//!    threshold the handler short-circuits to
//!    `AchievementFrame_HideSearchPreview()` and never touches
//!    `SetAchievementSearchString`. At/above threshold the handler calls
//!    `SetAchievementSearchString(self:GetText())`, stores the boolean
//!    result on `self.fullSearchFinished`, and routes to one of two
//!    forwards — `_UpdateSearchPreview` when the result is falsy,
//!    `_ShowSearchPreviewResults` when truthy. The simulator's
//!    `set_achievement_search_string`
//!    (`src/lua_api/globals/missing_surface/achievement_info.rs:847-859`)
//!    always returns `true`, so production traffic always takes the
//!    show-results branch; the test stubs the global to assert that the
//!    branch taken matches the returned value.
//!
//! 2. **Full-results population + per-row init** — see
//!    `full_search_results_chain_uses_filtered_globals_for_data_provider_and_per_row_init`.
//!    `AchievementFrame_UpdateFullSearchResults` (lua:3563-3570) reads
//!    `GetNumFilteredAchievements()`, wraps the count via
//!    `CreateDataProviderByIndexCount(numResults)`, and forwards the
//!    resulting provider to `AchievementFrame.SearchResults.ScrollBox:
//!    SetDataProvider`. The title text is rebuilt by formatting
//!    `ENCOUNTER_JOURNAL_SEARCH_RESULTS` with the live `SearchBox` query
//!    and the count. The XML widget for `SearchResults` is declared at
//!    xml:2567; the inner `ScrollBox` (xml:2673,
//!    `inherits="WowScrollBoxList"`) is the surface that owns the
//!    provider. `AchievementFullSearchResultsButtonMixin:Init`
//!    (lua:3394-3419) reads the index off `elementData.index`, calls
//!    `GetFilteredAchievementID(index)` to translate it into an
//!    achievement id, and writes the achievement metadata into the
//!    button's `Name`/`Icon`/`ResultType`/`Path` widgets. The mixin is
//!    bound to the row template via the
//!    `AchievementFullSearchResultsButtonTemplate` button (xml:62) which
//!    `AchievementFrameSearchBoxContainer_OnLoad` (lua:3421-3428) wires
//!    into the ScrollBox view via `SetElementInitializer`.
//!
//! **PLAN-named tripwire.** PLAN refers to `AchievementFullSearchResults`
//! as if it were a frame, but no such global exists. The actual
//! full-results frame is `AchievementFrame.SearchResults` (xml:2567), the
//! per-row mixin is `AchievementFullSearchResultsButtonMixin`, and the
//! button template is `AchievementFullSearchResultsButtonTemplate`. A
//! `_G.AchievementFullSearchResults` lookup MUST stay nil — if a future
//! refactor introduces a global with that name (e.g. by renaming the
//! frame), this assertion fails loud and forces a documentation update.
//!
//! **`depends-on: achievement search globals gap` is stale.** All four
//! search globals (`SetAchievementSearchString`,
//! `GetAchievementSearchProgress`, `GetAchievementSearchSize`,
//! `GetNumFilteredAchievements`, `GetFilteredAchievementID`) are already
//! wired by `register_search_setter` and `register_search_getters` at
//! `achievement_info.rs:257-292`. The same observation was made on Task
//! 66 (`behavior_search_progress.rs`).

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AchievementUI";
const PLAN_REFERENCED_GLOBAL: &str = "AchievementFullSearchResults";
const STUB_FILTERED_ID: i64 = 9001;
const STUB_ROW_INDEX: i64 = 5;
const STUB_NUM_FILTERED: i64 = 7;
const QUERY_AT_THRESHOLD: &str = "abc";
const QUERY_BELOW_THRESHOLD: &str = "ab";
const EXPECTED_BELOW_THRESHOLD: &str = "set_called=0 hide_called=1 update_called=0 show_called=0";
const EXPECTED_AT_THRESHOLD: &str = "set_called=1 query=abc finished=true \
    hide_called=0 update_called=0 show_called=1";
const EXPECTED_FULL_RESULTS: &str =
    "num_called=1 create_count=7 set_provider_called=1 provider=stub title_set=1";
const EXPECTED_BUTTON_INIT: &str = "id_called=1 id_index=5 achievement_id=9001";

type TextChangedProbe = (String, String, String, String);
type FullResultsProbe = (String, String, String, String);

#[test]
fn search_box_text_changed_calls_set_achievement_search_string_when_query_meets_min_length() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: TextChangedProbe = env
            .eval(
                r#"
                assert(AchievementFrame, "AchievementFrame must exist after addon load")
                assert(AchievementFrame.HeaderDetails.Filters.SearchBox,
                    "AchievementFrame.HeaderDetails.Filters.SearchBox must exist (xml:1709)")

                local search_box = AchievementFrame.HeaderDetails.Filters.SearchBox
                local original_text_changed = search_box:GetScript("OnTextChanged")
                -- Drive the handler explicitly; SetText must not also invoke the XML binding.
                search_box:SetScript("OnTextChanged", nil)

                local original_template = _G.SearchBoxTemplate_OnTextChanged
                _G.SearchBoxTemplate_OnTextChanged = function(self) end

                local routing = {hide = 0, update = 0, show = 0}
                local original_hide = _G.AchievementFrame_HideSearchPreview
                local original_update = _G.AchievementFrame_UpdateSearchPreview
                local original_show = _G.AchievementFrame_ShowSearchPreviewResults
                _G.AchievementFrame_HideSearchPreview =
                    function() routing.hide = routing.hide + 1 end
                _G.AchievementFrame_UpdateSearchPreview =
                    function() routing.update = routing.update + 1 end
                _G.AchievementFrame_ShowSearchPreviewResults =
                    function() routing.show = routing.show + 1 end

                local set_state = {calls = {}, return_finished = true}
                local original_set = _G.SetAchievementSearchString
                _G.SetAchievementSearchString = function(query)
                    set_state.calls[#set_state.calls + 1] = query
                    return set_state.return_finished
                end

                search_box:SetText("ab")
                AchievementFrameSearchBox_OnTextChanged(search_box)
                local below_threshold_signature = string.format(
                    "set_called=%d hide_called=%d update_called=%d show_called=%d",
                    #set_state.calls, routing.hide, routing.update, routing.show)

                routing.hide, routing.update, routing.show = 0, 0, 0
                set_state.calls = {}
                set_state.return_finished = true
                search_box:SetText("abc")
                AchievementFrameSearchBox_OnTextChanged(search_box)
                local at_threshold_signature = string.format(
                    "set_called=%d query=%s finished=%s hide_called=%d update_called=%d show_called=%d",
                    #set_state.calls, tostring(set_state.calls[1]),
                    tostring(search_box.fullSearchFinished),
                    routing.hide, routing.update, routing.show)

                _G.SearchBoxTemplate_OnTextChanged = original_template
                _G.AchievementFrame_HideSearchPreview = original_hide
                _G.AchievementFrame_UpdateSearchPreview = original_update
                _G.AchievementFrame_ShowSearchPreviewResults = original_show
                _G.SetAchievementSearchString = original_set
                search_box:SetScript("OnTextChanged", original_text_changed)

                return type(_G.AchievementFrameSearchBox_OnTextChanged),
                       type(_G.AchievementFullSearchResults),
                       below_threshold_signature,
                       at_threshold_signature
                "#,
            )
            .expect("setup + explicit handler drives must run cleanly");

        let (
            text_changed_type,
            plan_global_type,
            below_threshold_signature,
            at_threshold_signature,
        ) = observations;

        assert_eq!(
            text_changed_type, "function",
            "Expected `_G.AchievementFrameSearchBox_OnTextChanged` to be a function (declared at \
             `Mainline/Blizzard_AchievementUI.lua:3377`). Got `{text_changed_type}`. A `nil` \
             reading means the `<OnTextChanged>` script binding at xml:2365 would resolve to nil \
             at install time, leaving the search box mute — every keystroke would discard the \
             query without ever reaching `SetAchievementSearchString`."
        );

        assert_eq!(
            plan_global_type, "nil",
            "Expected `_G.{PLAN_REFERENCED_GLOBAL}` to be nil — PLAN refers to it as if it were \
             a frame, but no such global exists in `Blizzard_AchievementUI.lua`. The actual \
             full-results frame is `AchievementFrame.SearchResults` (xml:2567), the row mixin \
             is `AchievementFullSearchResultsButtonMixin` (lua:3392), and the button template \
             is `AchievementFullSearchResultsButtonTemplate` (xml:62). Got \
             `{plan_global_type}` — a non-nil reading means a future refactor introduced an \
             alias under the PLAN-named symbol; flag the rename and update PLAN wording \
             rather than silently assuming this assertion can be deleted."
        );

        assert_eq!(
            below_threshold_signature, EXPECTED_BELOW_THRESHOLD,
            "Expected below-threshold drive (text=`{QUERY_BELOW_THRESHOLD}`, 2 chars < \
             MIN_CHARACTER_SEARCH=3) to produce signature `{EXPECTED_BELOW_THRESHOLD}`. Got \
             `{below_threshold_signature}`. A `set_called` > 0 means the gate at lua:3380 \
             (`if strlen(self:GetText()) >= MIN_CHARACTER_SEARCH then`) leaks — every \
             keystroke would hit `SetAchievementSearchString`, generating filter work for \
             every partial input. A `hide_called=0` reading means the else branch at lua:3388 \
             (`AchievementFrame_HideSearchPreview()`) was severed — the preview panel would \
             stay open with stale data after the user backspaces below threshold."
        );

        assert_eq!(
            at_threshold_signature, EXPECTED_AT_THRESHOLD,
            "Expected at-threshold drive (text=`{QUERY_AT_THRESHOLD}`, 3 chars == \
             MIN_CHARACTER_SEARCH, stub returning true) to produce signature \
             `{EXPECTED_AT_THRESHOLD}`. Got `{at_threshold_signature}`. A `query=` other than \
             `abc` means the handler at lua:3381 (`SetAchievementSearchString(self:GetText())`) \
             does NOT forward the live edit-box text — likely a stale capture or an injected \
             constant. A `finished=` other than `true` means \
             `self.fullSearchFinished = SetAchievementSearchString(...)` at lua:3381 dropped \
             the return value; the next key press would re-evaluate the gate from scratch. A \
             `show_called=0 update_called=1` reading means the truthy branch at lua:3385 was \
             inverted — `_UpdateSearchPreview` ran instead of `_ShowSearchPreviewResults`, \
             which would surface as the preview list flashing instead of the final results \
             showing."
        );
    });
}

#[test]
fn full_search_results_chain_uses_filtered_globals_for_data_provider_and_per_row_init() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let observations: FullResultsProbe = env
            .eval(
                r#"
                assert(AchievementFrame.SearchResults,
                    "AchievementFrame.SearchResults must exist (xml:2567)")
                assert(AchievementFrame.SearchResults.ScrollBox,
                    "AchievementFrame.SearchResults.ScrollBox must exist (xml:2673)")
                assert(AchievementFrame.SearchResults.TitleText,
                    "AchievementFrame.SearchResults.TitleText must exist (xml:2583)")

                local search_results = AchievementFrame.SearchResults

                local stub_provider = {__stub_provider = true}
                local create_calls = {counts = {}}
                local original_create = _G.CreateDataProviderByIndexCount
                _G.CreateDataProviderByIndexCount = function(count)
                    create_calls.counts[#create_calls.counts + 1] = count
                    return stub_provider
                end

                local num_filter_calls = 0
                local original_num = _G.GetNumFilteredAchievements
                _G.GetNumFilteredAchievements = function()
                    num_filter_calls = num_filter_calls + 1
                    return 7
                end

                local set_provider_calls = {providers = {}}
                local original_set_provider = search_results.ScrollBox.SetDataProvider
                search_results.ScrollBox.SetDataProvider = function(self, provider)
                    set_provider_calls.providers[#set_provider_calls.providers + 1] = provider
                end

                local title_set_count = 0
                local original_title_set = search_results.TitleText.SetText
                search_results.TitleText.SetText = function(self, text)
                    title_set_count = title_set_count + 1
                end

                AchievementFrame_UpdateFullSearchResults()

                local first_provider = set_provider_calls.providers[1]
                local provider_marker =
                    (first_provider and first_provider.__stub_provider) and "stub" or "other"
                local full_results_signature = string.format(
                    "num_called=%d create_count=%d set_provider_called=%d provider=%s title_set=%d",
                    num_filter_calls, create_calls.counts[1] or -1,
                    #set_provider_calls.providers, provider_marker, title_set_count)

                local id_calls = {indices = {}}
                local original_id = _G.GetFilteredAchievementID
                _G.GetFilteredAchievementID = function(index)
                    id_calls.indices[#id_calls.indices + 1] = index
                    return 9001
                end

                local original_info = _G.GetAchievementInfo
                _G.GetAchievementInfo = function(id)
                    return id, "stub_name", 0, false, 0, 0, 0,
                        "stub_desc", 0, "Interface\\Icons\\stub", 0, false, false, false
                end
                local original_category = _G.GetAchievementCategory
                _G.GetAchievementCategory = function(id) return 1 end
                local original_category_info = _G.GetCategoryInfo
                _G.GetCategoryInfo = function(id) return "StubCategory", -1 end

                local fake_button = {
                    Name = {SetText = function(self, text) end},
                    Icon = {SetTexture = function(self, tex) end},
                    ResultType = {SetText = function(self, text) end},
                    Path = {SetText = function(self, text) end},
                }
                AchievementFullSearchResultsButtonMixin.Init(fake_button, {index = 5})

                local button_init_signature = string.format(
                    "id_called=%d id_index=%s achievement_id=%s",
                    #id_calls.indices,
                    tostring(id_calls.indices[1]),
                    tostring(fake_button.achievementID))

                _G.CreateDataProviderByIndexCount = original_create
                _G.GetNumFilteredAchievements = original_num
                search_results.ScrollBox.SetDataProvider = original_set_provider
                search_results.TitleText.SetText = original_title_set
                _G.GetFilteredAchievementID = original_id
                _G.GetAchievementInfo = original_info
                _G.GetAchievementCategory = original_category
                _G.GetCategoryInfo = original_category_info

                return type(_G.AchievementFrame_UpdateFullSearchResults),
                       type(_G.AchievementFullSearchResultsButtonMixin),
                       full_results_signature,
                       button_init_signature
                "#,
            )
            .expect("setup + update-full + button-init must run cleanly");

        let (
            update_full_results_type,
            button_mixin_type,
            full_results_signature,
            button_init_signature,
        ) = observations;

        assert_eq!(
            update_full_results_type, "function",
            "Expected `_G.AchievementFrame_UpdateFullSearchResults` to be a function (declared \
             at lua:3563). Got `{update_full_results_type}`. A `nil` reading means \
             `AchievementFrame_ShowFullSearch` at lua:3526 and `AchievementFrame_UpdateSearch` \
             at lua:3615 would both crash, breaking the path between pressing Enter on a query \
             and seeing the full results panel populated."
        );

        assert_eq!(
            button_mixin_type, "table",
            "Expected `_G.AchievementFullSearchResultsButtonMixin` to be a table (declared at \
             lua:3392 as `AchievementFullSearchResultsButtonMixin = {{}}` and extended at \
             lua:3394 with `:Init`). Got `{button_mixin_type}`. A `nil` reading means the row \
             initializer at lua:3423 (`button:Init(elementData)`) would crash for every \
             rendered row, leaving the search results scroll box visually empty."
        );

        assert_eq!(
            full_results_signature, EXPECTED_FULL_RESULTS,
            "Expected `_UpdateFullSearchResults` drive (num stubbed to {STUB_NUM_FILTERED}, \
             provider stubbed) to produce signature `{EXPECTED_FULL_RESULTS}`. Got \
             `{full_results_signature}`. A `num_called` other than 1 means the count read at \
             lua:3564 (`local numResults = GetNumFilteredAchievements()`) is duplicated or \
             severed. A `create_count` other than {STUB_NUM_FILTERED} means the count was \
             transformed before reaching `CreateDataProviderByIndexCount` at lua:3566 (e.g. \
             clamped, off-by-one). A `provider=other` reading means the data provider \
             returned by `CreateDataProviderByIndexCount` is NOT the value passed to \
             `ScrollBox:SetDataProvider` at lua:3567 — likely a re-wrap or an unrelated \
             fallback provider. A `title_set=0` reading means the formatted title write at \
             lua:3569 was severed; the search results panel would show the title from a \
             previous run."
        );

        assert_eq!(
            button_init_signature, EXPECTED_BUTTON_INIT,
            "Expected `AchievementFullSearchResultsButtonMixin:Init({{index = {STUB_ROW_INDEX}}})` \
             with `GetFilteredAchievementID` stubbed to return {STUB_FILTERED_ID} to produce \
             signature `{EXPECTED_BUTTON_INIT}`. Got `{button_init_signature}`. An \
             `id_index` other than {STUB_ROW_INDEX} means the index translation at lua:3395 \
             (`local index = elementData.index`) reads from the wrong field — likely \
             `elementData.idx` or `elementData[1]`. An `achievement_id` other than \
             {STUB_FILTERED_ID} means the `self.achievementID = achievementID` write at \
             lua:3402 was severed; the row's `OnClick` (lua:110, dispatching to \
             `AcheivementFullSearchResultsButton_OnClick` at lua:3518) checks \
             `if self.achievementID then ...` and would silently no-op for every click."
        );
    });
}
