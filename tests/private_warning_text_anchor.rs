//! Inputs-only contract fixtures; compile/RED belongs to the parent before production work.
//! Parenting, replacement and nil policies are inferences, not native-verified semantics.
//! See docs/specs/private-warning-text-anchor.md for evidence and confidence limits.

#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

fn warning_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create isolated warning anchor environment");
    env.exec(
        r#"
        PublicParent = CreateFrame('Frame', nil, UIParent)
        PublicParent:SetSize(800, 80)
        PublicParent:SetPoint('TOP', UIParent, 'TOP', 0, -182)
        OtherParent = CreateFrame('Frame', nil, UIParent)
        OtherParent:SetSize(600, 60)
        OtherParent:SetPoint('BOTTOM', UIParent, 'BOTTOM', 10, 90)
        PrivateText = CreateFrame('Frame', nil, UIParent)
        PrivateText:SetPoint('CENTER', UIParent, 'CENTER', 3, 4)
        function Binding(relativeTo)
            return {point = 'TOP', relativeTo = relativeTo, relativePoint = 'BOTTOM',
                offsetX = 17, offsetY = -23}
        end
        function AssertPlacement(frame, parent, relativeTo, point, relativePoint, x, y)
            assert(frame:GetParent() == parent, 'private frame parent identity')
            assert(frame:GetNumPoints() == 1, 'replace, not append, explicit private anchor')
            local actualPoint, actualRelative, actualRelativePoint, actualX, actualY = frame:GetPoint(1)
            assert(actualPoint == point and actualRelative == relativeTo, 'private point and relative frame')
            assert(actualRelativePoint == relativePoint, 'private relative point')
            assert(actualX == x and actualY == y, 'private anchor offsets')
        end
        function AssertPublicUnchanged()
            AssertPlacement(PublicParent, UIParent, UIParent, 'TOP', 'TOP', 0, -182)
            AssertPlacement(OtherParent, UIParent, UIParent, 'BOTTOM', 'BOTTOM', 10, 90)
            assert(PublicParent:GetWidth() == 800 and PublicParent:GetHeight() == 80)
            assert(OtherParent:GetWidth() == 600 and OtherParent:GetHeight() == 60)
        end
        function AssertBound()
            AssertPlacement(PrivateText, PublicParent, PublicParent, 'TOP', 'BOTTOM', 17, -23)
            AssertPublicUnchanged()
        end
        function AssertRejected(parent, binding)
            local ok, message = pcall(C_UnitAuras.SetPrivateWarningTextAnchor, parent, binding)
            assert(not ok, 'conservative simulator policy must reject invalid/secret input')
            assert(type(message) == 'string' and #message > 0, 'explicit input error')
        end
        "#,
    )
    .expect("create concrete public parents and separate private text frame");
    env
}

#[test]
fn public_binding_before_private_registration_applies_to_private_frame() {
    let env = warning_env();
    env.exec(
        r#"
        assert(select('#', C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))) == 0)
        AssertPlacement(PrivateText, UIParent, UIParent, 'CENTER', 'CENTER', 3, 4)
        AssertPublicUnchanged()
        collectgarbage('collect')
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertBound()
        "#,
    )
    .expect("retain public binding until separate private frame registers");
}

#[test]
fn public_binding_after_private_registration_accepts_tainted_caller_in_combat() {
    let env = warning_env();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(
        r#"
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertPlacement(PrivateText, UIParent, UIParent, 'CENTER', 'CENTER', 3, 4)
        assert(InCombatLockdown(), 'actual simulator combat state')
        local function addon()
            assert(not issecure(), 'actual tainted caller')
            assert(select('#', C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))) == 0)
            AssertBound()
            assert(not issecure(), 'public setter must preserve caller taint')
        end
        debug.setobjecttaint(addon, 'WarningAnchorFixture')
        addon()
        assert(issecure())
        "#,
    )
    .expect("restriction removal permits nonsecret valid binding from tainted caller in combat");
}

#[test]
fn latest_public_parent_and_binding_replace_previous_placement() {
    let env = warning_env();
    env.exec(
        r#"
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        local replacement = {point = 'LEFT', relativeTo = PublicParent, relativePoint = 'RIGHT',
            offsetX = -11, offsetY = 9}
        C_UnitAuras.SetPrivateWarningTextAnchor(OtherParent, replacement)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertPlacement(PrivateText, OtherParent, PublicParent, 'LEFT', 'RIGHT', -11, 9)
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(OtherParent))
        AssertPlacement(PrivateText, PublicParent, OtherParent, 'TOP', 'BOTTOM', 17, -23)
        AssertPublicUnchanged()
        "#,
    )
    .expect("latest retained binding wins before and after registration; relativeTo differs from parent");
}

