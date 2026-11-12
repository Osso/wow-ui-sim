#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn covenant_toasts_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_CovenantToasts/Blizzard_CovenantToasts.toc")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn blizzard_covenant_toasts_toc_is_non_lod_standard_only() {
    let toc = TocFile::from_file(&covenant_toasts_toc())
        .expect("Blizzard_CovenantToasts TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_CovenantToasts declares `## LoadOnDemand: 0` — both toast frames \
         (CovenantChoiceToast and CovenantRenownToast) register for COVENANT_CHOSEN / \
         COVENANT_SANCTUM_RENOWN_LEVEL_CHANGED at OnLoad, so the addon must auto-load on \
         Game-screen bring-up to be ready when those events fire"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_CovenantToasts does not declare UseSecureEnvironment"
    );
    assert!(
        toc.dependencies().is_empty(),
        "Blizzard_CovenantToasts declares no `## Dependencies` — TopBannerManager_Show / \
         TopBannerManager_BannerFinished and the COVENANT_COLORS / RenownRewardUtil / \
         SetupTextureKitOnFrames helpers all live in Blizzard_FrameXML, which is the always-on \
         baseline"
    );
}

#[test]
fn blizzard_covenant_toasts_appears_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_CovenantToasts");
    assert!(
        in_game,
        "Blizzard_CovenantToasts (`## LoadOnDemand: 0`, `## AllowLoadGameType: standard`) \
         should appear in Game-screen auto-discovery so the COVENANT_CHOSEN / \
         COVENANT_SANCTUM_RENOWN_LEVEL_CHANGED listeners are wired before either event fires"
    );
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_CovenantToasts")
                || message.contains("CovenantChoiceToast")
                || message.contains("CovenantRenownToast")
                || message.contains("CovenantCelebrationBanner")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_CovenantToasts emitted Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_toplevel_frames_are_defined(env: &WowLuaEnv) {

    let frames_present: bool = env
        .eval(
            "return type(_G.CovenantChoiceToast) == 'table' \
                and CovenantChoiceToast:GetParent() == UIParent \
                and not CovenantChoiceToast:IsShown() \
                and type(_G.CovenantRenownToast) == 'table' \
                and CovenantRenownToast:GetParent() == UIParent \
                and not CovenantRenownToast:IsShown()",
        )
        .expect("toplevel frame query should succeed");
    assert!(
        frames_present,
        "Blizzard_CovenantToasts should define both toast frames after load: \
         `CovenantChoiceToast` (XML: parent=UIParent hidden=true mixin=CovenantChoiceToastMixin \
         inherits=CovenantCelebrationBannerTemplate) listening for COVENANT_CHOSEN, and \
         `CovenantRenownToast` (XML: parent=UIParent hidden=true mixin=CovenantRenownToastMixin \
         inherits=CovenantCelebrationBannerTemplate) listening for \
         COVENANT_SANCTUM_RENOWN_LEVEL_CHANGED"
    );
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_celebration_banner_template_is_registered(env: &WowLuaEnv) {
    let _ = env;

    assert!(
        wow_ui_sim::xml::get_template("CovenantCelebrationBannerTemplate").is_some(),
        "CovenantCelebrationBannerTemplate (Blizzard_CovenantToasts.xml line 3: \
         `<Frame virtual=\"true\" frameStrata=\"DIALOG\" mixin=\"CovenantCelebrationBannerMixin\">` \
         with GlowLineTop / GlowLineTopAdditive textures, IconSwirlModelScene + Icon child \
         frame, and an OnHide=method handler) should be registered in the template registry \
         after Blizzard_CovenantToasts loads — both CovenantChoiceToast and CovenantRenownToast \
         inherit it"
    );
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_celebration_banner_mixin_methods_are_defined(env: &WowLuaEnv) {

    let methods_present: bool = env
        .eval(
            "return type(CovenantCelebrationBannerMixin) == 'table' \
                and type(CovenantCelebrationBannerMixin.CancelIconSwirlEffects) == 'function' \
                and type(CovenantCelebrationBannerMixin.OnHide) == 'function' \
                and type(CovenantCelebrationBannerMixin.SetCovenantTextureKit) == 'function' \
                and type(CovenantCelebrationBannerMixin.AddSwirlEffects) == 'function'",
        )
        .expect("CovenantCelebrationBannerMixin query should succeed");
    assert!(
        methods_present,
        "CovenantCelebrationBannerMixin should expose its 4 methods (CancelIconSwirlEffects \
         clearing IconSwirlModelScene effects; OnHide chaining CancelIconSwirlEffects; \
         SetCovenantTextureKit applying `CovenantChoice-Celebration-%sCloudyLine` / \
         `-DetailLine` / `-Sigil` atlases via SetupTextureKitOnFrames + cancelling+re-adding \
         swirl FX; AddSwirlEffects iterating the Kyrian/Venthyr/NightFae/Necrolord swirl FX \
         table from the local `covenantSwirlEffects` and calling \
         IconSwirlModelScene:AddEffect(effect, self))"
    );
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_choice_toast_mixin_methods_are_defined(env: &WowLuaEnv) {

    let methods_present: bool = env
        .eval(
            "return type(CovenantChoiceToastMixin) == 'table' \
                and type(CovenantChoiceToastMixin.OnLoad) == 'function' \
                and type(CovenantChoiceToastMixin.OnEvent) == 'function' \
                and type(CovenantChoiceToastMixin.OnHide) == 'function' \
                and type(CovenantChoiceToastMixin.PlayCovenantChoiceToast) == 'function' \
                and type(CovenantChoiceToastMixin.PlayBanner) == 'function' \
                and type(CovenantChoiceToastMixin.StopBanner) == 'function' \
                and type(CovenantChoiceToastMixin.OnAnimFinished) == 'function'",
        )
        .expect("CovenantChoiceToastMixin query should succeed");
    assert!(
        methods_present,
        "CovenantChoiceToastMixin should expose its 7 methods (OnLoad registering \
         COVENANT_CHOSEN; OnEvent dispatching COVENANT_CHOSEN→PlayCovenantChoiceToast; OnHide \
         calling CovenantCelebrationBannerMixin.OnHide(self) + TopBannerManager_BannerFinished; \
         PlayCovenantChoiceToast looking up C_Covenants.GetCovenantData(covenantID) and calling \
         TopBannerManager_Show with name/covenantColor/textureKit/celebrationSoundKit; \
         PlayBanner setting Covenant name+color, applying texture kit, hiding all decoration \
         layers then playing ShowAnim; StopBanner stopping the anim and hiding; OnAnimFinished \
         hiding the toast)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_renown_toast_mixin_methods_are_defined(env: &WowLuaEnv) {

    let methods_present: bool = env
        .eval(
            "return type(CovenantRenownToastMixin) == 'table' \
                and type(CovenantRenownToastMixin.OnLoad) == 'function' \
                and type(CovenantRenownToastMixin.OnEvent) == 'function' \
                and type(CovenantRenownToastMixin.OnHide) == 'function' \
                and type(CovenantRenownToastMixin.AddSwirlEffects) == 'function' \
                and type(CovenantRenownToastMixin.ShowRenownLevelUpToast) == 'function' \
                and type(CovenantRenownToastMixin.SetupRewardVisuals) == 'function' \
                and type(CovenantRenownToastMixin.PlayBanner) == 'function' \
                and type(CovenantRenownToastMixin.OnMouseEnter) == 'function' \
                and type(CovenantRenownToastMixin.OnMouseLeave) == 'function' \
                and type(CovenantRenownToastMixin.OnHoldAnimStarted) == 'function' \
                and type(CovenantRenownToastMixin.RefreshTooltip) == 'function' \
                and type(CovenantRenownToastMixin.StopBanner) == 'function' \
                and type(CovenantRenownToastMixin.OnAnimFinished) == 'function'",
        )
        .expect("CovenantRenownToastMixin query should succeed");
    assert!(
        methods_present,
        "CovenantRenownToastMixin should expose its 13 methods (OnLoad registering \
         COVENANT_SANCTUM_RENOWN_LEVEL_CHANGED; OnEvent gating new>old AND new>1 then calling \
         ShowRenownLevelUpToast(C_Covenants.GetActiveCovenantID(), newRenownLevel); OnHide \
         chaining the celebration-banner OnHide + TopBannerManager_BannerFinished; \
         AddSwirlEffects override using IconSwirlModelScene:AddDynamicEffect; \
         ShowRenownLevelUpToast hiding CovenantRenownFrame and TopBannerManager_Show with the \
         {{covenantID/name/renownLevel/covenantColor/textureKit}} payload; SetupRewardVisuals \
         pulling C_CovenantSanctumUI.GetRenownRewardsForLevel + RenownRewardUtil and toggling \
         the RewardIcon/RewardIconRing visibility; PlayBanner setting Renown level format text, \
         GlowLineTopBottom atlas + per-textureKit SOUND_KIT_BY_TEXTURE_KIT \
         (default/milestone/final based on level==#levels or levelInfo.isMilestone) + playing \
         ShowAnim; OnMouseEnter pausing ShowAnim.HoldAlpha; OnMouseLeave fading and replaying; \
         OnHoldAnimStarted; RefreshTooltip composing the GameTooltip via \
         RenownRewardUtil.GetRenownRewardInfo / RENOWN_REWARD_CAPSTONE_TOOLTIP* / \
         RENOWN_REWARD_MILESTONE_TOOLTIP_TITLE; StopBanner; OnAnimFinished)"
    );
}
}

prefork_full_ui_case! {
fn retail_renown_tooltip_does_not_invent_the_path_of_ascension_reward(env: &WowLuaEnv) {
    env.exec(
        r#"
        local source = debug.getinfo(CovenantRenownToastMixin.RefreshTooltip, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_CovenantToasts/Blizzard_CovenantRenownToast.lua', 1, true))
        local rewardCalls = 0
        RenownRewardUtil.GetRenownRewardInfo = function()
            rewardCalls = rewardCalls + 1
            return 1, 'Path of Ascension', 'unverified reward'
        end
        RENOWN_REWARD_MILESTONE_TOOLTIP_TITLE = 'Milestone %d'
        local title
        GameTooltip = GameTooltip or {}
        GameTooltip.GetOwner = function() return CovenantRenownToast.RewardIconMouseOver end
        GameTooltip.Show = function() end
        GameTooltip_SetTitle = function(_, value) title = value end
        GameTooltip_AddNormalLine = function() end
        local toast = {
            bannerData = { covenantID = 1, renownLevel = 5 },
            isCapstone = false,
            RewardIconMouseOver = {},
        }
        CovenantRenownToast = toast
        CovenantRenownToastMixin.RefreshTooltip(toast)
        assert(rewardCalls == 0, 'zero accepted rows do not create a reward tooltip')
        assert(title == 'Milestone 5', 'zero accepted rows use the milestone title')
        "#,
    )
    .expect("Retail toast consumes the empty no-data reward list");
}
}

prefork_full_ui_case! {
fn blizzard_covenant_toasts_namespace_helper_is_defined(env: &WowLuaEnv) {

    let helper_present: bool = env
        .eval(
            "return type(CovenantChoiceToasts) == 'table' \
                and type(CovenantChoiceToasts.GetSwirlEffectsByTextureKit) == 'function' \
                and CovenantChoiceToasts.GetSwirlEffectsByTextureKit('Kyrian')[1] == 91 \
                and CovenantChoiceToasts.GetSwirlEffectsByTextureKit('Venthyr')[1] == 92 \
                and CovenantChoiceToasts.GetSwirlEffectsByTextureKit('NightFae')[1] == 93 \
                and CovenantChoiceToasts.GetSwirlEffectsByTextureKit('NightFae')[2] == 96 \
                and CovenantChoiceToasts.GetSwirlEffectsByTextureKit('Necrolord')[1] == 94",
        )
        .expect("CovenantChoiceToasts.GetSwirlEffectsByTextureKit query should succeed");
    assert!(
        helper_present,
        "Blizzard_CovenantToasts.lua line 1-13 should publish the global namespace \
         `CovenantChoiceToasts = {{}}` with `GetSwirlEffectsByTextureKit(textureKit)` indexing \
         the local `covenantSwirlEffects = {{Kyrian={{91}}, Venthyr={{92}}, \
         NightFae={{93,96}}, Necrolord={{94}}}}` table — both CovenantCelebrationBannerMixin \
         and the CovenantRenownToast override use this to attach per-covenant swirl FX to the \
         IconSwirlModelScene"
    );
}
}
