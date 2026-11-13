//! Chat frame initialization — simulator features, not workarounds.
//!
//! Sets up ChatFrame1 position/size, chat type colors, and fake chat
//! messages for a realistic UI appearance.
//!
use super::WowLuaEnv;

// Wrath's native FloatingChatFrame owns placement and editbox geometry.
const PRESERVE_NATIVE_CHAT_LAYOUT: bool = cfg!(feature = "client-wrath");

/// Show ChatFrame1 and set DEFAULT_CHAT_FRAME after addon loading.
pub fn show_chat_frame(env: &WowLuaEnv) {
    activate_chat_frame(env);
    start_fake_chat(env);
}

/// Prepare the real Blizzard chat frame before third-party addons run.
pub fn prepare_for_third_party_addons(env: &WowLuaEnv) {
    init_chat_type_colors(env);
    activate_chat_frame(env);
}

fn activate_chat_frame(env: &WowLuaEnv) {
    if PRESERVE_NATIVE_CHAT_LAYOUT {
        return;
    }
    let _ = env.exec(SHOW_CHAT_FRAME_LUA);
}

/// Preserve each caller's existing parent-frame requirement outside native Wrath.
pub(crate) fn normalize_chat_editbox_width(env: &WowLuaEnv, require_chat_frame: bool) {
    if PRESERVE_NATIVE_CHAT_LAYOUT {
        return;
    }
    let frame_check = if require_chat_frame {
        "and ChatFrame1"
    } else {
        ""
    };
    let _ = env.exec(&format!(
        "if ChatFrame1EditBox {frame_check} then ChatFrame1EditBox:SetWidth(447) end"
    ));
}

/// Initialize r,g,b on ChatTypeInfo entries.
pub fn init_chat_type_colors(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not ChatTypeInfo then return end
        local defaults = {
            SYSTEM={1,1,0}, SAY={1,1,1}, PARTY={.67,.67,1}, RAID={1,.5,0},
            GUILD={.25,1,.25}, OFFICER={.25,.75,.25}, YELL={1,.25,.25},
            WHISPER={1,.5,1}, WHISPER_INFORM={1,.5,1}, EMOTE={1,.5,.25},
            TEXT_EMOTE={1,.5,.25}, CHANNEL={1,.75,.5}, LOOT={0,.67,0},
            MONEY={1,1,0}, SKILL={.33,.33,1}, ACHIEVEMENT={1,1,0},
            GUILD_ACHIEVEMENT={.25,1,.25}, BN_WHISPER={0,.8,1},
            BN_WHISPER_INFORM={0,.8,1}, INSTANCE_CHAT={1,.5,0},
            INSTANCE_CHAT_LEADER={1,.5,0},
        }
        local function applyDefaults(chatTypeInfo)
            for key, info in pairs(chatTypeInfo) do
                local d = defaults[key]
                if d or rawget(info, "r") == nil then
                    d = d or {1, 1, 1}
                    info.r, info.g, info.b = d[1], d[2], d[3]
                end
            end
        end

        applyDefaults(ChatTypeInfo)

        local mt = getmetatable(ChatTypeInfo)
        if type(mt) == "table" and type(mt.__index) == "table" then
            applyDefaults(mt.__index)
        end
    "#,
    );
}

fn start_fake_chat(env: &WowLuaEnv) {
    register_fake_chat_data(env);
    fix_chat_scrollbar_anchors(env);
    schedule_fake_chat_tickers(env);
}

fn register_fake_chat_data(env: &WowLuaEnv) {
    register_fake_chat_messages(env);
    register_fake_chat_names(env);
}

fn register_fake_chat_messages(env: &WowLuaEnv) {
    let _ = env.exec(FAKE_CHAT_MESSAGES_LUA);
}