#[test]
fn replacing_private_frame_reapplies_retained_binding_without_moving_old_frame() {
    let env = warning_env();
    env.exec(
        r#"
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        AssertBound()
        local replacement = CreateFrame('Frame', nil, UIParent)
        replacement:SetPoint('CENTER', UIParent, 'CENTER', 5, 6)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(replacement)
        AssertPlacement(replacement, PublicParent, PublicParent, 'TOP', 'BOTTOM', 17, -23)
        C_UnitAuras.SetPrivateWarningTextAnchor(OtherParent, Binding(OtherParent))
        AssertPlacement(replacement, OtherParent, OtherParent, 'TOP', 'BOTTOM', 17, -23)
        AssertPlacement(PrivateText, PublicParent, PublicParent, 'TOP', 'BOTTOM', 17, -23)
        AssertPublicUnchanged()
        "#,
    )
    .expect("new private registration receives latest binding; superseded frame no longer follows updates");
}

#[test]
fn nil_binding_clears_private_points_and_retains_parent_in_both_call_orders() {
    let env = warning_env();
    env.exec(
        r#"
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        C_UnitAuras.SetPrivateWarningTextAnchor(OtherParent, nil)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        assert(PrivateText:GetParent() == OtherParent and PrivateText:GetNumPoints() == 0,
            'inferred nil binding retained before private registration')
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        AssertBound()
        assert(select('#', C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent)) == 0)
        assert(PrivateText:GetParent() == PublicParent and PrivateText:GetNumPoints() == 0,
            'inferred omitted/nil binding clears existing explicit points, not supplied parent')
        local replacement = CreateFrame('Frame', nil, UIParent)
        replacement:SetPoint('CENTER', UIParent)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(replacement)
        assert(replacement:GetParent() == PublicParent and replacement:GetNumPoints() == 0)
        AssertPublicUnchanged()
        "#,
    )
    .expect("inferred nil/omitted-anchor policy, not native characterized semantics");
}

#[test]
fn warning_anchor_bindings_are_isolated_between_environments() {
    let first = warning_env();
    first
        .exec("C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))")
        .expect("retain first environment binding");
    let second = warning_env();
    second
        .exec(
            r#"
            C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
            AssertPlacement(PrivateText, UIParent, UIParent, 'CENTER', 'CENTER', 3, 4)
            C_UnitAuras.SetPrivateWarningTextAnchor(OtherParent, Binding(OtherParent))
            AssertPlacement(PrivateText, OtherParent, OtherParent, 'TOP', 'BOTTOM', 17, -23)
            AssertPublicUnchanged()
            "#,
        )
        .expect("second environment must not inherit first environment pending binding");
    first
        .exec("C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText); AssertBound()")
        .expect("second environment changes must not replace first environment binding");
}

#[test]
fn retained_binding_is_a_snapshot_and_never_reanchors_public_parent_to_itself() {
    let env = warning_env();
    env.exec(
        r#"
        local binding = {point = 'TOP', relativeTo = PublicParent, relativePoint = 'TOP',
            offsetX = 0, offsetY = 0}
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, binding)
        binding.point = 'LEFT'
        binding.relativeTo = OtherParent
        binding.offsetX = 100
        collectgarbage('collect')
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertPlacement(PrivateText, PublicParent, PublicParent, 'TOP', 'TOP', 0, 0)
        AssertPublicUnchanged()
        "#,
    )
    .expect(
        "cached self-relative binding belongs on private frame; inferred input snapshot policy",
    );
}

#[test]
fn malformed_public_inputs_preserve_applied_and_pending_binding_atomically() {
    let env = warning_env();
    env.exec(
        r#"
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        local function rejectMalformed()
            for _, parent in ipairs({false, 7, 'PublicParent', {}, UIParent:CreateTexture()}) do
                AssertRejected(parent, Binding(OtherParent))
            end
            AssertRejected(nil, Binding(OtherParent))
            for _, binding in ipairs({false, 7, 'TOP', {}}) do
                AssertRejected(OtherParent, binding)
            end
            for _, field in ipairs({'point', 'relativeTo', 'relativePoint', 'offsetX', 'offsetY'}) do
                local binding = Binding(OtherParent)
                binding[field] = nil
                AssertRejected(OtherParent, binding)
            end
            for field, value in pairs({point = 'INVALID', relativeTo = {}, relativePoint = 'INVALID',
                offsetX = '17', offsetY = false}) do
                local binding = Binding(OtherParent)
                binding[field] = value
                AssertRejected(OtherParent, binding)
            end
        end
        rejectMalformed()
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertBound()
        rejectMalformed()
        AssertBound()
        local replacement = CreateFrame('Frame', nil, UIParent)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(replacement)
        AssertPlacement(replacement, PublicParent, PublicParent, 'TOP', 'BOTTOM', 17, -23)
        AssertPublicUnchanged()
        "#,
    )
    .expect("reject whole input before changing pending or applied placement, without coercion");
}

