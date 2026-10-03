//! Texture insertion and cached tooltip-line secrecy, source-inspected only.
//! Historical Retail 12.0.5 lacks secret-origin ingestion; current Retail has it.
#![cfg(all(feature = "retail-12-0-5", feature = "forbidden-aspects"))]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_string;
use wow_ui_sim::lua_api::WowLuaEnv;

fn create_tooltip_texture_secret_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create real tooltip environment");
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let secret = wrap_host_secret_string(lua.state_mut(), "Private tooltip content");
        lua.state_mut().push(secret);
        lua.set_global_val("TTTextureSecret", secret)
            .expect("root authentic VM secret string");
        lua.state_mut().pop();
    }
    env.exec(
        r#"
        assert(issecretvalue(TTTextureSecret), 'authentic secret positive control')
        function TTTextureAddon(probe)
            assert(issecure(), 'secure outer fixture')
            local function addon()
                assert(debug.getstacktaint() == 'TooltipTextureProbe')
                probe()
                assert(debug.getstacktaint() == 'TooltipTextureProbe')
            end
            debug.setobjecttaint(addon, 'TooltipTextureProbe')
            addon()
            assert(issecure(), 'outer context restored')
        end
        function TTTexturePublic(object, expected)
            local ok, text = pcall(object.GetText, object)
            assert(ok, 'public text readable by addon')
            assert(text == expected, 'actual content, not placeholder')
            assert(not issecretvalue(text))
        end
        function TTTexturePrivate(object)
            local ok, err = pcall(object.GetText, object)
            assert(not ok, 'real secret-origin read gate must reject addon')
            assert(type(err) == 'string')
            assert(string.find(err, 'secret-origin widget readout', 1, true))
        end
        "#,
    )
    .expect("install assertion helpers without overriding tooltip methods");
    env
}

#[test]
fn tooltip_texture_secrecy_public_texture_and_atlas_preserve_readability() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local tooltip = CreateFrame('GameTooltip')
        tooltip:SetText('Public tooltip content')
        TTTextureAddon(function() TTTexturePublic(tooltip, 'Public tooltip content') end)
        tooltip:AddTexture(134400)
        assert(tooltip:NumLines() == 2, 'texture insertion actually occurred')
        TTTextureAddon(function() TTTexturePublic(tooltip, 'Public tooltip content') end)
        tooltip:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        assert(tooltip:NumLines() == 3, 'atlas insertion actually occurred')
        TTTextureAddon(function() TTTexturePublic(tooltip, 'Public tooltip content') end)
        "#,
    )
    .expect("ordinary textures cannot make public tooltip text unreadable");
}

#[test]
fn tooltip_texture_secrecy_private_texture_and_atlas_preserve_denial() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local tooltip = CreateFrame('GameTooltip')
        tooltip:SetText(TTTextureSecret)
        assert(tooltip:GetText() == 'Private tooltip content', 'secure decoded content')
        TTTextureAddon(function() TTTexturePrivate(tooltip) end)
        tooltip:AddTexture(134400)
        assert(tooltip:NumLines() == 2)
        TTTextureAddon(function() TTTexturePrivate(tooltip) end)
        tooltip:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        assert(tooltip:NumLines() == 3)
        TTTextureAddon(function() TTTexturePrivate(tooltip) end)
        assert(tooltip:GetText() == 'Private tooltip content', 'no content loss')
        "#,
    )
    .expect("texture and atlas insertion do not declassify existing secret text");
}

#[test]
fn tooltip_texture_secrecy_cleared_textures_allow_public_repopulation() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local tooltip = CreateFrame('GameTooltip')
        tooltip:SetText(TTTextureSecret)
        tooltip:AddTexture(134400)
        tooltip:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        TTTextureAddon(function() TTTexturePrivate(tooltip) end)
        tooltip:ClearLines()
        assert(tooltip:NumLines() == 0)
        assert(tooltip:GetLeftLine(1) == nil)
        tooltip:SetText('Public replacement')
        assert(tooltip:NumLines() == 1)
        TTTextureAddon(function() TTTexturePublic(tooltip, 'Public replacement') end)
        tooltip:AddTexture(134400)
        assert(tooltip:NumLines() == 2)
        TTTextureAddon(function() TTTexturePublic(tooltip, 'Public replacement') end)
        "#,
    )
    .expect("cleared texture history cannot poison replacement content");
}

#[test]
fn tooltip_texture_secrecy_isolated_between_public_and_private_tooltips() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local public = CreateFrame('GameTooltip')
        local private = CreateFrame('GameTooltip')
        public:SetText('Independent public tooltip')
        private:SetText(TTTextureSecret)
        private:AddTexture(134400)
        public:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        TTTextureAddon(function()
            TTTexturePublic(public, 'Independent public tooltip')
            TTTexturePrivate(private)
        end)
        public:ClearLines()
        public:SetText('Reused public tooltip')
        public:AddTexture(134400)
        TTTextureAddon(function()
            TTTexturePublic(public, 'Reused public tooltip')
            TTTexturePrivate(private)
        end)
        private:SetText('Declassified by replacement')
        TTTextureAddon(function()
            TTTexturePublic(private, 'Declassified by replacement')
            TTTexturePublic(public, 'Reused public tooltip')
        end)
        "#,
    )
    .expect("tooltip mutation and secrecy remain per-instance");
}

#[test]
fn tooltip_texture_secrecy_line_replacement_preserves_other_private_line() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local tooltip = CreateFrame('GameTooltip')
        tooltip:AddLine('First line')
        tooltip:AddLine('Second line')
        local first = tooltip:GetLeftLine(1)
        local second = tooltip:GetLeftLine(2)
        first:SetText(TTTextureSecret)
        second:SetText(TTTextureSecret)
        TTTextureAddon(function() TTTexturePrivate(first); TTTexturePrivate(second) end)
        tooltip:AddTexture(134400)
        tooltip:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        assert(tooltip:NumLines() == 4)
        tooltip:Hide()
        tooltip:Show()
        TTTextureAddon(function() TTTexturePrivate(first); TTTexturePrivate(second) end)
        first:SetText('Public first line')
        TTTextureAddon(function()
            TTTexturePublic(first, 'Public first line')
            TTTexturePrivate(second)
        end)
        "#,
    )
    .expect("line replacement declassifies only replaced text; visibility does not");
}

#[test]
fn tooltip_texture_secrecy_cleared_cached_line_reuse_is_public() {
    let env = create_tooltip_texture_secret_env();
    env.exec(
        r#"
        local tooltip = CreateFrame('GameTooltip', 'TTTextureReusedTooltip')
        tooltip:AddLine('Initial line')
        local cached = tooltip:GetLeftLine(1)
        cached:SetText(TTTextureSecret)
        assert(cached:GetText() == 'Private tooltip content')
        TTTextureAddon(function() TTTexturePrivate(cached) end)
        tooltip:AddTexture(134400)
        tooltip:AddAtlas('UI-HUD-UnitFrame-Player-PortraitOn-Bar-Health')
        assert(tooltip:NumLines() == 3)
        tooltip:ClearLines()
        assert(tooltip:NumLines() == 0 and tooltip:GetLeftLine(1) == nil)
        tooltip:AddLine('Public reused line')
        local reused = tooltip:GetLeftLine(1)
        assert(reused == cached, 'same observable pooled FontString, not fresh allocation')
        assert(reused:GetText() == 'Public reused line', 'secure confirms actual replacement')
        TTTextureAddon(function() TTTexturePublic(reused, 'Public reused line') end)
        "#,
    )
    .expect("cleared pooled line must not retain historical secret-origin denial");
}
