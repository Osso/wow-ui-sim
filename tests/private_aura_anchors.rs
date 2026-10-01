//! Public producer and cached-consumer fixtures.
//! Inferred policies and native-security exclusions: docs/specs/private-aura-anchors.md.

use rilua::LuaApiMut;
use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string,
};
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "retail-12-1-0")]
mod cached_consumer_regression {
    use super::WowLuaEnv;

    const CLEARED_EVENT: &str = "UNIT_AURA_BLOCK_LIST_CLEARED";

    #[test]
    fn block_list_cleared_registration_accepts_exact_event_and_rejects_unknown() {
        let env = WowLuaEnv::new().expect("create block-list-cleared registration environment");
        env.exec(
            r#"
            local frame = CreateFrame('Frame')
            assert(pcall(frame.RegisterEvent, frame, 'UNIT_AURA_BLOCK_LIST_CLEARED'),
                'RegisterEvent must accept exact cached 12.1 event')
            assert(frame:IsEventRegistered('UNIT_AURA_BLOCK_LIST_CLEARED'),
                'RegisterEvent must retain exact event registration')
            frame:UnregisterAllEvents()
            assert(pcall(frame.RegisterUnitEvent, frame, 'UNIT_AURA_BLOCK_LIST_CLEARED', 'player'),
                'RegisterUnitEvent must accept exact cached 12.1 event')
            assert(frame:IsEventRegistered('UNIT_AURA_BLOCK_LIST_CLEARED'),
                'RegisterUnitEvent must retain exact event registration')
            for _, method in ipairs({'RegisterEvent', 'RegisterUnitEvent'}) do
                local ok, message = pcall(frame[method], frame,
                    'UNIT_AURA_BLOCK_LIST_CLEARED_UNKNOWN', 'player')
                assert(not ok, method .. ' must still reject unknown events')
                assert(type(message) == 'string' and
                    string.find(message, 'UNIT_AURA_BLOCK_LIST_CLEARED_UNKNOWN', 1, true),
                    method .. ' unknown-event rejection must identify rejected name')
            end
            "#,
        )
        .expect("exact cleared-event registration versus unknown-event rejection");
    }

    #[test]
    fn block_list_cleared_explicit_dispatch_has_one_unit_payload_and_player_filter() {
        let env = WowLuaEnv::new().expect("create explicit cleared-event dispatch environment");
        env.exec(
            r#"
            clearedAll, clearedPlayer = {}, {}
            local function record(events)
                return function(_, event, ...)
                    assert(event == 'UNIT_AURA_BLOCK_LIST_CLEARED', 'exact event delivery')
                    assert(select('#', ...) == 1, 'cleared event must deliver exactly one unit payload')
                    events[#events + 1] = ...
                end
            end
            local all = CreateFrame('Frame')
            all:RegisterAllEvents()
            all:SetScript('OnEvent', record(clearedAll))
            local player = CreateFrame('Frame')
            player:RegisterUnitEvent('UNIT_AURA_BLOCK_LIST_CLEARED', 'player')
            player:SetScript('OnEvent', record(clearedPlayer))
            "#,
        )
        .expect("RegisterAllEvents and player-filtered cleared-event receiver setup");
        for (unit, expected_all) in [("player", 1), ("target", 2)] {
            env.fire_event_with_args(CLEARED_EVENT, &[env.lua_string(unit)])
                .expect("explicit one-unit cleared-event delivery, not native production");
            let counts: (i64, i64) = env
                .eval("return #clearedAll, #clearedPlayer")
                .expect("read synchronous receiver counts after explicit delivery");
            assert_eq!(counts, (expected_all, 1), "delivery counts after {unit}");
        }
        env.exec(
            r#"
            assert(clearedAll[1] == 'player' and clearedAll[2] == 'target',
                'RegisterAllEvents must receive both exact unit tokens in order')
            assert(clearedPlayer[1] == 'player', 'player filter must exclude target payload')
            "#,
        )
        .expect("exact one-unit payload order and player-versus-target filtering");
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "explicit dispatch handler errors: {:?}",
            env.state().borrow().lua_errors
        );
    }

    fn assert_no_lifecycle_errors(env: &WowLuaEnv, prior_errors: &[String], phase: &str) {
        let state = env.state().borrow();
        let new_errors = &state.lua_errors[prior_errors.len()..];
        assert!(
            new_errors.is_empty(),
            "{phase} callback/handler errors: {new_errors:?}; separate closure-load errors: {prior_errors:?}"
        );
    }

