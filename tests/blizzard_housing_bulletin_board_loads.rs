#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn bulletin_board_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_HousingBulletinBoard")
}

fn bulletin_board_toc() -> PathBuf {
    bulletin_board_dir().join("Blizzard_HousingBulletinBoard.toc")
}

fn load_housing_bulletin_board(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &bulletin_board_toc())
        .expect("Blizzard_HousingBulletinBoard should load via explicit Rust loader call");
}

#[test]
fn blizzard_housing_bulletin_board_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&bulletin_board_dir())
        .expect("Blizzard_HousingBulletinBoard TOC should resolve");
    assert_eq!(
        resolved,
        bulletin_board_toc(),
        "Blizzard_HousingBulletinBoard ships exactly one bare TOC \
         (`Blizzard_HousingBulletinBoard.toc`) — no flavor variants. The neighborhood roster + \
         resident-invite UI only ships on retail (`## AllowLoadGameType: standard`) and uses the \
         bare TOC suffix that `find_toc_file` (src/loader/mod.rs:65) falls through to"
    );
}

#[test]
fn blizzard_housing_bulletin_board_toc_declares_lod_with_single_dependency() {
    let toc =
        TocFile::from_file(&bulletin_board_toc()).expect("HousingBulletinBoard TOC should parse");
    assert!(
        toc.is_load_on_demand(),
        "Blizzard_HousingBulletinBoard declares `## LoadOnDemand: 1` — the neighborhood roster \
         dialog only loads when the player interacts with a bulletin-board NPC. \
         `Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua` line 47 + 56 call \
         `C_AddOns.LoadAddOn(\"Blizzard_HousingBulletinBoard\")` from the player-interaction \
         dispatch handler when the interaction type matches the bulletin board"
    );
    assert!(
        !toc.is_load_first(),
        "Blizzard_HousingBulletinBoard does not declare `## LoadFirst: 1` — LoadOnDemand \
         precludes any load-order priority"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_HousingBulletinBoard does not declare `## UseSecureEnvironment` — runs in the \
         standard Lua environment"
    );
    assert_eq!(
        toc.dependencies(),
        vec!["Blizzard_HousingTemplates".to_string()],
        "Blizzard_HousingBulletinBoard declares exactly one `## Dependencies:` entry — \
         Blizzard_HousingTemplates provides the housing-themed atlas references \
         (`housing-wood-frame`, `housing-dashboard-bg-elwynn` + the neighborhood-specific \
         `housing-dashboard-bg-*` family selected by \
         `C_HousingNeighborhood.GetCurrentNeighborhoodTextureSuffix()`, \
         `housing-basic-panel-gradient-header-bg`, `housing-basic-panel-footer`, `housing-woodsign`, \
         `housing-decorative-foliage-{{left,right,small-left}}`, \
         `housing-bulletinboard-list-item-bg-{{light,dark}}`) and the C_HousingNeighborhood / \
         C_Housing API surfaces consumed by the roster + invite + rename flows"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Blizzard_HousingBulletinBoard declares NO `## SavedVariables*` — neighborhood roster + \
         pending-invites state are server-authoritative (synced via C_HousingNeighborhood \
         RPCs and the UPDATE_BULLETIN_BOARD_ROSTER / UPDATE_BULLETIN_BOARD_ROSTER_STATUSES / \
         UPDATE_BULLETIN_BOARD_MEMBER_TYPE / NEIGHBORHOOD_INFO_UPDATED / \
         NEIGHBORHOOD_INVITE_RESPONSE / CANCEL_NEIGHBORHOOD_INVITE_RESPONSE / \
         PENDING_NEIGHBORHOOD_INVITES_RECIEVED / NEIGHBORHOOD_NAME_VALIDATED event family), no \
         per-installation persistence is needed"
    );
}