fn register_fake_chat_names(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not _FakeChat then return end
        _FakeChat.names.general = {"Thunderfury", "Moonwhisper", "Stabbymcstab", "Healbot", "Tanklord"}
        _FakeChat.names.trade = {"Goldmaker", "Craftypants", "Auctioneer", "Bankalt"}
        _FakeChat.names.say = {"Legolas", "Arthasdklol", "Pwnstar", "Noobslayer"}
        _FakeChat.names.guild = {"Valorheart", "Shieldmaiden", "Firestorm", "Arcanewing"}
        _FakeChat.idx = {general = 1, trade = 1, say = 1, guild = 1}
        function _FakeChat:pick(channel)
            local list = self.msgs[channel]
            local i = self.idx[channel]
            self.idx[channel] = (i % #list) + 1
            return list[i], self.names[channel][math.random(#self.names[channel])]
        end
    "#,
    );
}

fn schedule_fake_chat_tickers(env: &WowLuaEnv) {
    let _ = env.exec(SCHEDULE_FAKE_CHAT_TICKERS_LUA);
}

fn fix_chat_scrollbar_anchors(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if ChatFrame1 and ChatFrame1.ScrollBar and ChatFrame1.ScrollToBottomButton then
            ChatFrame1.ScrollBar:ClearAllPoints()
            ChatFrame1.ScrollBar:SetPoint("TOPLEFT", ChatFrame1, "TOPRIGHT", 0, 0)
            ChatFrame1.ScrollBar:SetPoint("BOTTOMLEFT", ChatFrame1.ScrollToBottomButton, "TOPLEFT", 0, 2)
        end
    "#,
    );
}

const SHOW_CHAT_FRAME_LUA: &str = r#"
    local function __wow_prepare_default_chat_frame()
        if ChatFrame1 then
            DEFAULT_CHAT_FRAME = ChatFrame1
            if ChatFrame1EditBox then
                ChatFrame1.editBox = ChatFrame1EditBox
            end
        elseif DEFAULT_CHAT_FRAME and ChatFrame1EditBox and DEFAULT_CHAT_FRAME.editBox == nil then
            DEFAULT_CHAT_FRAME.editBox = ChatFrame1EditBox
        end
    end

    if ChatFrame1 then
        ChatFrame1:Show()
        __wow_prepare_default_chat_frame()
        if type(__wow_send_chat_message) == "function" then
            SendChatMessage = __wow_send_chat_message
            if type(C_ChatInfo) == "table" then
                C_ChatInfo.SendChatMessage = __wow_send_chat_message
            end
        end
        ChatFrame1:ClearAllPoints()
        ChatFrame1:SetPoint("BOTTOMLEFT", UIParent, "BOTTOMLEFT", 35, 50)
        ChatFrame1:SetSize(430, 170)
        if FloatingChatFrameMixin and FloatingChatFrameMixin.OnLoad and not ChatFrame1.__codexOnLoadRan then
            ChatFrame1.__codexOnLoadRan = true
            FloatingChatFrameMixin.OnLoad(ChatFrame1)
        end
        ChatFrame1.oldAlpha = ChatFrame1.oldAlpha or DEFAULT_CHATFRAME_ALPHA or 0.3
        if ChatFrame1.ResizeButton then
            ChatFrame1.ResizeButton:ClearAllPoints()
            ChatFrame1.ResizeButton:SetPoint("BOTTOMRIGHT", ChatFrame1, "BOTTOMRIGHT", 0, 0)
        end
        if ChatFrame1.ScrollToBottomButton and ChatFrame1.ResizeButton then
            ChatFrame1.ScrollToBottomButton:ClearAllPoints()
            ChatFrame1.ScrollToBottomButton:SetPoint("BOTTOMRIGHT", ChatFrame1.ResizeButton, "TOPRIGHT", -2, -2)
        end
        if ChatFrame1.ScrollBar and ChatFrame1.ScrollToBottomButton then
            ChatFrame1.ScrollBar:ClearAllPoints()
            ChatFrame1.ScrollBar:SetPoint("TOPLEFT", ChatFrame1, "TOPRIGHT", 0, 0)
            ChatFrame1.ScrollBar:SetPoint("BOTTOMLEFT", ChatFrame1.ScrollToBottomButton, "TOPLEFT", 0, 2)
        end
        if FCF_UpdateResizeButton then
            FCF_UpdateResizeButton(ChatFrame1)
        end
        if ChatFrame1.ScrollBar then
            ChatFrame1.ScrollBar:SetWidth(23)
        end
        local function __wow_guard_reinitialize(target)
            if target and not target.__wowDefaultChatFrameGuarded then
                local originalReinitialize = target.Reinitialize
                if type(originalReinitialize) == "function" then
                    target.Reinitialize = function(self, ...)
                        __wow_prepare_default_chat_frame()
                        return originalReinitialize(self, ...)
                    end
                    target.__wowDefaultChatFrameGuarded = true
                end
            end
        end
        __wow_guard_reinitialize(ChatFrameMenuButtonMixin)
        __wow_guard_reinitialize(ChatFrameMenuButton)
    end
"#;

const FAKE_CHAT_MESSAGES_LUA: &str = r#"
    if not ChatFrame1 then return end
    _FakeChat = { msgs = {}, names = {}, idx = {} }
    _FakeChat.msgs.general = {
        "Anyone know where the portal trainer is?",
        "LFM Deadmines, need healer",
        "WTS [Copper Bar] x20, 5g each",
        "How do I get to Ironforge from here?",
        "Is the Darkmoon Faire up this week?",
        "Just hit level 60!",
        "What's the fastest way to level cooking?",
        "Any good guilds recruiting?",
    }
    _FakeChat.msgs.trade = {
        "WTS [Enchant Weapon - Crusader] your mats + 10g tip",
        "WTB [Large Brilliant Shard] x5, paying 3g each",
        "LF Blacksmith to craft [Arcanite Reaper], have mats",
        "WTS [Flask of the Titans] 45g, cheap!",
        "WTB [Righteous Orb] x2, PST with price",
        "Selling port to Dalaran, 1g",
    }
    _FakeChat.msgs.say = {
        "Anyone else lagging?", "Thanks for the group!",
        "Where did that quest NPC go?",
        "I think I took a wrong turn somewhere",
        "Wow, this place is huge", "Can someone help with this elite?",
    }
    _FakeChat.msgs.guild = {
        "Hey everyone!", "Anyone up for a dungeon run?",
        "Grats on the new gear!", "Guild bank has some free enchanting mats",
        "Raid signup is up on the calendar",
        "I just finished the attunement quest chain",
    }
"#;

const SCHEDULE_FAKE_CHAT_TICKERS_LUA: &str = r#"
    if not _FakeChat then return end
    local fc = _FakeChat
    local function timestamp()
        local fmt = GetCVar and GetCVar("showTimestamps")
        if fmt and fmt ~= "" and fmt ~= "none" then
            return date(fmt, time())
        end
        return ""
    end
    local function post(channel, prefix, r, g, b)
        local msg, name = fc:pick(channel)
        ChatFrame1:_AddMessageSilent(timestamp() .. prefix ..
            "|Hplayer:" .. name .. "|h[" .. name .. "]|h: " .. msg,
            r, g, b)
    end
    C_Timer.After(0, function() C_Timer.NewTicker(40, function()
        post("general", "|Hchannel:General|h[1. General]|h ", 1.0, 0.75, 0.5)
    end) end)
    C_Timer.After(5, function() C_Timer.NewTicker(40, function()
        post("trade", "|Hchannel:Trade|h[2. Trade]|h ", 1.0, 0.75, 0.5)
    end) end)
    C_Timer.After(10, function() C_Timer.NewTicker(40, function()
        local msg, name = fc:pick("say")
        ChatFrame1:_AddMessageSilent(
            timestamp() .. "|Hplayer:" .. name .. "|h[" .. name .. "]|h says: " .. msg,
            1.0, 1.0, 1.0)
    end) end)
    C_Timer.After(15, function() C_Timer.NewTicker(40, function()
        post("guild", "|Hchannel:Guild|h[Guild]|h ", 0.25, 1.0, 0.25)
    end) end)
"#;