    fn add_container_anchor(env: &WowLuaEnv, unit: &str, prior_errors: &[String], phase: &str) {
        let id: i64 = env
            .eval(&format!(
                "return C_UnitAuras.AddPrivateAuraAnchor({{unitToken = '{unit}', \
                 auraIndex = 1, parent = RegressionContainer, isContainer = true}})"
            ))
            .unwrap_or_else(|error| panic!("{phase} public Add failed: {error}"));
        assert!(id > 0, "{phase} must return a positive public anchor ID");
        assert_no_lifecycle_errors(env, prior_errors, phase);
        env.exec(&format!(
            r#"
            RegressionAnchorID = {id}
            local anchors = C_UnitAurasPrivate.GetPrivateAuraAnchors()
            assert(#anchors == 1, '{phase}: one live public anchor record')
            local anchor = anchors[1]
            assert(anchor.anchorID == RegressionAnchorID and anchor.unitToken == '{unit}',
                '{phase}: exact public ID and transitioned unit')
            assert(anchor.isContainer and rawequal(anchor.parent, RegressionContainer),
                '{phase}: container flag and original parent identity')
            assert(anchor.parent.fixtureMarker == 'same-parent', '{phase}: original parent fields')
            assert(RegressionContainer:GetScript('OnAttributeChanged') ~= nil,
                '{phase}: actual added callback must install container settings handler')
            RegressionContainer:SetAttribute('update-settings', true)
            "#
        ))
        .unwrap_or_else(|error| panic!("{phase} public state/settings assertions: {error}"));
        assert_no_lifecycle_errors(env, prior_errors, phase);
    }

    fn remove_container_anchor(env: &WowLuaEnv, prior_errors: &[String], phase: &str) {
        env.exec("C_UnitAuras.RemovePrivateAuraAnchor(RegressionAnchorID)")
            .unwrap_or_else(|error| panic!("{phase} public Remove failed: {error}"));
        assert_no_lifecycle_errors(env, prior_errors, phase);
        env.exec(
            r#"
            assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0,
                'public Remove must leave zero anchor records')
            assert(RegressionContainer:GetScript('OnAttributeChanged') == nil,
                'actual removed callback must release settings handler before same-parent re-add')
            assert(RegressionContainer.fixtureMarker == 'same-parent',
                'removal must preserve original parent identity/state')
            "#,
        )
        .unwrap_or_else(|error| {
            panic!("{phase} public state/handler teardown assertions: {error}")
        });
    }

