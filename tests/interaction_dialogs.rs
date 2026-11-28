//! Server dialogs (item text, petition, duel, instance countdowns)
//! driven through their original Blizzard frames with A_Admin host inputs.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const DIALOG_ROOTS: &[&str] = &[
    "Blizzard_UIPanels_Game",
    "Blizzard_StaticPopup_Game",
    "Blizzard_Game",
    // UIParentPanelManager.ShowUIPanel indexes CatalogShopInboundInterface.
    "Blizzard_CatalogShop",
];

fn run_cases(roots: &[&str], cases: &str, what: &str) {
    with_blizzard_addon_closure(roots, &[], |env, loaded| {
        assert!(
            loaded.iter().any(|addon| addon == roots[0]),
            "{what}: {loaded:?}"
        );
        env.exec(cases)
            .unwrap_or_else(|err| panic!("{what}: {err}"));
    });
}

const ITEM_TEXT_CASES: &str = r#"
assert(not pcall(A_Admin.OpenItemText, "", nil, nil, {"x"}, false), "reject empty item")
assert(not pcall(A_Admin.OpenItemText, "Letter", nil, nil, {}, false), "reject no pages")
A_Admin.OpenItemText("Worn Letter", nil, "Mother", {"Dear child,", "Come home."}, false)
assert(ItemTextFrame:IsShown(), "ITEM_TEXT_READY shows the frame")
assert(ItemTextGetItem() == "Worn Letter" and ItemTextGetCreator() == "Mother")
assert(ItemTextGetMaterial() == nil and not ItemTextIsFullPage())
assert(ItemTextCurrentPage:GetText() == "1", "page label " .. tostring(ItemTextCurrentPage:GetText()))
assert(ItemTextNextPageButton:IsShown() and not ItemTextPrevPageButton:IsShown())
assert(ItemTextFramePageBg:IsShown(), "nil material uses the Parchment page")
ItemTextNextPageButton:Click()
assert(ItemTextGetPage() == 2 and ItemTextGetText() == "Come home.")
assert(ItemTextCurrentPage:GetText() == "2" and ItemTextPrevPageButton:IsShown())
assert(not ItemTextNextPageButton:IsShown() and not ItemTextHasNextPage())
ItemTextPrevPageButton:Click()
assert(ItemTextGetPage() == 1 and ItemTextGetText() == "Dear child,")
A_Admin.OpenItemText("Stone Tablet", "Stone", nil, {"Here lies Ragnar."}, true)
assert(ItemTextGetItem() == "Stone Tablet" and ItemTextGetMaterial() == "Stone")
assert(ItemTextIsFullPage() and ItemTextGetCreator() == nil and ItemTextGetPage() == 1)
assert(ItemTextMaterialTopLeft:IsShown() and not ItemTextFramePageBg:IsShown())
assert(ItemTextMaterialTopLeft:GetTexture() ~= nil, "stone material texture")
local closed = 0
EventRegistry:RegisterFrameEventAndCallback("ITEM_TEXT_CLOSED", function() closed = closed + 1 end)
HideUIPanel(ItemTextFrame)
assert(closed == 1 and ItemTextGetItem() == nil and not ItemTextFrame:IsShown())
"#;

#[test]
#[cfg(feature = "client-retail")]
fn item_text_frame_pages_through_supplied_text() {
    run_cases(DIALOG_ROOTS, ITEM_TEXT_CASES, "ItemTextFrame");
}

const PETITION_CASES: &str = r#"
assert(not pcall(A_Admin.OpenPetition, "club", "X", "", 1, 2, "A", false, true, {}), "reject type")
A_Admin.OpenPetition("guild", "Knights of Dawn", "Join us", 4, 9, "Arthas", false, true, {"Jaina", "Uther"})
assert(PetitionFrame:IsShown(), "PETITION_SHOW opens the frame")
assert(PetitionFrameCharterName:GetText() == "Knights of Dawn")
assert(PetitionFrameMasterName:GetText() == "Arthas")
assert(PetitionFrameMemberName1:GetText() == "Jaina" and PetitionFrameMemberName2:GetText() == "Uther")
assert(PetitionFrameMemberName3:GetText() == NOT_YET_SIGNED and PetitionFrameMemberName4:IsShown())
assert(PetitionFrameSignButton:IsShown() and PetitionFrameSignButton:IsEnabled())
assert(not PetitionFrameRequestButton:IsShown())
local closed = 0
EventRegistry:RegisterFrameEventAndCallback("PETITION_CLOSED", function() closed = closed + 1 end)
A_Admin.OpenPetition("guild", "Silver Hand", "", 2, 2, "Me", true, false, {"Tirion", "Bolvar"})
assert(PetitionFrameRequestButton:IsShown() and not PetitionFrameRequestButton:IsEnabled())
assert(PetitionFrameMemberName2:GetText() == "Bolvar" and not PetitionFrameMemberName3:IsShown(),
    "rows beyond the new minSignatures are hidden")