#[test]
fn blizzard_housing_bulletin_board_toc_is_retail_only_and_omits_allow_load() {
    let toc =
        TocFile::from_file(&bulletin_board_toc()).expect("HousingBulletinBoard TOC should parse");
    let toc_text = std::fs::read_to_string(bulletin_board_toc()).expect("TOC should read");
    assert!(
        toc_text.contains("## AllowLoadGameType: standard"),
        "Blizzard_HousingBulletinBoard declares `## AllowLoadGameType: standard` — the \
         neighborhood bulletin board is a Midnight expansion feature that only ships on retail. \
         `is_game_type_restricted()` (src/toc.rs:294) treats `standard` and `mainline` as the \
         unrestricted retail flavor, so this addon is NOT considered game-type-restricted"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "Blizzard_HousingBulletinBoard must NOT be game-type restricted — \
         `## AllowLoadGameType: standard` matches the retail flavor that the simulator runs as"
    );
    assert!(
        !toc_text.contains("## AllowLoad:"),
        "Blizzard_HousingBulletinBoard omits `## AllowLoad:` — LoadOnDemand precludes \
         auto-discovery gating, so the AllowLoad value would be inert. The addon is pulled \
         exclusively via the Lua-side LoadAddOn(\"Blizzard_HousingBulletinBoard\") path"
    );
    assert!(
        !toc_text.contains("## DefaultState:"),
        "Blizzard_HousingBulletinBoard omits `## DefaultState:` — relies on the loader's \
         implicit-enabled default for Blizzard prefix LoD addons"
    );
}

#[test]
fn blizzard_housing_bulletin_board_toc_lists_bootstrap_then_three_files() {
    let toc =
        TocFile::from_file(&bulletin_board_toc()).expect("HousingBulletinBoard TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(
        files,
        vec![
            "Blizzard_HousingBulletinBoard_Bootstrap.lua".to_string(),
            "Blizzard_HousingBulletinBoard.lua".to_string(),
            "Blizzard_HousingBulletinBoard.xml".to_string(),
            "Blizzard_HousingBulletinBoardRegistration.lua".to_string(),
        ],
        "Blizzard_HousingBulletinBoard TOC lists its bootstrap, Lua, XML, and registration \
         tail in published order."
    );
}

#[test]
fn blizzard_housing_bulletin_board_directory_holds_five_entries() {
    let dir = bulletin_board_dir();
    let entries: Vec<String> = std::fs::read_dir(&dir)
        .expect("Blizzard_HousingBulletinBoard directory should exist")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        entries.len(),
        5,
        "Blizzard_HousingBulletinBoard directory ships its bootstrap, three TOC-listed source \
         files, and TOC. Got: {entries:?}"
    );
    assert!(
        entries.contains(&"Blizzard_HousingBulletinBoard.toc".to_string()),
        "Blizzard_HousingBulletinBoard directory must contain the bare TOC file"
    );
    assert!(
        entries.contains(&"Blizzard_HousingBulletinBoardRegistration.lua".to_string()),
        "Blizzard_HousingBulletinBoard directory must contain the Registration tail file"
    );
}