    #[test]
    fn cached_private_auras_container_callbacks_remove_readd_and_transition_unit() {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_BuffFrame", "Blizzard_PrivateAurasUI"],
            &[],
            |env, loaded| {
                assert!(
                    loaded
                        .iter()
                        .any(|addon| addon == "Blizzard_PrivateAurasUI"),
                    "actual cached PrivateAurasUI root must load through its TOC closure"
                );
                let prior_errors = env.state().borrow().lua_errors.clone();
                if !prior_errors.is_empty() {
                    eprintln!("separate PrivateAurasUI closure-load errors: {prior_errors:?}");
                }
                env.exec(
                    r#"
                    assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0,
                        'closure fixture starts without preexisting anchors')
                    RegressionContainer = CreateFrame('Frame', nil, UIParent)
                    RegressionContainer.fixtureMarker = 'same-parent'
                    RegressionContainer:SetSize(120, 60)
                    local attributes = {
                        ['max-buffs'] = 1, ['max-debuffs'] = 1, ['max-dispel-debuffs'] = 0,
                        ['aura-organization-type'] = Enum.RaidAuraOrganizationType.Legacy,
                        ['always-hide-duration'] = true, ['set-aura-size-to-icon-size'] = true,
                        ['display-larger-role-specific-debuffs'] = false,
                        ['dispel-indicator-overlay-type'] = Enum.RaidDispelOverlayType.UseDebuffColor,
                        ['dispel-indicator-overlay-animation'] = false,
                        ['show-big-defensive'] = false, ['big-defensive-size'] = 30,
                        ['power-bar-used-height'] = 0, ['group-type'] = 0,
                        ['display-only-dispellable-debuffs'] = false,
                        ['ignore-buffs'] = false, ['ignore-debuffs'] = false,
                        ['ignore-dispel-debuffs'] = true,
                        ['dispel-indicator-option'] = Enum.RaidDispelDisplayType.Disabled,
                        ['debuff-size'] = 20, ['buff-size'] = 20,
                        ['debuff-border-scale'] = 1, ['buff-border-scale'] = 1,
                    }
                    for name, value in pairs(attributes) do
                        RegressionContainer:SetAttribute(name, value)
                    end
                    RegressionContainer:Show()
                    assert(RegressionContainer:GetScript('OnAttributeChanged') == nil,
                        'new real container must not own a settings handler before public Add')
                    "#,
                )
                .expect("real CreateFrame container and cached-consumer input attributes, no prerequisite stubs");
                assert_no_lifecycle_errors(env, &prior_errors, "container input setup");
                add_container_anchor(env, "player", &prior_errors, "initial player Add");
                remove_container_anchor(env, &prior_errors, "initial player Remove");
                add_container_anchor(env, "player", &prior_errors, "same-parent player re-add");
                remove_container_anchor(env, &prior_errors, "player Remove before unit transition");
                add_container_anchor(
                    env,
                    "target",
                    &prior_errors,
                    "same-parent target transition",
                );
                remove_container_anchor(env, &prior_errors, "final target Remove");
            },
        );
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create private-anchor environment");
    env.exec(
        r#"
        assert(type(C_UnitAuras.AddPrivateAuraAnchor) == 'function', 'public Add missing')
        assert(type(C_UnitAuras.RemovePrivateAuraAnchor) == 'function', 'public Remove missing')
        function AnchorArgs(unit, index, parent)
            return {unitToken = unit, auraIndex = index, parent = parent}
        end
        function AnchorBinding(relativeTo)
            return {point = 'TOPLEFT', relativeTo = relativeTo,
                relativePoint = 'BOTTOMRIGHT', offsetX = 7.5, offsetY = -3.25}
        end
        function IconInfo(relativeTo)
            return {iconAnchor = AnchorBinding(relativeTo), iconWidth = 24,
                iconHeight = 30, borderScale = 1.25}
        end
        function FindAnchor(id)
            for _, anchor in ipairs(C_UnitAurasPrivate.GetPrivateAuraAnchors()) do
                if anchor.anchorID == id then return anchor end
            end
        end
        function AssertRejected(call, value)
            local ok, message = pcall(call, value)
            assert(not ok, 'malformed or inaccessible input must reject')
            assert(type(message) == 'string' and #message > 0, 'rejection needs context')
        end
    "#,
    )
    .expect("require public producer and install fixture constructors");
    env
}

#[test]
fn ids_are_monotonic_and_environment_local() {
    for _ in 0..2 {
        let env = fixture_env();
        env.exec(
            r#"
            assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0)
            local parent = CreateFrame('Frame')
            local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
            local second = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
            assert(type(first) == 'number' and first == 1, 'retained simulator start-at-1 policy')
            assert(second == first + 1)
            C_UnitAuras.RemovePrivateAuraAnchor(first)
            local third = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 3, parent))
            assert(third == second + 1, 'removed IDs must not be reused')
            assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 2)
        "#,
        )
        .expect("independent environments retain ordered anchor lifecycles");
    }
}

#[test]
fn added_payload_and_listing_preserve_parent_identity_and_default_flags() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        parent.fixtureMarker = 'original'
        local added, argumentCount
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(...)
            argumentCount = select('#', ...)
            added = ...
            assert(FindAnchor(added.anchorID) ~= nil, 'commit before callback')
        end)
        local id = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 4, parent))
        assert(argumentCount == 1)
        local listed = FindAnchor(id)
        for _, record in ipairs({added, listed}) do
            assert(record.anchorID == id and record.unitToken == 'player')
            assert(record.auraIndex == 4)
            assert(rawequal(record.parent, parent), 'parent must not be deep-copied')
            assert(record.parent.fixtureMarker == 'original')
            assert(record.showCooldownFrame == false and record.showCooldownEdge == false)
            assert(record.showCountdownNumbers == false and record.showDispelIcon == false)
            assert(record.isContainer == false)
            assert(record.iconWidth == nil and record.iconHeight == nil and record.borderScale == nil)
            assert(record.spellID == nil and record.spellId == nil and record.auraInstanceID == nil)
        end
        assert(#C_UnitAurasPrivate.GetAllPrivateAuras('player') == 0)
        assert(C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate('player', 4) == nil)
    "#,
    )
    .expect("publish only anchor metadata with canonical parent and schema defaults");
}

