BootstrapOrderProbeRecord("C:eager")
SLASH_BOOTSTRAPORDERPROBE1 = "/boprobe"
SlashCmdList.BOOTSTRAPORDERPROBE = function(message)
    if message == "load" then
        BootstrapOrderProbeRecord("load:before")
        local success, ok, reason = pcall(C_AddOns.LoadAddOn, "BootstrapOrderProbe_B")
        local event = BootstrapOrderProbeRecord("load:after")
        event.loadResult = { success = success }
        if success then
            event.loadResult.ok = ok
            event.loadResult.reason = reason and tostring(reason) or nil
        else
            event.loadResult.error = tostring(ok)
        end
        BootstrapOrderProbePublish()
    else
        BootstrapOrderProbeRecord("snapshot")
    end
    print("[BootstrapOrderProbe] current session events: " .. #BootstrapOrderProbeSession.events)
    print("[BootstrapOrderProbe] /boprobe load twice, then /reload to save")
end
