use super::CommandDispatch;
use super::Commands;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use wow_ui_sim::font::WowFontSystem;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{
    apply_delay_with_tick, apply_ui_scale, run_extra_update_ticks, settle_headless_startup,
};

pub(super) fn run_gui(dispatch: CommandDispatch) -> Result<(), Box<dyn std::error::Error>> {
    let debug_options = dispatch.debug_options();
    wow_ui_sim::run_iced_ui(
        dispatch.env,
        debug_options,
        dispatch.saved_vars,
        dispatch.exec_lua,
        dispatch.exec_lua_secure,
    )
}

pub(super) fn dispatch_headless_click_probe(
    dispatch: CommandDispatch,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(Commands::HeadlessClickProbe {
        panel,
        width,
        height,
    }) = dispatch.command
    else {
        unreachable!("dispatch_headless_click_probe only fires for Commands::HeadlessClickProbe");
    };
    let plan = headless_click_probe_plan(&panel)?;
    wow_ui_sim::iced_app::run_headless_named_click_probe(
        dispatch.env,
        dispatch.saved_vars,
        iced::Size::new(width as f32, height as f32),
        plan.setup_lua,
        plan.clicks,
        plan.verify_lua,
    )
    .map_err(Box::<dyn std::error::Error>::from)
}

struct HeadlessClickProbePlan<'a> {
    setup_lua: &'a str,
    clicks: &'a [wow_ui_sim::iced_app::NamedClick<'a>],
    verify_lua: Option<&'a str>,
}

fn headless_click_probe_plan(
    panel: &str,
) -> Result<HeadlessClickProbePlan<'static>, Box<dyn std::error::Error>> {
    match panel {
        "achievements" => Ok(achievements_click_probe_plan()),
        "talents" => Ok(talents_click_probe_plan()),
        "mounts" => Ok(mounts_click_probe_plan()),
        "micromenu" => Ok(micromenu_click_probe_plan()),
        _ => Err(format!("unknown headless click probe panel: {panel}").into()),
    }
}

pub(super) fn dispatch_screenshot(dispatch: CommandDispatch) {
    let Some(Commands::Screenshot {
        output,
        width,
        height,
        filter,
        crop,
        dump_tree,
        quality,
        ui_scale,
    }) = dispatch.command
    else {
        unreachable!("dispatch_screenshot only fires for Commands::Screenshot");
    };
    run_screenshot(
        &dispatch.env,
        &dispatch.font_system,
        ScreenshotCommand {
            output,
            width,
            height,
            filter,
            crop,
            delay: dispatch.delay,
            exec_lua: dispatch.exec_lua.as_deref(),
            exec_lua_secure: dispatch.exec_lua_secure,
            dump_tree,
            quality,
            ui_scale,
        },
    );
}

pub(super) fn dispatch_dump_texture(dispatch: CommandDispatch) {
    let Some(Commands::DumpTexture {
        output,
        filter,
        frame_filter,
    }) = dispatch.command
    else {
        unreachable!("dispatch_dump_texture only fires for Commands::DumpTexture");
    };
    run_dump_texture(
        &dispatch.env,
        &dispatch.font_system,
        output,
        filter,
        frame_filter,
    );
}

fn achievements_click_probe_plan() -> HeadlessClickProbePlan<'static> {
    HeadlessClickProbePlan {
        setup_lua: r#"
            ToggleAchievementFrame()
            if not AchievementFrame or not AchievementFrame:IsShown() then
                error("AchievementFrame did not open")
            end
        "#,
        clicks: &[
            wow_ui_sim::iced_app::NamedClick {
                frame_name: "AchievementFrameTab2",
            },
            wow_ui_sim::iced_app::NamedClick {
                frame_name: "AchievementFrameTab1",
            },
        ],
        verify_lua: None,
    }
}

