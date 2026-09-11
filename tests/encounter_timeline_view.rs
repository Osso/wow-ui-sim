#![cfg(feature = "client-ptr")]
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_view() -> WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui, &["Blizzard_EncounterTimeline"], &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_EncounterTimeline"));
    env
}

#[test]
fn encounter_tracks_real_blizzard_track_layout_and_view() {
    let env = load_view();
    env.exec(r#"
        T=C_EncounterTimeline
        local layout=CreateFromMixins(EncounterTimelineTrackLayoutMixin)
        layout:OnLoad(); layout:UpdateTrackList()
        assert(layout:GetTrackCount()==5)
        layout:SetTrackExtent(Enum.EncounterTimelineTrack.Short,150)
        layout:SetTrackExtent(Enum.EncounterTimelineTrack.Medium,150)
        layout:UpdateLayout()
        assert(layout:FindTrackForDuration(10).id==Enum.EncounterTimelineTrack.Short)
        assert(layout:FindTrackForDuration(20).id==Enum.EncounterTimelineTrack.Medium)
        local offset=layout:CalculateOffsetForDuration(10)
        assert(offset==offset and offset>=0 and offset<math.huge)
        EncounterTimeline:Show()
        view=EncounterTimeline.TrackView
        view:Show()
        id=T.AddScriptEvent({spellID=19750,iconFileID=135907,duration=10,maxQueueDuration=3})
        assert(view:HasEvent(id) and view:HasEventFrame(id))
        frame=view:GetEventFrame(id)
        assert(frame:GetEventTrack()==Enum.EncounterTimelineTrack.Short)
        assert(frame:GetEventTimeRemaining()==10)
        local color=frame:GetEventColor()
        assert(type(color.GetRGBA)=='function')
        local r,g,b,a=color:GetRGBA(); assert(a==1 and r>=0 and g>=0 and b>=0)
    "#).unwrap();
    env.fire_on_update(6.0).unwrap();
    env.exec(r#"
        assert(frame:GetEventTimeRemaining()==4)
        T.PauseScriptEvent(id)
        assert(T.GetEventTrack(id)==Enum.EncounterTimelineTrack.Indeterminate)
        T.ResumeScriptEvent(id)
        assert(T.GetEventTrack(id)==Enum.EncounterTimelineTrack.Short)
    "#).unwrap();
    env.fire_on_update(4.0).unwrap();
    env.exec("assert(T.GetEventTrack(id)==0 and frame:GetEventTimeRemaining()==0)").unwrap();
    env.exec("T.FinishScriptEvent(id)").unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(not view:HasEvent(id)); view:Hide(); assert(not view:HasAnyActiveEventFrames())").unwrap();
    let errors: Vec<_> = env.state().borrow().lua_errors.iter()
        .filter(|message| message.contains("EncounterTimeline"))
        .cloned().collect();
    assert!(errors.is_empty(), "timeline consumer errors: {errors:#?}");
}

#[test]
fn encounter_tracks_icon_mask_selection_clears_stale_slots() {
    let env=WowLuaEnv::new().unwrap();
    env.exec(r#"
        local T=C_EncounterTimeline
        local id=T.AddScriptEvent({spellID=19750,iconFileID=135907,duration=10,icons=128+256+1})
        local owner=CreateFrame('Frame')
        local textures={owner:CreateTexture(),owner:CreateTexture(),owner:CreateTexture()}
        T.SetEventIconTextures(id,128+256,textures)
        assert(textures[1]:GetAtlas()=='roleicon-tiny-tank')
        assert(textures[2]:GetAtlas()=='roleicon-tiny-healer')
        assert(textures[1]:GetAlpha()==1 and textures[2]:GetAlpha()==1 and textures[3]:GetAlpha()==0)
        T.SetEventIconTextures(id,1,textures)
        assert(textures[1]:GetTexture()==135907 and textures[1]:GetAlpha()==1)
        assert(textures[2]:GetAlpha()==0 and textures[3]:GetAlpha()==0)
        T.SetEventIconTextures(id,0,textures)
        for _,texture in ipairs(textures) do assert(texture:GetAlpha()==0 and texture:GetTexture()==nil) end
    "#).unwrap();
}
