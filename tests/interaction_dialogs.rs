//! Server dialogs (item text, guild invite, petition, duel, spirit healer)
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
        assert!(loaded.iter().any(|addon| addon == roots[0]), "{what}: {loaded:?}");
        env.exec(cases).unwrap_or_else(|err| panic!("{what}: {err}"));
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
fn petition_frame_shows_supplied_charter_and_closes_session() {
    run_cases(DIALOG_ROOTS, PETITION_CASES, "PetitionFrame");
}

const DUEL_AND_HEALER_CASES: &str = r#"
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
local function healer(seconds, cemetery)
    A_Admin.SetAreaSpiritHealer(seconds, cemetery)
    local dialog = StaticPopup_FindVisible("AREA_SPIRIT_HEAL")
    assert(dialog, "spirit healer popup")
    assert(math.abs(dialog.timeleft - seconds) < 0.5, "queued wave time " .. dialog.timeleft)
    assert(IsCemeterySelectionAvailable() == cemetery)
    assert(dialog.ButtonContainer.Button1:IsShown() == cemetery, "cemetery button")
    return dialog
end
healer(25, false)
local dialog = healer(12, true)
dialog.ButtonContainer.Button2:Click()
assert(GetAreaSpiritHealerTime() == 0, "cancel leaves the queue")
AcceptAreaSpiritHeal()
assert(math.abs(GetAreaSpiritHealerTime() - 12) < 0.5, "requeue")
A_Admin.SetAreaSpiritHealer(nil)
assert(GetAreaSpiritHealerTime() == 0 and not IsCemeterySelectionAvailable())
assert(not StaticPopup_FindVisible("AREA_SPIRIT_HEAL"), "out of range hides the popup")
"#;

#[test]
fn duel_and_spirit_healer_popups_answer_supplied_requests() {
    run_cases(DIALOG_ROOTS, DUEL_AND_HEALER_CASES, "duel/spirit healer popups");
}

const GUILD_INVITE_CASES: &str = r#"
assert(not pcall(A_Admin.ReceiveGuildInvite, "", "G", 0, "", false, "Initiate", 3), "reject inviter")
A_Admin.ReceiveGuildInvite("Thrall", "Earthen Ring", 1250, "", true, "Initiate", 40)
assert(GuildInviteFrame:IsShown() and GuildInviteFrameGuildName:GetText() == "Earthen Ring")
assert(GuildInviteFrameInviterName:GetText() == "Thrall")
GuildInviteFrameJoinButton:Click()
assert(IsInGuild(), "accepting joins the guild")
local name, rank = GetGuildInfo("player")
assert(name == "Earthen Ring" and rank == "Initiate", tostring(name) .. "/" .. tostring(rank))
A_Admin.LeaveGuild()
A_Admin.ReceiveGuildInvite("Garrosh", "Kor'kron", 90, "Earthen Ring", true, "Grunt", 12)
assert(GuildInviteFrameGuildName:GetText() == "Kor'kron")
GuildInviteFrameDeclineButton:Click()
assert(not GuildInviteFrame:IsShown())
AcceptGuild()
assert(not IsInGuild(), "a declined invite cannot be accepted")
"#;

#[test]
fn guild_invite_frame_joins_or_declines_supplied_invite() {
    run_cases(
        &["Blizzard_FrameXML", "Blizzard_StaticPopup_Game"],
        GUILD_INVITE_CASES,
        "GuildInviteFrame",
    );
}
