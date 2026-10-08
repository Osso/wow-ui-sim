//! Bounded current behavior; historical/native item and mouse policy remain unclaimed.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{XmlElement, parse_xml, register_template};

prefork_full_ui_case! {
fn patch_7_1_0_clip_children_state_and_intrinsic_xml(env: &WowLuaEnv) {
    let ui = parse_xml(r#"
        <Ui>
            <Frame name="P710ClipIntrinsic" intrinsic="true" clipChildren="true">
                <Size x="91" y="47"/>
                <Scripts><OnLoad>self.clippedOnLoad = self:DoesClipChildren()</OnLoad></Scripts>
            </Frame>
            <Frame name="P710ClipOverride" virtual="true" inherits="P710ClipIntrinsic" clipChildren="false"/>
        </Ui>
    "#).expect("parse intrinsic and clipping XML");
    for element in ui.elements {
        if let XmlElement::Frame(frame) = element {
            let name = frame.name.clone().expect("named template");
            register_template(&name, "Frame", frame);
        }
    }
    env.exec(r#"
        local intrinsic = CreateFrame('P710ClipIntrinsic', nil, UIParent)
        assert(intrinsic:GetObjectType() == 'Frame')
        assert(intrinsic:GetWidth() == 91 and intrinsic:GetHeight() == 47)
        assert(intrinsic.clippedOnLoad == true and intrinsic:DoesClipChildren() == true)
        intrinsic:SetClipsChildren(false)
        assert(intrinsic:DoesClipChildren() == false)
        intrinsic:SetClipsChildren(true)
        assert(intrinsic:DoesClipChildren() == true)
        local override = CreateFrame('Frame', nil, UIParent, 'P710ClipOverride')
        assert(override:DoesClipChildren() == false)
        assert(override.clippedOnLoad == false)
        assert(intrinsic:DoesClipChildren() == true, 'frame clipping state must remain independent')
    "#).expect("intrinsic factory and inherited clipping before OnLoad");
}
}

prefork_full_ui_case! {
fn patch_7_1_0_item_metadata_successor_bounded_fields(env: &WowLuaEnv) {
    env.exec(r#"
        local info = { C_Item.GetItemInfo(6948) }
        assert(info[1] == 'Hearthstone')
        assert(info[14] == 1, 'Hearthstone is bind-on-pickup')
        assert(info[15] == 0, 'Hearthstone belongs to original WoW')
        local level = C_Item.GetDetailedItemLevelInfo(6948)
        assert(level == info[4] and level > 0, 'base item level comes from the same catalog row')
        local linkLevel = C_Item.GetDetailedItemLevelInfo(info[2])
        assert(linkLevel == level, 'item links select the same catalog identity')
    "#).expect("real catalog binding, expansion and base level through current successor APIs");
}
}