fn mounts_click_probe_plan() -> HeadlessClickProbePlan<'static> {
    HeadlessClickProbePlan {
        setup_lua: r#"
            C_AddOns.LoadAddOn("Blizzard_Collections")
            ToggleCollectionsJournal(1)
            if not MountJournal or not MountJournal:IsShown() then
                error("MountJournal did not open")
            end
            local row = MountJournal.ScrollBox:GetFrames()[2]
            if not row or not row.mountID then
                error("mount list row 2 missing or has no mountID")
            end
            __mount_probe_target_id = row.mountID
            if MountJournal.selectedMountID == __mount_probe_target_id then
                error("mount list row 2 is already selected; probe needs a selection change")
            end
            __mount_probe_log = {}
            for _, script in ipairs({ "OnMouseDown", "OnMouseUp", "OnClick" }) do
                row:HookScript(script, function(_, button)
                    table.insert(__mount_probe_log, script .. ":" .. tostring(button))
                end)
            end
        "#,
        clicks: &[wow_ui_sim::iced_app::NamedClick {
            frame_name: "MountJournal.ScrollBox.ScrollTarget.#2",
        }],
        verify_lua: Some(
            r#"
            if MountJournal.selectedMountID ~= __mount_probe_target_id then
                error(string.format(
                    "mount selection did not switch: selected=%s want=%s row_scripts=[%s]",
                    tostring(MountJournal.selectedMountID),
                    tostring(__mount_probe_target_id),
                    table.concat(__mount_probe_log, ",")
                ))
            end
        "#,
        ),
    }
}

fn micromenu_click_probe_plan() -> HeadlessClickProbePlan<'static> {
    HeadlessClickProbePlan {
        setup_lua: r#"
            if not LFDMicroButton or not LFDMicroButton:IsShown() then
                error("LFDMicroButton missing or hidden")
            end
            print(string.format(
                "[micromenu-probe] LFD rect l=%s r=%s level=%d  Collections l=%s  Guild l=%s",
                tostring(LFDMicroButton:GetLeft()),
                tostring(LFDMicroButton:GetRight()),
                LFDMicroButton:GetFrameLevel(),
                tostring(CollectionsMicroButton and CollectionsMicroButton:GetLeft()),
                tostring(GuildMicroButton and GuildMicroButton:GetLeft())
            ))
            if PVEFrame and PVEFrame:IsShown() then
                error("PVEFrame already open; probe needs it closed")
            end
            local function rectOf(frame)
                if not frame then return "nil" end
                return string.format("(%s,%s %sx%s)",
                    tostring(frame:GetLeft()), tostring(frame:GetTop()),
                    tostring(frame:GetWidth()), tostring(frame:GetHeight()))
            end
            __micro_rects = function()
                return string.format(
                    "LFD=%s MicroMenu=%s Container=%s",
                    rectOf(LFDMicroButton), rectOf(MicroMenu), rectOf(MicroMenuContainer))
            end
            __micro_before = __micro_rects()
        "#,
        clicks: &[wow_ui_sim::iced_app::NamedClick {
            frame_name: "LFDMicroButton",
        }],
        verify_lua: Some(
            r#"
            if not PVEFrame or not PVEFrame:IsShown() then
                error(string.format(
                    "LFDMicroButton click did not open PVEFrame; before=[%s] after=[%s]",
                    tostring(__micro_before), __micro_rects()
                ))
            end
        "#,
        ),
    }
}

fn talents_click_probe_plan() -> HeadlessClickProbePlan<'static> {
    HeadlessClickProbePlan {
        setup_lua: r#"
            ToggleTalentFrame()
            if not PlayerTalentFrame or not PlayerTalentFrame:IsShown() then
                error("PlayerTalentFrame did not open")
            end
        "#,
        clicks: &[
            wow_ui_sim::iced_app::NamedClick {
                frame_name: "PlayerTalentFrameTab2",
            },
            wow_ui_sim::iced_app::NamedClick {
                frame_name: "PlayerTalentFrameTab3",
            },
            wow_ui_sim::iced_app::NamedClick {
                frame_name: "PlayerTalentFrameTab1",
            },
        ],
        verify_lua: None,
    }
}

