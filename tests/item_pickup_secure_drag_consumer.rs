//! Original SecureHandlerDragTemplate drag snippets pick up items by ID.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn secure_drag_snippet_places_item_id_cursor_and_events(env: &WowLuaEnv) {
    env.exec(r#"
        local source = debug.getinfo(SecureHandler_OnDragEvent, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_RestrictedAddOnEnvironment/SecureHandlers.lua', 1, true), source)
        local events = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({'CURSOR_CHANGED', 'ACTIONBAR_SHOWGRID', 'ACTIONBAR_HIDEGRID'}) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event, ...)
            local parts = {event}
            for index = 1, select('#', ...) do
                parts[#parts + 1] = tostring((select(index, ...)))
            end
            events[#events + 1] = table.concat(parts, ',')
        end)
        local frame = CreateFrame('Frame', nil, UIParent, 'SecureHandlerDragTemplate')
        local function drag(item)
            frame:SetAttribute('_ondragstart', 'return "item", ' .. item)
            events = {}
            frame:GetScript('OnDragStart')(frame, 'LeftButton')
            return table.concat(events, ';')
        end

        ClearCursor()
        assert(drag(6948) == 'ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,6,0,0', table.concat(events, ';'))
        local kind, id, link = GetCursorInfo()
        assert(kind == 'item' and id == 6948 and link == select(2, C_Item.GetItemInfo(6948)), tostring(link))
        assert(not CursorHasItem())

        assert(drag(211993) == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,6,6948;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,6,0,0', table.concat(events, ';'))
        assert(select(2, GetCursorInfo()) == 211993)

        assert(drag(999999) == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,6,211993;ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,6,0,0', table.concat(events, ';'))
        assert(select('#', GetCursorInfo()) == 0 and not CursorHasItem())

        assert(drag(0) == 'ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,6,999999', table.concat(events, ';'))
        assert(select('#', GetCursorInfo()) == 0)
        assert(drag(0) == '', table.concat(events, ';'))
    "#).unwrap();
}
}