#[test]
fn secret_public_inputs_preserve_binding_and_caller_taint_atomically() {
    let env = warning_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua)
            .expect("install actual VM secret wrapper");
        let number = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 17.0);
        lua.state_mut().push(number);
        let inserted = lua.set_global_val("SecretWarningOffset", number);
        lua.state_mut().pop();
        inserted.expect("root secret numeric offset");
    }
    env.exec(
        r#"
        C_UnitAuras.SetPrivateWarningTextAnchor(PublicParent, Binding(PublicParent))
        local cases = {
            {secretwrap(OtherParent), Binding(OtherParent)},
            {OtherParent, secretwrap(Binding(OtherParent))},
        }
        for field, value in pairs({point = secretwrap('TOP'), relativeTo = secretwrap(OtherParent),
            relativePoint = secretwrap('BOTTOM'), offsetX = SecretWarningOffset, offsetY = SecretWarningOffset}) do
            local binding = Binding(OtherParent)
            binding[field] = value
            cases[#cases + 1] = {OtherParent, binding}
        end
        collectgarbage('collect')
        local function reject()
            for _, case in ipairs(cases) do AssertRejected(case[1], case[2]) end
            assert(issecretvalue(cases[1][1]) and issecretvalue(cases[2][2]))
            assert(issecretvalue(SecretWarningOffset), 'must not declassify rejected input')
            AssertPublicUnchanged()
        end
        assert(issecure()); reject(); assert(issecure())
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(PrivateText)
        AssertBound()
        local function addon()
            assert(not issecure()); reject(); AssertBound(); assert(not issecure())
        end
        debug.setobjecttaint(addon, 'WarningAnchorFixture')
        addon(); assert(issecure())
        local replacement = CreateFrame('Frame', nil, UIParent)
        C_UnitAurasPrivate.SetPrivateWarningTextFrame(replacement)
        AssertPlacement(replacement, PublicParent, PublicParent, 'TOP', 'BOTTOM', 17, -23)
        "#,
    )
    .expect("conservative secret rejection and atomicity, NOT AllowedWhenUntainted parity");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn cached_raid_warning_and_private_auras_lifecycle_places_distinct_private_frame() {
    use crate::common::blizzard_addon_harness::{
        load_blizzard_addon_closure_into_env, with_blizzard_addon_closure,
    };
    use crate::common::panel_fixtures::blizzard_ui_dir;

    with_blizzard_addon_closure(&["Blizzard_RaidWarning"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_RaidWarning"));
        env.exec(
            r#"
            assert(PrivateRaidBossEmoteFrameAnchor, 'actual RaidWarning XML public anchor exists')
            local point, relativeTo, relativePoint, x, y = PrivateRaidBossEmoteFrameAnchor:GetPoint(1)
            assert(point == 'TOP' and relativeTo == RaidWarningFrame and relativePoint == 'TOP')
            assert(x == 0 and y == 0, 'original public XML anchor offsets')
            "#,
        )
        .expect("actual cached public OnLoad must preserve preexisting public anchor geometry");
        let private_loaded = load_blizzard_addon_closure_into_env(
            env,
            &blizzard_ui_dir(),
            &["Blizzard_BuffFrame", "Blizzard_PrivateAurasUI"],
            &[],
        );
        assert!(
            private_loaded
                .iter()
                .any(|name| name == "Blizzard_PrivateAurasUI")
        );
        let prior_errors = env.state().borrow().lua_errors.clone();
        if !prior_errors.is_empty() {
            eprintln!("separate cached closure-load errors: {prior_errors:?}");
        }
        env.exec(
            r#"
            -- Existing private registration slot locates the hidden secure-environment XML frame.
            local private = C_UnitAurasPrivate._state.warningTextFrame
            local public = PrivateRaidBossEmoteFrameAnchor
            assert(private and private:GetName() == 'RaidBossEmoteFramePrivate')
            assert(private ~= public, 'private XML consumer must be distinct from public anchor')
            local function assertPrivateBinding()
                assert(private:GetParent() == public and private:GetNumPoints() == 1)
                local point, relativeTo, relativePoint, x, y = private:GetPoint(1)
                assert(point == 'TOP' and relativeTo == public and relativePoint == 'TOP')
                assert(x == 0 and y == 0)
            end
            assertPrivateBinding()
            assert(DeadlyDebuffFrame, 'real BuffFrame dependency supplies center-screen consumer')
            DeadlyDebuffFrame:Show()
            RaidWarningUtil.UpdateCenterScreenAnchors()
            local point, relativeTo, relativePoint, x, y = public:GetPoint(1)
            assert(point == 'TOP' and relativeTo == DeadlyDebuffFrame and relativePoint == 'BOTTOM')
            assert(x == 0 and y == 0, 'real utility repositions only public anchor')
            assertPrivateBinding()
            DeadlyDebuffFrame:Hide()
            RaidWarningUtil.UpdateCenterScreenAnchors()
            point, relativeTo, relativePoint = public:GetPoint(1)
            assert(point == 'TOP' and relativeTo == RaidWarningFrame and relativePoint == 'TOP')
            assertPrivateBinding()
            "#,
        )
        .expect("actual TOC/XML/mixin registration and dynamic utility repositioning, no fake callbacks");
        let state = env.state().borrow();
        assert!(
            state.lua_errors[prior_errors.len()..].is_empty(),
            "warning lifecycle errors: {:?}; separate closure errors: {prior_errors:?}",
            &state.lua_errors[prior_errors.len()..]
        );
    });
}