pub(super) struct ScreenshotCommand<'a> {
    pub(super) output: PathBuf,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) filter: Option<String>,
    pub(super) crop: Option<String>,
    pub(super) delay: Option<u64>,
    pub(super) exec_lua: Option<&'a str>,
    pub(super) exec_lua_secure: bool,
    pub(super) dump_tree: Option<Option<String>>,
    pub(super) quality: f32,
    pub(super) ui_scale: f32,
}

pub(super) fn run_screenshot(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    command: ScreenshotCommand<'_>,
) {
    prepare_screenshot_env(env, &command);
    let (batch, glyph_atlas) = build_screenshot_batch(
        env,
        font_system,
        command.width,
        command.height,
        command.filter.as_deref(),
    );
    if let Some(dump_filter) = &command.dump_tree {
        dump_screenshot_tree(env, dump_filter.as_deref(), command.width, command.height);
    }
    log_screenshot_batch(&batch);

    let img = render_screenshot_image(&batch, &glyph_atlas, command.width, command.height);
    let img = apply_optional_crop(img, command.crop.as_deref());
    let output = save_screenshot(&img, &command.output, command.quality);
    eprintln!(
        "Saved {}x{} screenshot to {}",
        img.width(),
        img.height(),
        output.display()
    );
}

fn prepare_screenshot_env(env: &WowLuaEnv, command: &ScreenshotCommand<'_>) {
    settle_headless_startup(env);
    apply_ui_scale(env, command.ui_scale);
    env.set_screen_size(command.width as f32, command.height as f32);
    wow_ui_sim::debug_helpers::debug_show_game_menu(env);
    apply_delay_with_tick(env, command.delay);
    run_screenshot_exec_lua(env, command);
    run_extra_update_ticks(env, 3);
}

fn run_screenshot_exec_lua(env: &WowLuaEnv, command: &ScreenshotCommand<'_>) {
    let Some(code) = command.exec_lua else {
        return;
    };
    if let Err(e) = env.exec_maybe_secure(code, command.exec_lua_secure) {
        eprintln!("[exec-lua] error: {e}");
    }
}

fn log_screenshot_batch(batch: &wow_ui_sim::render::QuadBatch) {
    eprintln!(
        "QuadBatch: {} quads, {} texture requests",
        batch.quad_count(),
        batch.texture_requests.len()
    );
}

fn render_screenshot_image(
    batch: &wow_ui_sim::render::QuadBatch,
    glyph_atlas: &wow_ui_sim::render::GlyphAtlas,
    width: u32,
    height: u32,
) -> image::RgbaImage {
    let mut tex_mgr = create_texture_manager();
    let glyph_data = glyph_atlas.is_dirty().then(|| {
        let (data, size, _) = glyph_atlas.texture_data();
        (data, size)
    });
    wow_ui_sim::render::headless::render_to_image(batch, &mut tex_mgr, width, height, glyph_data)
}

fn apply_optional_crop(img: image::RgbaImage, crop: Option<&str>) -> image::RgbaImage {
    match crop {
        Some(crop_str) => apply_crop(img, crop_str),
        None => img,
    }
}

pub(super) fn run_dump_texture(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    output: PathBuf,
    filter: Option<String>,
    frame_filter: Option<String>,
) {
    env.set_screen_size(1600.0, 1200.0);
    settle_headless_startup(env);
    let (batch, _) = build_screenshot_batch(env, font_system, 1600, 1200, frame_filter.as_deref());
    eprintln!(
        "QuadBatch: {} quads, {} tex requests",
        batch.quad_count(),
        batch.texture_requests.len()
    );
    let mut tex_mgr = create_texture_manager();
    wow_ui_sim::dump_texture::dump_batch_textures(&batch, &mut tex_mgr, &output, filter.as_deref());
}

