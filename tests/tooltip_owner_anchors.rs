//! WM63 O17 §1: `SetOwner` anchor mode table, parser default and default
//! displacements for Retail/PTR. Spec: `docs/specs/tooltip-owner-anchors.md`.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv init");
    env.exec(
        r#"
        _G.OwnerAnchorFrame = CreateFrame("Frame", "TooltipAnchorOwner", UIParent)
        OwnerAnchorFrame:SetSize(100, 50)
        _G.OwnerAnchorTip = CreateFrame("GameTooltip", "OwnerAnchorProbeTip", UIParent, "GameTooltipTemplate")
        "#,
    )
    .unwrap();
    env
}

fn owner_anchor_point(
    env: &WowLuaEnv,
    mode: &str,
    explicit_zero: bool,
) -> (String, String, f64, f64) {
    let call = if explicit_zero {
        format!("OwnerAnchorTip:SetOwner(OwnerAnchorFrame, {mode:?}, 0, 0)")
    } else {
        format!("OwnerAnchorTip:SetOwner(OwnerAnchorFrame, {mode:?})")
    };
    env.eval(&format!(
        r#"
        {call}
        local point, _, relativePoint, x, y = OwnerAnchorTip:GetPoint(1)
        return point, relativePoint, x, y
        "#
    ))
    .unwrap()
}

/// O17 §1: each mode pins the tooltip point to the owner point in the
/// verified table, and both displacements default to zero.
#[test]
fn tooltip_owner_anchor_mode_table_matches_native_point_pairs() {
    let env = env();
    let expected = [
        ("ANCHOR_LEFT", "BOTTOMRIGHT", "TOPLEFT"),
        ("ANCHOR_RIGHT", "BOTTOMLEFT", "TOPRIGHT"),
        ("ANCHOR_BOTTOMLEFT", "TOPRIGHT", "BOTTOMLEFT"),
        ("ANCHOR_BOTTOM", "TOP", "BOTTOM"),
        ("ANCHOR_BOTTOMRIGHT", "TOPLEFT", "BOTTOMRIGHT"),
        ("ANCHOR_TOPLEFT", "BOTTOMLEFT", "TOPLEFT"),
        ("ANCHOR_TOP", "BOTTOM", "TOP"),
        ("ANCHOR_TOPRIGHT", "BOTTOMRIGHT", "TOPRIGHT"),
    ];
    for (mode, tooltip_point, owner_point) in expected {
        let (point, relative_point, x, y) = owner_anchor_point(&env, mode, false);
        assert_eq!(point, tooltip_point, "{mode} tooltip point");
        assert_eq!(relative_point, owner_point, "{mode} owner point");
        assert_eq!(
            (x, y),
            (0.0, 0.0),
            "{mode} must not add a default displacement"
        );
    }
}

/// O17 §1: explicit zero displacements stay zero; zero is not a sentinel for
/// the removed +20 default.
#[test]
fn tooltip_owner_anchor_explicit_zero_displacements_are_zero() {
    let env = env();
    for mode in [
        "ANCHOR_LEFT",
        "ANCHOR_RIGHT",
        "ANCHOR_BOTTOM",
        "ANCHOR_TOPRIGHT",
    ] {
        let (_, _, x, y) = owner_anchor_point(&env, mode, true);
        assert_eq!((x, y), (0.0, 0.0), "{mode} explicit zero offsets");
    }
}

/// O17 §1: absent, empty and unrecognized names select LEFT without
/// reporting an invalid anchor.
#[test]
fn tooltip_owner_anchor_absent_empty_and_unknown_names_select_left() {
    let env = env();
    let (point, relative_point, anchor_type): (String, String, String) = env
        .eval(
            r#"
            OwnerAnchorTip:SetOwner(OwnerAnchorFrame)
            local point, _, relativePoint = OwnerAnchorTip:GetPoint(1)
            local absentType = OwnerAnchorTip:GetAnchorType()
            OwnerAnchorTip:SetOwner(OwnerAnchorFrame, "")
            local emptyPoint, _, emptyRelative = OwnerAnchorTip:GetPoint(1)
            assert(point == emptyPoint and relativePoint == emptyRelative, "empty name must match the absent default")
            OwnerAnchorTip:SetOwner(OwnerAnchorFrame, "ANCHOR_NOT_A_MODE")
            local unknownPoint, _, unknownRelative = OwnerAnchorTip:GetPoint(1)
            assert(emptyPoint == unknownPoint and emptyRelative == unknownRelative, "unknown name must match the absent default")
            return unknownPoint, unknownRelative, OwnerAnchorTip:GetAnchorType()
            "#,
        )
        .unwrap();
    assert_eq!(point, "BOTTOMRIGHT", "absent anchor uses the LEFT pair");
    assert_eq!(
        relative_point, "TOPLEFT",
        "absent anchor uses the LEFT pair"
    );
    assert_eq!(anchor_type, "ANCHOR_LEFT", "stored mode is LEFT");
}

/// O17 §1 through the original `TotemButtonMixin:OnEnter` consumer, which
/// passes `ANCHOR_BOTTOMRIGHT` for a plain button owner.
#[test]
fn tooltip_owner_anchor_totem_button_enter_anchors_bottomright() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.state().borrow_mut().totem_slots =
        vec![Some(wow_ui_sim::lua_api::globals::real::totems::Totem {
            name: "Searing Totem".into(),
            start_time: 90.0,
            duration: 60.0,
            icon: 0,
            mod_rate: 1.0,
            spell_id: 19750,
        })];
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let source =
        std::fs::read_to_string(addons.join("Blizzard_UnitFrame/Mainline/TotemFrame.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        local button = CreateFrame("Button", "OwnerAnchorTotemButton", UIParent)
        button.slot = 1
        button:SetSize(32, 32)
        TotemButtonMixin.OnEnter(button)
        local point, relative, relativePoint, x, y = GameTooltip:GetPoint(1)
        assert(relative == button, "tooltip is owned by the totem button")
        assert(point == "TOPLEFT" and relativePoint == "BOTTOMRIGHT",
            "TotemFrame ANCHOR_BOTTOMRIGHT pair, got " .. tostring(point) .. "/" .. tostring(relativePoint))
        assert(x == 0 and y == 0, "totem tooltip must not add a default displacement")
        assert(GameTooltip:GetOwner() == button, "owner recorded")
        "#,
    )
    .unwrap();
}
