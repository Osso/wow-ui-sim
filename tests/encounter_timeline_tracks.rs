use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        T = C_EncounterTimeline
        function add(duration, extra)
            local request = {spellID=19750, iconFileID=135907, duration=duration}
            for k,v in pairs(extra or {}) do request[k]=v end
            return T.AddScriptEvent(request)
        end
    "#).unwrap();
    env
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_tracks_queue_hold_and_transition_callbacks() {
    let env = environment();
    env.exec(r#"
        assert(Enum.EncounterTimelineViewType.None==0 and Enum.EncounterTimelineViewType.Timeline==1 and Enum.EncounterTimelineViewType.Bars==2)
        assert(T.GetViewType() == Enum.EncounterTimelineViewType.Timeline)
        local tracks = T.GetTrackList()
        assert(#tracks == 5 and tracks[1].id == 0 and tracks[5].id == 4)
        assert(tracks[1].type == Enum.EncounterTimelineTrackType.Sorted)
        assert(tracks[2].minimumDuration == 0 and tracks[2].maximumDuration == 15)
        assert(tracks[3].minimumDuration == 15 and tracks[3].maximumDuration == 60)
        assert(tracks[5].type == Enum.EncounterTimelineTrackType.Hidden)
        assert(tracks[1].maximumEventCount==3 and tracks[4].maximumEventCount==3)
        assert(tracks[1].minimumDuration==0 and tracks[1].maximumDuration==0)
        for _,track in ipairs(tracks) do
            assert(track.minimumEventIntroDuration==0 and track.minimumEventGapDuration==0)
            assert(T.GetTrackType(track.id)==track.type)
        end
        assert(T.GetTrackMaxEventDuration(1) == 15 and T.GetTrackMaxEventDuration(2) == 60)
        tracks[2].maximumDuration = 999
        assert(T.GetTrackInfo(1).maximumDuration == 15)
        changes = {}; highlights = 0
        listener = CreateFrame('Frame')
        listener:RegisterEvent('ENCOUNTER_TIMELINE_EVENT_TRACK_CHANGED')
        listener:RegisterEvent('ENCOUNTER_TIMELINE_EVENT_HIGHLIGHT')
        listener:SetScript('OnEvent', function(_,event,eventID)
            assert(T.GetEventInfo(eventID))
            if event == 'ENCOUNTER_TIMELINE_EVENT_TRACK_CHANGED' then
                changes[#changes+1] = T.GetEventTrack(eventID)
            else highlights = highlights + 1 end
        end)
        id = add(16, {maxQueueDuration=3}); timer=T.GetEventTimer(id)
        assert(T.GetEventTrack(id) == 2)
    "#).unwrap();
    env.fire_on_update(11.0).unwrap();
    env.exec("assert(T.GetEventTrack(id)==1 and highlights==1); assert(T.GetEventHighlightTime()==5)").unwrap();
    env.fire_on_update(5.0).unwrap();
    env.exec(r#"
        local track,index=T.GetEventTrack(id)
        assert(track==0 and index==1)
        assert(T.GetEventState(id)==Enum.EncounterTimelineEventState.Active)
        assert(T.GetEventTimeRemaining(id)==0 and timer:GetRemainingDuration()==0)
        assert(timer:GetElapsedDuration()==16 and T.HasVisibleEvents())
    "#).unwrap();
    env.fire_on_update(2.5).unwrap();
    env.exec("assert(T.GetEventState(id)==0 and timer:GetRemainingDuration()==0 and highlights==1)").unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("assert(T.GetEventState(id)==2 and not T.HasVisibleEvents()); assert(T.GetEventInfo(id))").unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec("assert(T.GetEventInfo(id)==nil and timer:GetElapsedDuration()==16)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_tracks_filter_limits_ties_capacity_pause_and_view() {
    let env=environment();
    env.exec(r#"
        a=add(5); b=add(5); c=add(20); d=add(65); e=add(66); f=add(67); g=add(68)
        paused=add(1,{paused=true})
        assert(T.GetEventTrack(g)==4 and T.GetEventTrack(paused)==4)
        local _,i=T.GetEventTrack(d); assert(i==1)
        local list=T.GetSortedEventList(2,5)
        assert(#list==2 and list[1]==a and list[2]==b)
        assert(#T.GetSortedEventList()==6)
        assert(#T.GetSortedEventList(nil,nil,true,false)==8)
        assert(#T.GetSortedEventList(0)==0)
        T.FinishScriptEvent(a)
        assert(T.GetSortedEventList()[1]==b)
        assert(#T.GetSortedEventList(nil,nil,false,false)==8)
        T.PauseScriptEvent(d)
        assert(T.GetEventTrack(d)==4 and T.GetEventTrack(g)==3)
        T.ResumeScriptEvent(d)
        assert(T.GetEventTrack(g)==4)
        views={}
        local frame=CreateFrame('Frame')
        for _,name in ipairs({'ENCOUNTER_TIMELINE_VIEW_DEACTIVATED','ENCOUNTER_TIMELINE_LAYOUT_UPDATED','ENCOUNTER_TIMELINE_VIEW_ACTIVATED'}) do frame:RegisterEvent(name) end
        frame:SetScript('OnEvent',function(_,event,view)
            views[#views+1]=event
            if event=='ENCOUNTER_TIMELINE_VIEW_ACTIVATED' then assert(T.GetViewType()==view); assert(#T.GetTrackList()==5) end
        end)
        T.SetViewType(Enum.EncounterTimelineViewType.Bars)
        assert(#views==3 and views[1]=='ENCOUNTER_TIMELINE_VIEW_DEACTIVATED' and views[3]=='ENCOUNTER_TIMELINE_VIEW_ACTIVATED')
        T.SetViewType(Enum.EncounterTimelineViewType.Bars); assert(#views==3)
        assert(not pcall(T.SetViewType, 99))
        assert(not pcall(T.GetSortedEventList,-1))
        assert(not pcall(T.GetTrackInfo,9))
        T.SetViewType(Enum.EncounterTimelineViewType.None)
        assert(not T.HasVisibleEvents() and #T.GetSortedEventList()==0)
        T.SetViewType(Enum.EncounterTimelineViewType.Timeline)
        assert(T.HasVisibleEvents())
    "#).unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(T.GetSortedEventList(1,4)[1]==b); assert(#T.GetSortedEventList(nil,4)==1)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_tracks_preview_refresh_and_script_ownership() {
    let env=environment();
    env.exec(r#"
        script=add(100)
        assert(T.AddEditModeEvents()==30)
        assert(T.GetEventCountBySource(Enum.EncounterTimelineEventSource.EditMode)==3)
        local before=#T.GetEventList()
        assert(T.AddEditModeEvents()==30 and #T.GetEventList()==before)
        assert(T.GetEventCountBySource(Enum.EncounterTimelineEventSource.Script)==1)
        T.CancelAllScriptEvents()
        assert(T.GetEventState(script)==3)
        for _,id in ipairs(T.GetEventList()) do
            local info=T.GetEventInfo(id)
            if info.source==2 then assert(T.GetEventState(id)==0 and info.spellName:find('Simulator preview')) end
        end
        T.CancelEditModeEvents()
        for _,id in ipairs(T.GetEventList()) do assert(T.GetEventState(id)==3) end
    "#).unwrap();
    env.fire_on_update(0.0).unwrap();
    env.exec("assert(#T.GetEventList()==0); assert(T.AddEditModeEvents()==30)").unwrap();
    env.fire_on_update(30.0).unwrap();
    env.exec("assert(T.AddEditModeEvents()==30); assert(T.GetEventCountBySource(2)==3)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_tracks_reentrant_changes_and_queued_pause() {
    let env=environment();
    env.exec(r#"
        id=add(1,{maxQueueDuration=4}); retained=T.GetEventTimer(id)
        local frame=CreateFrame('Frame'); frame:RegisterEvent('ENCOUNTER_TIMELINE_EVENT_TRACK_CHANGED')
        frame:SetScript('OnEvent',function(_,_,eventID)
            if eventID==id and T.GetEventTrack(id)==0 and not pausedOnce then pausedOnce=true; T.PauseScriptEvent(id) end
        end)
    "#).unwrap();
    env.fire_on_update(2.0).unwrap();
    env.exec("assert(T.GetEventState(id)==1 and T.GetEventTrack(id)==4); assert(retained:GetRemainingDuration()==0)").unwrap();
    env.fire_on_update(10.0).unwrap();
    env.exec("T.ResumeScriptEvent(id); assert(T.GetEventTrack(id)==0)").unwrap();
    env.fire_on_update(2.9).unwrap();
    env.exec("assert(T.GetEventState(id)==0)").unwrap();
    env.fire_on_update(0.1).unwrap();
    env.exec("assert(T.GetEventState(id)==2)").unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn encounter_tracks_queued_capacity_and_reentrant_view_updates() {
    let env=environment();
    env.exec(r#"
        ids={add(0,{maxQueueDuration=10}),add(0,{maxQueueDuration=10}),add(0,{maxQueueDuration=10}),add(0,{maxQueueDuration=10})}
        assert(T.GetEventTrack(ids[4])==4)
        assert(#T.GetSortedEventList(nil,0)==3)
        T.FinishScriptEvent(ids[1])
        assert(T.GetEventTrack(ids[4])==0)
        local _,index=T.GetEventTrack(ids[4]); assert(index==3)
        local listener=CreateFrame('Frame')
        listener:RegisterEvent('ENCOUNTER_TIMELINE_VIEW_DEACTIVATED')
        listener:RegisterEvent('ENCOUNTER_TIMELINE_VIEW_ACTIVATED')
        activated={}
        listener:SetScript('OnEvent',function(_,event,view)
            if event=='ENCOUNTER_TIMELINE_VIEW_DEACTIVATED' and not redirected then
                redirected=true; T.SetViewType(Enum.EncounterTimelineViewType.None)
            elseif event=='ENCOUNTER_TIMELINE_VIEW_ACTIVATED' then
                activated[#activated+1]=view; assert(T.GetViewType()==view)
            end
        end)
        T.SetViewType(Enum.EncounterTimelineViewType.Bars)
        assert(T.GetViewType()==0 and #activated==0)
        assert(not T.HasVisibleEvents())
    "#).unwrap();
}