assert(not PetitionFrameSignButton:IsShown() and not CanSignPetition())
assert(GetNumPetitionNames() == 2 and GetPetitionNameInfo(2) == "Bolvar" and GetPetitionNameInfo(3) == nil)
assert(select(7, GetPetitionInfo()) == 2 and select(6, GetPetitionInfo()) == true)
PetitionFrameCancelButton:Click()
assert(closed == 1 and GetPetitionInfo() == nil and GetNumPetitionNames() == 0)
"#;

#[test]
#[cfg(feature = "client-retail")]
fn petition_frame_shows_supplied_charter_and_closes_session() {
    run_cases(DIALOG_ROOTS, PETITION_CASES, "PetitionFrame");
}

const DUEL_CASES: &str = r#"
local finished = 0
EventRegistry:RegisterFrameEventAndCallback("DUEL_FINISHED", function() finished = finished + 1 end)
for _, challenger in ipairs({"Varian", "Garrosh"}) do
    A_Admin.RequestDuel(challenger)
    local dialog = StaticPopup_FindVisible("DUEL_REQUESTED")
    assert(dialog, "duel popup for " .. challenger)
    dialog.ButtonContainer.Button2:Click()
end
assert(finished == 2 and not StaticPopup_FindVisible("DUEL_REQUESTED"), "declines " .. finished)
CancelDuel()
assert(finished == 2, "no request left to decline")
"#;

#[test]
#[cfg(feature = "client-retail")]
fn duel_popup_declines_supplied_requests() {
    run_cases(DIALOG_ROOTS, DUEL_CASES, "duel popup");
}

const TIMER_CASES: &str = r##"
A_Admin.SetManualClock(50)
assert(GetInstanceBootTimeRemaining() == 0, "no countdowns")
assert(select("#", GetInstanceLockTimeRemainingEncounter(1)) == 0, "no lock encounters")
assert(not pcall(A_Admin.SetInstanceBoot, -1), "negative boot time")
A_Admin.SetInstanceBoot(90)
local boot = StaticPopup_FindVisible("INSTANCE_BOOT")
assert(boot and boot.timeleft == 90, "INSTANCE_BOOT_START opens the boot popup")
A_Admin.AdvanceTime(30)
assert(GetInstanceBootTimeRemaining() == 60)
A_Admin.SetInstanceBoot(nil)
assert(not StaticPopup_FindVisible("INSTANCE_BOOT"), "INSTANCE_BOOT_STOP hides it")
assert(not pcall(A_Admin.SetInstanceLock, 60, false, false, {{texture = "x"}}), "boss name")
A_Admin.SetInstanceLock(600, true, false, {
    {name = "Ragnaros", texture = "boss-rag", killed = true},
    {name = "Majordomo", killed = false, ineligible = true},
    {name = "Golemagg", killed = true},
})
local lock = StaticPopup_FindVisible("INSTANCE_LOCK")
assert(lock and not lock.data.enforceTime, "INSTANCE_LOCK_WARNING opens the warning")
assert(lock.data.encountersTotal == 3 and lock.data.encountersComplete == 2)
assert(lock.lockTimeleft == 600 and lock.isPreviousInstance == true)
InstanceLock_OnEnter(lock:GetTextFontString())
assert(GameTooltipTextLeft2:GetText() == "Ragnaros" and GameTooltipTextRight2:GetText() == BOSS_DEAD)
assert(GameTooltipTextLeft3:GetText() == "Majordomo" and GameTooltipTextRight3:GetText() == BOSS_ALIVE)
GameTooltip:Hide()
local name, texture, killed, ineligible = GetInstanceLockTimeRemainingEncounter(2)
assert(name == "Majordomo" and texture == "" and killed == false and ineligible == true)
assert(GetInstanceLockTimeRemainingEncounter(1) == "Ragnaros" and select(2, GetInstanceLockTimeRemainingEncounter(1)) == "boss-rag")
A_Admin.SetInstanceLock(nil)
assert(not StaticPopup_FindVisible("INSTANCE_LOCK"), "INSTANCE_LOCK_STOP hides it")
A_Admin.SetInstanceLock(45, false, true, {})
lock = StaticPopup_FindVisible("INSTANCE_LOCK")
assert(lock and lock.data.enforceTime and lock.lockTimeleft == 45, "INSTANCE_LOCK_START enforces")
assert(select(3, GetInstanceLockTimeRemaining()) == 0)
A_Admin.SetInstanceLock(nil)
"##;

#[test]
fn instance_popups_read_supplied_countdowns() {
    run_cases(DIALOG_ROOTS, TIMER_CASES, "instance countdown popups");
}
