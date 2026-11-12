//! Show/hide smoke coverage for `Blizzard_AuctionHouseUI`.

use crate::common;

use common::auction_fixtures::auction_root;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_auction_house_does_not_publish_snapshot_only_hide_wrapper() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            let env = common::panel_fixtures::setup_env();
            load_auction_house_through_runtime(&env);
            env.apply_post_load_workarounds();

            let wrapper_hides_frame: bool = env
                .eval(
                    r#"
                    local originalHide = HideUIPanel
                    local originalLoaded = C_AddOns.IsAddOnLoaded
                    local hidAuctionHouse = false
                    C_AddOns.IsAddOnLoaded = function(name)
                        return name == "Blizzard_AuctionHouseUI"
                    end
                    HideUIPanel = function(frame)
                        hidAuctionHouse = frame == AuctionHouseFrame
                    end
                    AuctionHouseFrame = {}
                    local callable = type(HideAuctionHouseFrame) == "function"
                    HideAuctionHouseFrame()
                    HideUIPanel = originalHide
                    C_AddOns.IsAddOnLoaded = originalLoaded
                    return callable and hidAuctionHouse
                    "#,
                )
                .expect("auction house wrapper behavior should be queryable");
            assert!(
                wrapper_hides_frame,
                "PTR AuctionHouseUI bootstrap.lua:10-17 publishes a loaded-addon hide delegate"
            );
        });
    });
}

#[test]
fn auction_house_show_hide_round_trip_has_no_lua_errors() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            let root = auction_root();
            let env = common::panel_fixtures::setup_env();
            load_auction_house_through_runtime(&env);
            load_show_hide_prerequisites(&env);
            clear_recorded_lua_errors(&env);

            let result: String = env
                .eval(&common::auction_fixtures::select_native_frame(
                    r#"
                    if not AuctionHouseFrame then
                        return "missing_frame"
                    end

                    ShowUIPanel(AuctionHouseFrame)
                    if not AuctionHouseFrame:IsShown() then
                        return "show_failed"
                    end

                    HideUIPanel(AuctionHouseFrame)
                    if AuctionHouseFrame:IsShown() then
                        return "hide_failed"
                    end

                    return "ok"
                    "#,
                ))
                .expect("AuctionHouseFrame ShowUIPanel/HideUIPanel probe should run");
            assert_eq!("ok", result, "`{root}` show/hide result: {result}");

            let errors = recorded_lua_errors(&env);
            assert!(
                errors.is_empty(),
                "`{root}` show/hide emitted Lua errors:\n{}",
                errors.join("\n")
            );
        });
    });
}

fn load_show_hide_prerequisites(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        return;
    }
    env.exec(
        r#"
        local tokenLoaded, tokenReason = C_AddOns.LoadAddOn("Blizzard_TokenUI")
        assert(tokenLoaded, "Blizzard_TokenUI should load: " .. tostring(tokenReason))
        if ContainerFrameSettingsManager and not ContainerFrameSettingsManager.TokenTracker then
            ContainerFrameSettingsManager:OnAddonLoaded("Blizzard_TokenUI")
        end

        local wowTokenLoaded, wowTokenReason = C_AddOns.LoadAddOn("Blizzard_WowTokenUI")
        assert(wowTokenLoaded, "Blizzard_WowTokenUI should load: " .. tostring(wowTokenReason))

        if type(WowToken_IsWowTokenAuctionDialogShown) ~= "function" then
            function WowToken_IsWowTokenAuctionDialogShown()
                return false
            end
        end
        "#,
    )
    .expect("show/hide prerequisite addons should load");
    env.apply_post_load_workarounds();
}

fn load_auction_house_through_runtime(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let root = auction_root();
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        env.exec("A_Admin.ClearAuctionBrowseResults()")
            .expect("isolate native auction visibility from modern sample rows");
    }
    let (loaded, reason): (bool, Option<String>) = env
        .eval(&format!(r#"return C_AddOns.LoadAddOn("{root}")"#))
        .expect("C_AddOns.LoadAddOn should return");
    assert!(loaded, "`{root}` should load: {reason:?}");
}