#[test]
fn blizzard_housing_bulletin_board_excluded_from_all_screen_auto_discovery_passes() {
    let ui = blizzard_ui_dir();
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let discovered = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_HousingBulletinBoard");
        assert!(
            !discovered,
            "Blizzard_HousingBulletinBoard MUST NOT appear in {screen:?} auto-discovery — \
             `## LoadOnDemand: 1` keeps it out of every screen pass. The only consumer \
             (Blizzard_UIPanels_Game/Shared/PlayerInteractionFrameManager.lua) calls \
             `C_AddOns.LoadAddOn(\"Blizzard_HousingBulletinBoard\")` from the \
             PLAYER_INTERACTION_MANAGER_FRAME_SHOW handler — never via `## RequiredDep:`, so \
             the LoD-pull promotion path in `pull_required_lod_addons` (src/loader/mod.rs:357) \
             does not escalate it onto any auto-discovery pass"
        );
    }
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    let lua_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let related: Vec<&String> = lua_errors
        .iter()
        .filter(|e| {
            e.contains("Blizzard_HousingBulletinBoard/")
                || e.contains("Blizzard_HousingBulletinBoard\\")
                || e.contains("HousingBulletinBoardFrameMixin")
                || e.contains("NeighborhoodRosterMixin")
                || e.contains("NeighborhoodRosterEntryMixin")
                || e.contains("HousingInviteResidentFrameMixin")
                || e.contains("NeighborhoodChangeNameDialogMixin")
        })
        .collect();
    assert!(
        related.is_empty(),
        "Blizzard_HousingBulletinBoard emitted addon-specific Lua errors during explicit LoD \
         load:\n  {}",
        related
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_is_addon_loaded_returns_true_after_explicit_lod_load(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingBulletinBoard')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        loaded,
        "After explicit `load_addon` of Blizzard_HousingBulletinBoard.toc following \
         Game-screen auto-discovery, \
         `C_AddOns.IsAddOnLoaded('Blizzard_HousingBulletinBoard')` should return true"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_publishes_three_named_frames_globally(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for frame in [
        "HousingBulletinBoardFrame",
        "HousingInviteResidentFrame",
        "NeighborhoodChangeNameDialog",
    ] {
        let exists: bool = env
            .eval(&format!(
                "local f = _G['{frame}']; return type(f) == 'table' and type(f.GetName) == 'function'"
            ))
            .expect("named frame global lookup should succeed");
        assert!(
            exists,
            "After LoD load, `{frame}` should publish as a global frame instance — \
             Blizzard_HousingBulletinBoard.xml declares 3 named non-virtual frames at file \
             scope: HousingBulletinBoardFrame (line 382, inherits TabSystemOwnerTemplate, the \
             umbrella tab system that hosts the ResidentsTab roster panel), \
             HousingInviteResidentFrame (line 219, the player-search/pending-invite list panel \
             that pops in via ShowUIPanel from the InviteResidentClicked handler), and \
             NeighborhoodChangeNameDialog (line 468, inherits TranslucentFrameTemplate at \
             frameStrata=DIALOG, the rename-confirmation dialog driven by the \
             NEIGHBORHOOD_NAME_VALIDATED event)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_publishes_eight_mixins_globally(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for mixin in [
        "HousingBulletinBoardFrameMixin",
        "BulletinBoardColumnDisplayMixin",
        "NeighborhoodRosterMixin",
        "NeighborhoodRosterEntryMixin",
        "HousingInviteResidentFrameMixin",
        "HousingInviteSearchBoxMixin",
        "NeighborhoodChangeNameDialogMixin",
        "NeighborhoodChangeNameCostMixin",
    ] {
        let exists: bool = env
            .eval(&format!("return type(_G['{mixin}']) == 'table'"))
            .expect("mixin existence query should succeed");
        assert!(
            exists,
            "Blizzard_HousingBulletinBoard must publish `_G['{mixin}']` — the addon publishes 8 \
             mixins covering distinct subsystems: HousingBulletinBoardFrameMixin (umbrella tab \
             frame, dispatches NEIGHBORHOOD_INFO_UPDATED, manages GearDropdown visibility based \
             on Charter/Guild owner type, Background atlas swap from \
             GetCurrentNeighborhoodTextureSuffix); BulletinBoardColumnDisplayMixin (extends \
             ColumnDisplayMixin via CreateFromMixins — overrides OnLoad to wire \
             BulletinBoardColumnDisplayButtonTemplate as the column-header pool); \
             NeighborhoodRosterMixin (the residents tab — drives sortable name/status/plot/\
             subdivision columns, alphabetical+sortedMemberList shadow copies, \
             UPDATE_BULLETIN_BOARD_ROSTER + ROSTER_STATUSES + MEMBER_TYPE event handling, \
             eviction/manager/owner StaticPopup chain); NeighborhoodRosterEntryMixin (per-row \
             scroll entry — alternates light/dark atlas backgrounds, RankIcon for Owner / \
             Manager via Enum.ResidentType, RightButton opens UnitPopup_OpenMenu \
             \"NEIGHBORHOOD_ROSTER\" with the resident contextData); \
             HousingInviteResidentFrameMixin (player-search invite panel — pendingInvitesPool \
             backed by PendingInviteTemplate, NEIGHBORHOOD_INVITE_RESPONSE / \
             CANCEL_NEIGHBORHOOD_INVITE_RESPONSE / PENDING_NEIGHBORHOOD_INVITES_RECIEVED [sic] \
             event handling, error string lookup via Enum.NeighborhoodInviteResult); \
             HousingInviteSearchBoxMixin (AutoCompleteEditBox with C_AutoComplete \
             ALL_CHARS source, OnEnterPressed delegates to OnSendInviteClicked when not \
             auto-completing, FillText placeholder toggles invite button enabled state); \
             NeighborhoodChangeNameDialogMixin (rename dialog — \
             C_PlayerInteractionManager.ClearInteraction(Enum.PlayerInteractionType.\
             RenameNeighborhood) on OnHide, C_Housing.ValidateNeighborhoodName + \
             TryRenameNeighborhood on confirm, NEIGHBORHOOD_NAME_VALIDATED dispatch); \
             NeighborhoodChangeNameCostMixin (rename token cost icon, item ID 234128, GameTooltip \
             SetItemByID on hover)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_frame_mixin_publishes_six_methods(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for method in [
        "OnEvent",
        "OnShow",
        "OnHide",
        "OnNeighborhoodInfoUpdated",
        "ReportNeighborhood",
        "GetRosterFrame",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingBulletinBoardFrameMixin['{method}']) == 'function'"
            ))
            .expect("HousingBulletinBoardFrameMixin method existence query should succeed");
        assert!(
            exists,
            "HousingBulletinBoardFrameMixin must expose `:{method}()` — the umbrella mixin \
             drives the bulletin-board frame: OnEvent dispatches NEIGHBORHOOD_INFO_UPDATED to \
             OnNeighborhoodInfoUpdated; OnShow registers the event, hides GearDropdown until \
             info arrives, calls C_HousingNeighborhood.RequestNeighborhoodInfo, shows \
             ResidentsTab, plays SOUNDKIT.HOUSING_BULLETIN_BOARD_OPEN; OnHide calls \
             C_HousingNeighborhood.OnBulletinBoardClosed and plays the close sound; \
             OnNeighborhoodInfoUpdated copies neighborhoodName / neighborhoodOwnerType, swaps \
             Background atlas to `housing-dashboard-bg-` plus the suffix from \
             GetCurrentNeighborhoodTextureSuffix, gates GearDropdown visibility on \
             Enum.NeighborhoodOwnerType.{{Charter, Guild}}, wires the report button via \
             SetupMenu + GenerateClosure; ReportNeighborhood instantiates a \
             ReportInfo:CreateNeighborhoodReportInfo with Enum.ReportType.Neighborhood and \
             calls ReportFrame:InitiateReport; GetRosterFrame returns the ResidentsTab child"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_neighborhood_roster_mixin_publishes_fifteen_methods(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for method in [
        "OnLoad",
        "OnEvent",
        "UpdateRosterMembers",
        "UpdateRosterMember",
        "ShouldShowSubdivision",
        "SetAlphabeticalSortedMemberList",
        "CopyAlphabeticalMemberList",
        "UpdateRoster",
        "SortByColumnIndex",
        "OnShow",
        "OnHide",
        "OnNeighborhoodInfoUpdated",
        "InviteResidentClicked",
        "TryEvictResident",
        "ConfirmEviction",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(NeighborhoodRosterMixin['{method}']) == 'function'"
            ))
            .expect("NeighborhoodRosterMixin method existence query should succeed");
        assert!(
            exists,
            "NeighborhoodRosterMixin must expose `:{method}()` — the residents tab mixin drives \
             the sortable roster panel covering eviction / manager promotion / ownership \
             transfer flows. The 4 sort attributes (name / status / plot / subdivision) drive \
             distinct sort paths in SortByColumnIndex, with the subdivision column only \
             visible when ShouldShowSubdivision returns true (private guild neighborhoods only \
             — Enum.NeighborhoodOwnerType.Guild)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_neighborhood_roster_management_methods_publish(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for method in [
        "TryAddManager",
        "ConfirmAddManager",
        "TryRemoveManager",
        "ConfirmRemoveManager",
        "TryTransferOwnership",
        "ConfirmTransferOwnership",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(NeighborhoodRosterMixin['{method}']) == 'function'"
            ))
            .expect("NeighborhoodRosterMixin management method existence query should succeed");
        assert!(
            exists,
            "NeighborhoodRosterMixin must expose `:{method}()` — the manager/owner workflow \
             pairs 3 try-then-confirm sequences: TryAddManager + ConfirmAddManager call \
             C_HousingNeighborhood.PromoteToManager(playerGUID); TryRemoveManager + \
             ConfirmRemoveManager call C_HousingNeighborhood.DemoteToResident(playerGUID); \
             TryTransferOwnership + ConfirmTransferOwnership call \
             C_HousingNeighborhood.TransferNeighborhoodOwnership(playerGUID). Each Try* \
             stashes the pending GUID on self and triggers the corresponding StaticPopup_Show \
             which routes OnAccept back to the matching Confirm* method via \
             HousingBulletinBoardFrame:GetRosterFrame()"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_neighborhood_roster_entry_mixin_publishes_six_methods(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for method in [
        "OnShow",
        "OnHide",
        "OnEvent",
        "OnClick",
        "Init",
        "UpdateRank",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(NeighborhoodRosterEntryMixin['{method}']) == 'function'"
            ))
            .expect("NeighborhoodRosterEntryMixin method existence query should succeed");
        assert!(
            exists,
            "NeighborhoodRosterEntryMixin must expose `:{method}()` — the per-row mixin powers \
             each scroll-box entry: OnShow / OnHide / OnEvent wire the (currently empty) \
             NEIGHBORHOOD_ROSTER_ENTRY_EVENTS table; OnClick on RightButton builds the \
             contextData record (guid, name, plotID, subdivision, targetResidentType, \
             playerIsOwner, playerIsManager, canBeManaged) and routes through \
             UnitPopup_OpenMenu(\"NEIGHBORHOOD_ROSTER\", contextData); Init populates Plot / \
             NameFrame.Name / Status / Subdivision text, alternates light/dark atlas based on \
             GetOrderIndex parity, colors text via HIGHLIGHT_FONT_COLOR (online) or \
             DISABLED_FONT_COLOR (offline); UpdateRank shows the GroupFrame leader/assistant \
             icon for Enum.ResidentType.Owner / Manager, hides for plain Resident. \
             UpdateNameFrame is a layout-only helper that doesn't need direct verification \
             since it's covered by the Init call path"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_invite_resident_mixin_publishes_eleven_methods(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for method in [
        "OnLoad",
        "OnShow",
        "OnHide",
        "OnEvent",
        "UpdatePendingInvitesList",
        "AddPendingInvite",
        "RemovePendingInvite",
        "CancelRemovePendingInvite",
        "SetInviteEnabled",
        "OnSendInviteClicked",
        "CancelInviteClicked",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingInviteResidentFrameMixin['{method}']) == 'function'"
            ))
            .expect("HousingInviteResidentFrameMixin method existence query should succeed");
        assert!(
            exists,
            "HousingInviteResidentFrameMixin must expose `:{method}()` — the invite panel mixin \
             drives the player-search → C_HousingNeighborhood.InvitePlayerToNeighborhood → \
             pending-invites-list flow. OnLoad creates the pendingInvitesPool against \
             PendingInviteTemplate; OnEvent handles 3 events (NEIGHBORHOOD_INVITE_RESPONSE \
             routes success/error via NeighborhoodInviteErrorTypeStrings keyed on \
             Enum.NeighborhoodInviteResult; CANCEL_NEIGHBORHOOD_INVITE_RESPONSE removes or \
             un-cancels the row; PENDING_NEIGHBORHOOD_INVITES_RECIEVED rebuilds the list); \
             UpdatePendingInvitesList rebuilds via pool ReleaseAll + AddPendingInvite per \
             entry; AddPendingInvite handles the duplicate-name early-return; \
             RemovePendingInvite + CancelRemovePendingInvite manage the pool slots; \
             SetInviteEnabled toggles the SendInviteButton; OnSendInviteClicked trims the \
             search box via StringUtil.RemoveTrailingSpaces + capitalizes via \
             C_CharacterServices.CapitalizeCharName before calling \
             InvitePlayerToNeighborhood; CancelInviteClicked triggers the \
             HOUSING_BULLETIN_CONFIRM_CANCEL_NEIGHBORHOOD_INVITE StaticPopup"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_publishes_two_module_event_tables_globally(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    let bulletin_table_size: i64 = env
        .eval("return BULLETIN_BOARD_SHOWING_EVENTS and #BULLETIN_BOARD_SHOWING_EVENTS or -1")
        .expect("BULLETIN_BOARD_SHOWING_EVENTS query should succeed");
    assert_eq!(
        bulletin_table_size, 1,
        "BULLETIN_BOARD_SHOWING_EVENTS is a module-level global table holding exactly 1 entry — \
         `NEIGHBORHOOD_INFO_UPDATED`. HousingBulletinBoardFrameMixin:OnShow registers it via \
         FrameUtil.RegisterFrameForEvents, OnHide unregisters. Drives the show/hide-scoped \
         neighborhood-info refresh"
    );

    let roster_entry_table_size: i64 = env
        .eval("return NEIGHBORHOOD_ROSTER_ENTRY_EVENTS and #NEIGHBORHOOD_ROSTER_ENTRY_EVENTS or -1")
        .expect("NEIGHBORHOOD_ROSTER_ENTRY_EVENTS query should succeed");
    assert_eq!(
        roster_entry_table_size, 0,
        "NEIGHBORHOOD_ROSTER_ENTRY_EVENTS is a module-level global table — currently empty \
         (the source has a `--TODO: set up events to update a single entry rather than the \
         entire neighborhood roster for evicting / adding managers` comment marking this as \
         intentional placeholder for future per-row event wiring). \
         NeighborhoodRosterEntryMixin:OnShow / OnHide still call \
         FrameUtil.{{Register,Unregister}}FrameForEvents against the empty table — correct \
         no-op behavior"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_publishes_global_column_click_callback(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    let exists: bool = env
        .eval("return type(_G['HousingBulletinBoardRosterColumnDisplay_OnClick']) == 'function'")
        .expect("global free function lookup should succeed");
    assert!(
        exists,
        "Blizzard_HousingBulletinBoard publishes the free function \
         `HousingBulletinBoardRosterColumnDisplay_OnClick(self, columnIndex)` (line 218) — XML \
         column-button OnClick handlers reference this global by name to forward into \
         `HousingBulletinBoardFrame.ResidentsTab:SortByColumnIndex(columnIndex)`. Free \
         functions are needed here because XML inline scripts cannot capture local Lua values \
         at load time"
    );
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_registers_five_static_popup_dialogs(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for dialog in [
        "HOUSING_BULLETIN_EVICT_CONFIRMATION",
        "HOUSING_BULLETIN_ADD_MANAGER_CONFIRMATION",
        "HOUSING_BULLETIN_REMOVE_MANAGER_CONFIRMATION",
        "HOUSING_BULLETIN_TRANSFER_OWNER_CONFIRMATION",
        "HOUSING_BULLETIN_CONFIRM_CANCEL_NEIGHBORHOOD_INVITE",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(StaticPopupDialogs['{dialog}']) == 'table' and type(StaticPopupDialogs['{dialog}'].OnAccept) == 'function'"
            ))
            .expect("StaticPopupDialogs entry lookup should succeed");
        assert!(
            exists,
            "Blizzard_HousingBulletinBoard registers `StaticPopupDialogs['{dialog}']` with an \
             OnAccept handler — the addon publishes 5 confirmation dialogs covering the \
             destructive neighborhood actions: EVICT (TryEvictResident → ConfirmEviction → \
             C_HousingNeighborhood.TryEvictPlayer), ADD_MANAGER (TryAddManager → \
             ConfirmAddManager → PromoteToManager), REMOVE_MANAGER (TryRemoveManager → \
             ConfirmRemoveManager → DemoteToResident), TRANSFER_OWNER (TryTransferOwnership → \
             ConfirmTransferOwnership → TransferNeighborhoodOwnership), \
             CONFIRM_CANCEL_NEIGHBORHOOD_INVITE (CancelInviteClicked → OnAccept → \
             CancelInviteToNeighborhood, with OnCancel that resets the row's loading spinner + \
             enables the RemoveButton + plays the cancel sound)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_does_not_publish_virtual_templates_globally(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for template in [
        "NeighborhoodRosterEntryTemplate",
        "BulletinBoardColumnDisplayButtonTemplate",
        "BulletinBoardColumnDisplayTemplate",
        "NeighborhoodRosterTemplate",
        "PendingInviteTemplate",
    ] {
        let published: bool = env
            .eval(&format!("return _G['{template}'] ~= nil"))
            .expect("virtual template global lookup should succeed");
        assert!(
            !published,
            "Virtual XML templates must NOT publish to `_G`. \
             Blizzard_HousingBulletinBoard.xml declares 5 virtual templates: \
             NeighborhoodRosterEntryTemplate (Button registerForClicks=RightButtonUp, \
             instantiated per row by the WowScrollBoxList view), \
             BulletinBoardColumnDisplayButtonTemplate (Button column header), \
             BulletinBoardColumnDisplayTemplate (Frame inheriting ColumnDisplay), \
             NeighborhoodRosterTemplate (Frame applied to HousingBulletinBoardFrame's \
             ResidentsTab via parentKey inheritance), PendingInviteTemplate (Frame \
             instantiated by HousingInviteResidentFrameMixin's pendingInvitesPool). All 5 stay \
             nil at `_G` proving the loader honors `virtual=\"true\"`"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_frame_publishes_named_children(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    for parent_key in [
        "ResidentsTab",
        "RosterTabButton",
        "FoliageDecoration",
        "GearDropdown",
        "Border",
        "Background",
        "Header",
        "Footer",
    ] {
        let exists: bool = env
            .eval(&format!(
                "return type(HousingBulletinBoardFrame['{parent_key}']) == 'table' and type(HousingBulletinBoardFrame['{parent_key}'].GetName) == 'function'"
            ))
            .expect("HousingBulletinBoardFrame parentKey child lookup should succeed");
        assert!(
            exists,
            "HousingBulletinBoardFrame.{parent_key} must publish via `parentKey` — the umbrella \
             frame wires 8 non-named children: ResidentsTab (NeighborhoodRosterTemplate, the \
             roster panel), RosterTabButton (custom-Texture wood-sign tab button at top-left), \
             FoliageDecoration (a visual-only Frame holding 3 atlas-sized textures), \
             GearDropdown (UIPanelIconDropdownButtonTemplate, gated visible only for \
             Charter/Guild owner types and wired with the report-neighborhood button), Border \
             (housing-wood-frame texture), Background (housing-dashboard-bg-elwynn default \
             swapped via OnNeighborhoodInfoUpdated), Header (gradient header bar), Footer \
             (panel footer)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_housing_bulletin_board_dependency_loads_via_game_screen_pass(env: &WowLuaEnv) {
    load_housing_bulletin_board(env);

    let templates_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_HousingTemplates')")
        .expect("IsAddOnLoaded query should succeed");
    assert!(
        templates_loaded,
        "Blizzard_HousingTemplates must be loaded by the Game-screen auto-discovery pass before \
         the explicit HousingBulletinBoard LoD load runs. HousingTemplates is the only \
         `## Dependencies` entry on the bulletin board's TOC, and the test harness's full \
         Game-screen pass hits it via the normal discovery flow because HousingTemplates is \
         itself non-LoD"
    );
}
}