#[test]
fn nonempty_optional_bindings_publish_flattened_icon_dimensions() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent, relative = CreateFrame('Frame'), CreateFrame('Frame')
        local args = AnchorArgs('target', 2, parent)
        args.iconInfo = IconInfo(relative)
        args.durationAnchor = AnchorBinding(relative)
        args.showCooldownFrame = true
        args.showCooldownEdge = true
        args.showCountdownNumbers = true
        args.showDispelIcon = true
        args.isContainer = true
        local added
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info) added = info end)
        local id = C_UnitAuras.AddPrivateAuraAnchor(args)
        args.iconInfo.iconWidth = 999
        args.iconInfo.borderScale = 999
        args.unitToken = 'mutated'
        for _, info in ipairs({added, FindAnchor(id)}) do
            assert(info.anchorID == id and info.unitToken == 'target' and info.auraIndex == 2)
            assert(rawequal(info.parent, parent))
            assert(info.iconWidth == 24 and info.iconHeight == 30 and info.borderScale == 1.25)
            assert(info.iconInfo == nil and info.iconAnchor == nil and info.durationAnchor == nil)
            assert(info.showCooldownFrame and info.showCooldownEdge)
            assert(info.showCountdownNumbers and info.showDispelIcon and info.isContainer)
        end
        local optional = AnchorArgs('player', 3, parent)
        optional.iconInfo = IconInfo(relative)
        optional.iconInfo.borderScale = nil
        assert(FindAnchor(C_UnitAuras.AddPrivateAuraAnchor(optional)).borderScale == nil)
    "#,
    )
    .expect("accept full binding records and snapshot optional scalar metadata");
}

#[test]
fn listing_filters_units_and_isolates_callback_and_result_mutations() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info)
            info.unitToken = 'callback mutation'
            info.auraIndex = 999
            info.parent = nil
            info.iconWidth = 999
        end)
        local args = AnchorArgs('player', 1, parent)
        args.iconInfo = IconInfo(parent)
        local first = C_UnitAuras.AddPrivateAuraAnchor(args)
        local second = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
        local all = C_UnitAurasPrivate.GetPrivateAuraAnchors()
        local players = C_UnitAurasPrivate.GetPrivateAuraAnchors('player')
        local targets = C_UnitAurasPrivate.GetPrivateAuraAnchors('target')
        assert(#all == 2 and all[1].anchorID == first and all[2].anchorID == second)
        assert(#players == 1 and players[1].anchorID == first)
        assert(#targets == 1 and targets[1].anchorID == second)
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors('party1') == 0)
        assert(not rawequal(all, C_UnitAurasPrivate.GetPrivateAuraAnchors()))
        assert(not rawequal(all[1], players[1]))
        all[1].unitToken = 'list mutation'
        all[1].parent = nil
        players[1].auraIndex = 999
        players[1].iconWidth = 999
        table.remove(all, 2)
        local fresh = FindAnchor(first)
        assert(fresh.unitToken == 'player' and fresh.auraIndex == 1)
        assert(fresh.iconWidth == 24 and rawequal(fresh.parent, parent))
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 2)
    "#,
    )
    .expect("fresh ordered snapshots preserve retained unit filtering and copy isolation");
}

#[test]
fn removal_has_no_results_and_reentrant_callbacks_observe_committed_state() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local second = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
        local removed = {}
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(...)
            assert(select('#', ...) == 1)
            local id = ...
            removed[#removed + 1] = id
            assert(FindAnchor(id) == nil, 'remove before dispatch')
            assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(id)) == 0)
            if id == first then
                local remaining = C_UnitAurasPrivate.GetPrivateAuraAnchors()
                assert(#remaining == 1 and remaining[1].anchorID == second)
                C_UnitAuras.RemovePrivateAuraAnchor(second)
            end
        end)
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(first)) == 0)
        assert(#removed == 2 and removed[1] == first and removed[2] == second)
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0)
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(second + 100)) == 0)
        assert(#removed == 2, 'unknown ID no-op is inferred simulator policy')
    "#,
    )
    .expect("remove real IDs before callback reentry without duplicate notification");
}