fn build_screenshot_batch(
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
    width: u32,
    height: u32,
    filter: Option<&str>,
) -> (
    wow_ui_sim::render::QuadBatch,
    wow_ui_sim::render::GlyphAtlas,
) {
    use wow_ui_sim::iced_app::{
        RegistryQuadBatchParams, build_quad_batch_for_registry_with_quest_blobs,
    };
    use wow_ui_sim::render::GlyphAtlas;

    let mut glyph_atlas = GlyphAtlas::new();
    let batch = {
        let mut fs = font_system.borrow_mut();
        let buckets = {
            let mut state = env.state().borrow_mut();
            wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut fs);
            state.ensure_layout_rects();
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
        build_quad_batch_for_registry_with_quest_blobs(
            RegistryQuadBatchParams::new(&state.widgets, (width as f32, height as f32), &buckets)
                .root_name(filter)
                .text_ctx(Some((&mut fs, &mut glyph_atlas)))
                .message_frames(Some(&state.message_frames))
                .tooltip_data(Some(&tooltip_data))
                .quest_blobs(Some(&state.quest_blobs)),
        )
    };
    (batch, glyph_atlas)
}

fn dump_screenshot_tree(env: &WowLuaEnv, filter_key: Option<&str>, width: u32, height: u32) {
    let state = env.state().borrow();
    let addon_names: Vec<String> = state.addons.iter().map(|a| a.folder_name.clone()).collect();
    wow_ui_sim::dump::print_frame_tree(
        &state.widgets,
        &addon_names,
        None,
        filter_key,
        false,
        false,
        width as f32,
        height as f32,
    );
}

fn parse_crop(spec: &str) -> Option<(u32, u32, u32, u32)> {
    let (dims, rest) = spec.split_once('+')?;
    let (x_str, y_str) = rest.split_once('+')?;
    let (w_str, h_str) = dims.split_once('x')?;
    Some((
        w_str.parse().ok()?,
        h_str.parse().ok()?,
        x_str.parse().ok()?,
        y_str.parse().ok()?,
    ))
}

fn apply_crop(img: image::RgbaImage, crop_str: &str) -> image::RgbaImage {
    use image::GenericImageView;

    let (crop_width, crop_height, crop_x, crop_y) = parse_crop(crop_str).unwrap_or_else(|| {
        eprintln!("Invalid crop format '{}', expected WxH+X+Y", crop_str);
        std::process::exit(1);
    });
    if crop_x + crop_width > img.width() || crop_y + crop_height > img.height() {
        eprintln!("Crop region exceeds image bounds");
        std::process::exit(1);
    }
    img.view(crop_x, crop_y, crop_width, crop_height).to_image()
}

/// Write the rendered frame to `output`.
///
/// A `.png` extension is honoured and written losslessly. Anything else is
/// forced to `.webp` and encoded lossily at `quality`. UI screenshots are
/// mostly small text and one-pixel borders, which lossy WebP smears first, so
/// the default is high and PNG is available when the pixels have to be exact.
fn save_screenshot(img: &image::RgbaImage, output: &Path, quality: f32) -> PathBuf {
    let wants_png = output
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));

    if wants_png {
        if let Err(e) = img.save(output) {
            eprintln!("Failed to save PNG: {}", e);
            std::process::exit(1);
        }
        return output.to_path_buf();
    }

    let output = output.with_extension("webp");
    let encoder = webp::Encoder::from_rgba(img.as_raw(), img.width(), img.height());
    let mem = encoder.encode(quality.clamp(1.0, 100.0));
    if let Err(e) = std::fs::write(&output, &*mem) {
        eprintln!("Failed to save WebP: {}", e);
        std::process::exit(1);
    }
    output
}

fn create_texture_manager() -> wow_ui_sim::texture::TextureManager {
    use wow_ui_sim::texture::TextureManager;

    let config = wow_ui_sim::config::SimConfig::load();
    let mut mgr =
        TextureManager::new().with_addons_paths(wow_ui_sim::paths::default_addons_paths());
    mgr.preload_talent_textures(790);
    mgr.preload_talent_panel_textures(&config.player_class);
    mgr
}
