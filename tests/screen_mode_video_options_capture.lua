local createDropdown = Settings.CreateDropdown
Settings.CreateDropdown = function(category, setting, getOptions, ...)
    if setting:GetVariable() == "PROXY_RESOLUTION" then
        __video_resolution_options = getOptions
    elseif setting:GetVariable() == "PROXY_GX_ADAPTER" then
        __video_adapter_options = getOptions
    end
    return createDropdown(category, setting, getOptions, ...)
end