// Characterize inferred simulator policy, not native callback-error semantics.
#[test]
fn added_callback_error_retains_anchor_without_returning_id_and_dispatch_recovers() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local baseline = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 1, parent))
        local reached, failedPayload, visibleDuringCallback = 0, nil, nil
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info)
            reached = reached + 1
            failedPayload = info
            visibleDuringCallback = FindAnchor(info.anchorID)
            error('private-anchor-added-callback-failure', 0)
        end)
        local function pack(...) return {n = select('#', ...), ...} end
        local result = pack(pcall(C_UnitAuras.AddPrivateAuraAnchor,
            AnchorArgs('player', 4, parent)))
        assert(reached == 1, 'ordinary public Add must reach added callback')
        assert(result.n == 2 and result[1] == false,
            'failed Add must yield only pcall failure and error, not a fabricated ID')
        assert(type(result[2]) == 'string' and
            string.find(result[2], 'private-anchor-added-callback-failure', 1, true),
            'added callback error must propagate to public caller')
        local failedID = failedPayload.anchorID
        assert(failedID == baseline + 1, 'failed Add still consumes its ID')
        assert(visibleDuringCallback.anchorID == failedID and
            visibleDuringCallback.unitToken == 'player', 'insert precedes added callback')
        assert(failedPayload.auraIndex == 4 and rawequal(failedPayload.parent, parent),
            'callback receives concrete published metadata and actual parent')
        local all = C_UnitAurasPrivate.GetPrivateAuraAnchors()
        local players = C_UnitAurasPrivate.GetPrivateAuraAnchors('player')
        assert(#all == 2 and all[1].anchorID == baseline and all[2].anchorID == failedID,
            'callback failure must not roll back insertion or disturb existing record')
        assert(#players == 1 and players[1].anchorID == failedID and
            players[1].auraIndex == 4 and rawequal(players[1].parent, parent),
            'failed Add retains original state in filtered listing')

        local added, removed = {}, {}
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(...)
            assert(select('#', ...) == 1, 'recovered added dispatch has one payload')
            local info = ...
            assert(rawequal(info.parent, parent) and FindAnchor(info.anchorID) ~= nil)
            added[#added + 1] = info.anchorID
        end)
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(...)
            assert(select('#', ...) == 1, 'recovered removed dispatch has one ID')
            local id = ...
            assert(FindAnchor(id) == nil)
            removed[#removed + 1] = id
        end)
        local nextID = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 5, parent))
        assert(nextID == failedID + 1, 'next successful Add must not reuse failed Add ID')
        assert(#added == 1 and added[1] == nextID, 'replacement callback remains usable')
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(failedID)) == 0)
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(nextID)) == 0)
        assert(#removed == 2 and removed[1] == failedID and removed[2] == nextID)
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors('player') == 0)
        assert(FindAnchor(baseline).unitToken == 'target', 'unrelated anchor survives recovery')
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(nil)
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(nil)
        C_UnitAuras.RemovePrivateAuraAnchor(baseline)
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0)
        "#,
    )
    .expect("inferred added callback error retains insertion and later public dispatch works");
}

// Characterize inferred simulator policy, not native callback-error semantics.
#[test]
fn removed_callback_error_retains_deletion_and_dispatch_recovers() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local survivor = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
        local reached, notifiedID, absentDuringCallback = 0, nil, false
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(id)
            reached = reached + 1
            notifiedID = id
            absentDuringCallback = FindAnchor(id) == nil
            error('private-anchor-removed-callback-failure', 0)
        end)
        local function pack(...) return {n = select('#', ...), ...} end
        local result = pack(pcall(C_UnitAuras.RemovePrivateAuraAnchor, first))
        assert(reached == 1 and notifiedID == first, 'public Remove reaches callback with exact ID')
        assert(absentDuringCallback, 'deletion precedes removed callback')
        assert(result.n == 2 and result[1] == false,
            'failed Remove must yield only pcall failure and error')
        assert(type(result[2]) == 'string' and
            string.find(result[2], 'private-anchor-removed-callback-failure', 1, true),
            'removed callback error must propagate to public caller')
        local all = C_UnitAurasPrivate.GetPrivateAuraAnchors()
        assert(#all == 1 and all[1].anchorID == survivor and
            all[1].unitToken == 'target' and rawequal(all[1].parent, parent),
            'callback failure retains deletion without disturbing survivor')
        assert(FindAnchor(first) == nil and
            #C_UnitAurasPrivate.GetPrivateAuraAnchors('player') == 0,
            'removed record stays absent after public failure')
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(first)) == 0)
        assert(reached == 1, 'retry of already deleted ID must not repeat failing callback')

        local added, removed = {}, {}
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(nil)
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(...)
            assert(select('#', ...) == 1, 'recovered removed dispatch has one ID')
            local id = ...
            assert(FindAnchor(id) == nil)
            removed[#removed + 1] = id
        end)
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(...)
            assert(select('#', ...) == 1, 'recovered added dispatch has one payload')
            local info = ...
            assert(info.unitToken == 'player' and rawequal(info.parent, parent))
            assert(FindAnchor(info.anchorID) ~= nil)
            added[#added + 1] = info.anchorID
        end)
        local nextID = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 3, parent))
        assert(nextID == survivor + 1, 'postfailure Add preserves monotonic IDs')
        assert(#added == 1 and added[1] == nextID, 'added callback usable after Remove error')
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(survivor)) == 0)
        assert(select('#', C_UnitAuras.RemovePrivateAuraAnchor(nextID)) == 0)
        assert(#removed == 2 and removed[1] == survivor and removed[2] == nextID,
            'replacement removed callback usable for later valid operations')
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0)
        "#,
    )
    .expect("inferred removed callback error retains deletion and later public dispatch works");
}

