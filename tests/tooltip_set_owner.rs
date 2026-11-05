//! `GameTooltip:SetOwner(frame, anchor, xOffset, yOffset)` anchor
//! handling contract.
//!
//! Named anchors (`ANCHOR_TOP` / `ANCHOR_LEFT` / `ANCHOR_TOPRIGHT` / …)
//! attach the tooltip to the owner with an inverted point pair.
//! `ANCHOR_PRESERVE` keeps existing anchors untouched.
//! `ANCHOR_CURSOR` currently falls through to "no anchor" — the
//! cursor-follow half of `SimState::collect_cursor_tooltip_positions`
//! exists but isn't wired up on the setter side. Keep the test so the
//! gap is visible when the wiring lands.
//!
//! `SetOwner` always updates `tooltip_owner_id`, regardless of the
//! anchor kind.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn setup_tooltip_and_owner(env: &WowLuaEnv) {
    env.exec(
        r#"
        _G.OwnerFrame = CreateFrame("Frame", "TooltipOwnerFrame", UIParent)
        OwnerFrame:SetSize(100, 50)
        OwnerFrame:SetPoint("CENTER", UIParent, "CENTER", 0, 0)
        _G.Tip = CreateFrame("GameTooltip", "SetOwnerProbeTip", UIParent, "GameTooltipTemplate")
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn set_owner_from_no_owner_hides_the_shown_tooltip() {
    let env = env();
    setup_tooltip_and_owner(&env);
    env.exec(
        r#"
        -- Permanent 1.60.1 does not reject an ownerless Show, so this fixture
        -- reaches the stored-nil-owner state WM106 §3 describes.
        GameTooltip:ClearLines()
        GameTooltip:Hide()
        GameTooltip:Show()
        assert(GameTooltip:GetOwner() == nil, 'fixture needs an ownerless tooltip')
        assert(GameTooltip:IsShown(), 'fixture needs the ownerless tooltip shown')
        -- WM106 §3: the non-null request differs from the stored owner, so Hide
        -- runs before the anchor mode is parsed, then the owner is installed.
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        assert(GameTooltip:GetOwner() == UIParent, 'consumer installs the new owner')
        assert(not GameTooltip:IsShown(),
            'SetOwner from a nil owner must hide the tooltip')
        "#,
    )
    .unwrap();
}

#[test]
fn set_owner_records_owner_id_for_any_anchor_kind() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let owners_match: bool = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_NONE")
            return Tip:GetOwner() == OwnerFrame
            "#,
        )
        .unwrap();
    assert!(owners_match);
}

#[test]
fn anchor_right_attaches_left_of_tooltip_to_right_of_owner() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let (point, relative_point, x, y): (String, String, f64, f64) = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_RIGHT", 7, -3)
            local p, rel, relp, x, y = Tip:GetPoint(1)
            -- `rel` is the owner frame; we compare by identity in the test wiring above.
            return p, relp, x, y
            "#,
        )
        .unwrap();
    // WM63 O17:20-30: mainline RIGHT uses bottom-left to top-right.
    if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    ) {
        assert_eq!(point, "BOTTOMLEFT");
        assert_eq!(relative_point, "TOPRIGHT");
    } else {
        assert_eq!(point, "LEFT");
        assert_eq!(relative_point, "RIGHT");
    }
    assert_eq!(x, 7.0);
    assert_eq!(y, -3.0);
}

#[test]
fn anchor_topleft_uses_inverted_corner_pair() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let (point, relative_point): (String, String) = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_TOPLEFT")
            local p, _, relp = Tip:GetPoint(1)
            return p, relp
            "#,
        )
        .unwrap();
    assert_eq!(point, "BOTTOMLEFT");
    assert_eq!(relative_point, "TOPLEFT");
}

#[test]
fn anchor_preserve_keeps_prior_anchor_in_place() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let (point, relative_point, x, y): (String, String, f64, f64) = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_TOP", 2, 9)
            Tip:SetOwner(OwnerFrame, "ANCHOR_PRESERVE", 999, 999)
            local p, _, relp, x, y = Tip:GetPoint(1)
            return p, relp, x, y
            "#,
        )
        .unwrap();
    assert_eq!(point, "BOTTOM");
    assert_eq!(relative_point, "TOP");
    assert_eq!(x, 2.0, "ANCHOR_PRESERVE must not overwrite xOffset");
    assert_eq!(y, 9.0, "ANCHOR_PRESERVE must not overwrite yOffset");
}

#[test]
fn anchor_none_clears_anchors_without_adding_new_ones() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let count: i64 = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_RIGHT", 0, 0)
            Tip:SetOwner(OwnerFrame, "ANCHOR_NONE")
            return Tip:GetNumPoints()
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

/// Documented gap: ANCHOR_CURSOR currently falls through to the
/// no-anchor branch. If this test starts failing because `GetNumPoints`
/// returns 1, the cursor-follow wiring has landed — delete this test
/// and the PLAN item.
#[test]
fn anchor_cursor_falls_through_to_no_anchor_for_now() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let count: i64 = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame, "ANCHOR_CURSOR", 5, 5)
            return Tip:GetNumPoints()
            "#,
        )
        .unwrap();
    assert_eq!(
        count, 0,
        "ANCHOR_CURSOR is an acknowledged gap — if this now returns 1, cursor-follow is wired up."
    );
}

#[test]
fn set_owner_without_explicit_anchor_defaults_to_no_anchor() {
    let env = env();
    setup_tooltip_and_owner(&env);
    let (count, has_owner): (i64, bool) = env
        .eval(
            r#"
            Tip:SetOwner(OwnerFrame)
            return Tip:GetNumPoints(), Tip:GetOwner() == OwnerFrame
            "#,
        )
        .unwrap();
    // WM63 O17:32: absent anchor defaults to LEFT on mainline.
    let expected = if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    ) {
        1
    } else {
        0
    };
    assert_eq!(count, expected, "default anchor count");
    assert!(has_owner, "owner is still recorded when anchor is omitted");
}