#[test]
fn added_callback_can_remove_the_just_published_id() {
    let env = fixture_env();
    env.exec(
        r#"
        local removed, addedCount, removedCount = nil, 0, 0
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(id)
            removedCount = removedCount + 1
            removed = id
        end)
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info)
            addedCount = addedCount + 1
            assert(FindAnchor(info.anchorID) ~= nil)
            C_UnitAuras.RemovePrivateAuraAnchor(info.anchorID)
            assert(FindAnchor(info.anchorID) == nil)
        end)
        local id = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, CreateFrame('Frame')))
        assert(type(id) == 'number' and addedCount == 1 and removedCount == 1)
        assert(removed == id)
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 0)
    "#,
    )
    .expect("added dispatch releases model borrow before synchronous removal");
}

#[test]
fn callbacks_survive_gc_and_replacement_uses_only_latest_handlers() {
    let env = fixture_env();
    env.exec(
        r#"
        local oldAdded, oldRemoved, newAdded, newRemoved = 0, 0, 0, 0
        do
            local captured = {marker = 'rooted closure'}
            C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info)
                assert(captured.marker == 'rooted closure' and info.anchorID > 0)
                oldAdded = oldAdded + 1
            end)
            C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(id)
                assert(captured.marker == 'rooted closure' and id > 0)
                oldRemoved = oldRemoved + 1
            end)
        end
        collectgarbage('collect')
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        C_UnitAuras.RemovePrivateAuraAnchor(first)
        assert(oldAdded == 1 and oldRemoved == 1)
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function(info)
            assert(FindAnchor(info.anchorID) ~= nil)
            newAdded = newAdded + 1
        end)
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function(id)
            assert(FindAnchor(id) == nil)
            newRemoved = newRemoved + 1
        end)
        collectgarbage('collect')
        local second = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
        C_UnitAuras.RemovePrivateAuraAnchor(second)
        assert(oldAdded == 1 and oldRemoved == 1 and newAdded == 1 and newRemoved == 1)
    "#,
    )
    .expect("root callback functions across collection and honor replacement");
}

#[test]
fn parent_identity_and_custom_fields_survive_gc_without_input_roots() {
    let env = fixture_env();
    env.exec(
        r#"
        local weak = setmetatable({}, {__mode = 'v'})
        local id
        do
            local parent = CreateFrame('Frame')
            parent.fixtureMarker = 'parent survives'
            weak[1] = parent
            id = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        end
        collectgarbage('collect')
        collectgarbage('collect')
        local info = FindAnchor(id)
        assert(weak[1] ~= nil and rawequal(info.parent, weak[1]))
        assert(info.parent.fixtureMarker == 'parent survives')
        info.parent:SetSize(41, 53)
        assert(weak[1]:GetWidth() == 41 and weak[1]:GetHeight() == 53)
        C_UnitAuras.RemovePrivateAuraAnchor(id)
        assert(FindAnchor(id) == nil)
    "#,
    )
    .expect("retrieve original usable frame, not copied userdata-like tables");
}

#[test]
fn malformed_inputs_are_atomic_and_do_not_consume_ids() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local notifications = 0
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function() notifications = notifications + 1 end)
        AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, nil)
        AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, 'not a structure')
        for _, field in ipairs({'unitToken', 'auraIndex', 'parent'}) do
            local args = AnchorArgs('player', 2, parent)
            args[field] = nil
            AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
        end
        local badFields = {unitToken = {}, auraIndex = 'index', parent = {},
            showCooldownFrame = 'yes', showCooldownEdge = {}, showCountdownNumbers = 'yes',
            showDispelIcon = {}, isContainer = 'yes', iconInfo = 'icon', durationAnchor = 'anchor'}
        for field, value in pairs(badFields) do
            local args = AnchorArgs('player', 2, parent)
            args[field] = value
            AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
        end
        AssertRejected(C_UnitAuras.RemovePrivateAuraAnchor, nil)
        AssertRejected(C_UnitAuras.RemovePrivateAuraAnchor, {})
        assert(notifications == 0 and #C_UnitAurasPrivate.GetPrivateAuraAnchors() == 1)
        assert(FindAnchor(first) ~= nil)
        local nextID = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent))
        assert(nextID == first + 1 and notifications == 1)
    "#,
    )
    .expect("required fields and scalar types validate before any mutation");
}

#[test]
fn optional_icon_and_duration_bindings_validate_every_required_field_atomically() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local notifications = 0
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function() notifications = notifications + 1 end)
        for _, field in ipairs({'iconAnchor', 'iconWidth', 'iconHeight'}) do
            local args = AnchorArgs('player', 2, parent)
            args.iconInfo = IconInfo(parent)
            args.iconInfo[field] = nil
            AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
        end
        for _, slot in ipairs({'iconInfo', 'durationAnchor'}) do
            for _, field in ipairs({'point', 'relativeTo', 'relativePoint', 'offsetX', 'offsetY'}) do
                local args = AnchorArgs('player', 2, parent)
                local binding = AnchorBinding(parent)
                binding[field] = nil
                if slot == 'iconInfo' then
                    args.iconInfo = IconInfo(parent)
                    args.iconInfo.iconAnchor = binding
                else
                    args.durationAnchor = binding
                end
                AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
            end
            for field, value in pairs({point = 'NO_POINT', relativeTo = {},
                relativePoint = 'NO_POINT', offsetX = 'x', offsetY = {}}) do
                local args = AnchorArgs('player', 2, parent)
                local binding = AnchorBinding(parent)
                binding[field] = value
                if slot == 'iconInfo' then
                    args.iconInfo = IconInfo(parent)
                    args.iconInfo.iconAnchor = binding
                else
                    args.durationAnchor = binding
                end
                AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
            end
        end
        for _, field in ipairs({'iconWidth', 'iconHeight', 'borderScale'}) do
            local args = AnchorArgs('player', 2, parent)
            args.iconInfo = IconInfo(parent)
            args.iconInfo[field] = 'not a number'
            AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
        end
        assert(notifications == 0 and #C_UnitAurasPrivate.GetPrivateAuraAnchors() == 1)
        assert(C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent)) == first + 1)
    "#,
    )
    .expect("parse nonempty optional structures fully before consuming an ID");
}

fn install_host_security_fixtures(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua)
        .expect("install actual VM table guards");
    let number = wrap_host_secret_number(lua.state_mut(), 2.0);
    lua.state_mut().push(number);
    let inserted = lua.set_global_val("SecretAnchorNumber", number);
    lua.state_mut().pop();
    inserted.expect("root secret number");
    let boolean = wrap_host_secret_bool(lua.state_mut(), true);
    lua.state_mut().push(boolean);
    let inserted = lua.set_global_val("SecretAnchorBool", boolean);
    lua.state_mut().pop();
    inserted.expect("root secret boolean");
    let string = wrap_host_secret_string(lua.state_mut(), "player");
    lua.state_mut().push(string);
    let inserted = lua.set_global_val("SecretAnchorString", string);
    lua.state_mut().pop();
    inserted.expect("root secret string");
}

#[test]
fn secret_outer_nested_and_scalar_inputs_reject_without_clearing_taint() {
    let env = fixture_env();
    install_host_security_fixtures(&env);
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local notifications = 0
        C_UnitAurasPrivate.SetPrivateAuraAnchorAddedCallback(function() notifications = notifications + 1 end)
        C_UnitAurasPrivate.SetPrivateAuraAnchorRemovedCallback(function() notifications = notifications + 1 end)
        local cases = {secretwrap(AnchorArgs('player', 2, parent))}
        for field, value in pairs({unitToken = SecretAnchorString, auraIndex = SecretAnchorNumber,
            parent = secretwrap(parent), showCooldownFrame = SecretAnchorBool,
            showCooldownEdge = SecretAnchorBool, showCountdownNumbers = SecretAnchorBool,
            showDispelIcon = SecretAnchorBool, isContainer = SecretAnchorBool,
            iconInfo = secretwrap(IconInfo(parent)), durationAnchor = secretwrap(AnchorBinding(parent))}) do
            local args = AnchorArgs('player', 2, parent)
            args[field] = value
            cases[#cases + 1] = args
        end
        for _, field in ipairs({'iconWidth', 'iconHeight', 'borderScale'}) do
            local args = AnchorArgs('player', 2, parent)
            args.iconInfo = IconInfo(parent)
            args.iconInfo[field] = SecretAnchorNumber
            cases[#cases + 1] = args
        end
        local nested = AnchorArgs('player', 2, parent)
        nested.iconInfo = IconInfo(parent)
        nested.iconInfo.iconAnchor = secretwrap(AnchorBinding(parent))
        cases[#cases + 1] = nested
        for _, slot in ipairs({'iconInfo', 'durationAnchor'}) do
            for field, value in pairs({point = secretwrap('TOPLEFT'), relativeTo = secretwrap(parent),
                relativePoint = secretwrap('BOTTOMRIGHT'), offsetX = SecretAnchorNumber, offsetY = SecretAnchorNumber}) do
                local args = AnchorArgs('player', 2, parent)
                local binding = AnchorBinding(parent)
                binding[field] = value
                if slot == 'iconInfo' then
                    args.iconInfo = IconInfo(parent)
                    args.iconInfo.iconAnchor = binding
                else
                    args.durationAnchor = binding
                end
                cases[#cases + 1] = args
            end
        end
        collectgarbage('collect')
        local function reject()
            for _, args in ipairs(cases) do AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args) end
            AssertRejected(C_UnitAuras.RemovePrivateAuraAnchor, SecretAnchorNumber)
            assert(issecretvalue(cases[1]) and issecretvalue(SecretAnchorNumber))
            assert(issecretvalue(SecretAnchorBool) and issecretvalue(SecretAnchorString))
            assert(notifications == 0 and FindAnchor(first) ~= nil)
            assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 1)
        end
        assert(issecure()); reject(); assert(issecure())
        local function addon()
            assert(not issecure()); reject(); assert(not issecure())
        end
        debug.setobjecttaint(addon, 'PrivateAnchorFixture')
        addon(); assert(issecure())
        assert(C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('target', 2, parent)) == first + 1)
    "#,
    )
    .expect("conservative secret rejection preserves state, IDs and caller taint");
}

#[test]
fn secured_outer_and_nested_tables_respect_vm_access_guards() {
    let env = fixture_env();
    install_host_security_fixtures(&env);
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local first = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
        local cases, guards, probeFields = {}, {}, {}
        for _, slot in ipairs({'outer', 'iconInfo', 'iconAnchor', 'durationAnchor'}) do
            local args = AnchorArgs('player', 2, parent)
            args.iconInfo = IconInfo(parent)
            args.durationAnchor = AnchorBinding(parent)
            local guarded, probe
            if slot == 'outer' then guarded, probe = args, 'unitToken'
            elseif slot == 'iconInfo' then guarded, probe = args.iconInfo, 'iconWidth'
            elseif slot == 'iconAnchor' then guarded, probe = args.iconInfo.iconAnchor, 'point'
            else guarded, probe = args.durationAnchor, 'point' end
            settablesecurity(guarded, 0)
            cases[#cases + 1], guards[#guards + 1], probeFields[#probeFields + 1] = args, guarded, probe
        end
        local function addon()
            for index, args in ipairs(cases) do
                local ok, message = pcall(rawget, guards[index], probeFields[index])
                assert(not ok and string.find(message, 'tainted access to secured table', 1, true))
                AssertRejected(C_UnitAuras.AddPrivateAuraAnchor, args)
                assert(not issecure(), 'guard rejection preserves addon taint')
            end
        end
        debug.setobjecttaint(addon, 'PrivateAnchorFixture')
        addon(); assert(issecure())
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 1)
        for index, args in ipairs(cases) do
            assert(C_UnitAuras.AddPrivateAuraAnchor(args) == first + index)
        end
        assert(#C_UnitAurasPrivate.GetPrivateAuraAnchors() == 5)
    "#,
    )
    .expect("read nested structures through guarded VM access, not raw unguarded storage");
}

#[test]
fn ordinary_public_add_and_remove_preserve_addon_taint() {
    let env = fixture_env();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local function addon()
            assert(not issecure())
            local id = C_UnitAuras.AddPrivateAuraAnchor(AnchorArgs('player', 1, parent))
            assert(not issecure() and FindAnchor(id) ~= nil)
            C_UnitAuras.RemovePrivateAuraAnchor(id)
            assert(not issecure() and FindAnchor(id) == nil)
        end
        debug.setobjecttaint(addon, 'PrivateAnchorFixture')
        assert(issecure()); addon(); assert(issecure())
    "#,
    )
    .expect("restriction removal does not launder ordinary addon callers");
}
