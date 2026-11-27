#![cfg(feature = "client-wowforever")]
//! Forever publishes the Retail TextWithState, BulletTextList, Spacer,
//! SpellDisplay and ButtonHeader visualization getters (the pinned
//! UIWidgetManagerDocumentation.lua functions and structures are
//! byte-identical to Retail's). Original consumer: `UIWidgetTopCenterContainerFrame`
//! (Blizzard_UIWidgetTopCenterFrame.lua) registers for
//! `C_UIWidgetManager.GetTopCenterWidgetSetID()`; each `UPDATE_UI_WIDGET` runs
//! the type's `visInfoDataFunction` getter and the template `Setup`.

use wow_ui_sim::c_api::ui_widget_manager::{
    BulletTextListRecord, ButtonHeaderRecord, SpacerRecord, SpellDisplayRecord,
    TextWithStateRecord, WidgetSetEntry, WidgetSetRecord, WidgetSpellButtonRecord,
    WidgetSpellInfoRecord, WidgetVisualization,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const TOP_CENTER_SET: f64 = 1.0;
const SHOWN: u32 = 1;
const YELLOW: u32 = 1;

/// One input round: a label used by every text field and a size.
struct Round {
    label: &'static str,
    size: f64,
    lines: usize,
}

fn text_with_state(round: &Round) -> TextWithStateRecord {
    TextWithStateRecord {
        shown_state: SHOWN,
        enabled_state: YELLOW,
        text: format!("{} text", round.label),
        tooltip: String::new(),
        text_size_type: 0,
        font_type: 0,
        bottom_padding: 0.0,
        tooltip_loc: 0,
        h_align: 0,
        widget_size_setting: 0.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 1.0,
        widget_tag: String::new(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 0,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn bullet_text_list(round: &Round) -> BulletTextListRecord {
    let lines = (1..=round.lines)
        .map(|index| format!("{} line {index}", round.label))
        .collect();
    BulletTextListRecord {
        shown_state: SHOWN,
        enabled_state: YELLOW,
        lines,
        widget_size_setting: 0.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 2.0,
        widget_tag: String::new(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 0,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn spacer(round: &Round) -> SpacerRecord {
    SpacerRecord {
        shown_state: SHOWN,
        widget_width: round.size,
        widget_height: round.size,
        widget_size_setting: 0.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 3.0,
        widget_tag: String::new(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 0,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn spell_info(round: &Round) -> WidgetSpellInfoRecord {
    WidgetSpellInfoRecord {
        spell_id: 0.0,
        shown_state: SHOWN,
        enabled_state: YELLOW,
        tooltip: String::new(),
        text: format!("{} spell", round.label),
        stack_display: round.lines as f64,
        icon_size_type: 0,
        icon_display_type: 0,
        text_shown_state: 0, // SpellDisplayTextShownStateType.Shown
        border_color: 0,
        text_font_type: 0,
        text_size_type: 0,
        h_align_type: 0,
        tint: 0,
        show_glow_state: 0,
        show_as_earned: false,
    }
}

fn spell_display(round: &Round) -> SpellDisplayRecord {
    SpellDisplayRecord {
        spell_info: spell_info(round),
        tooltip_loc: 0,
        widget_size_setting: 0.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 4.0,
        widget_tag: String::new(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 0,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

fn header_button(index: usize) -> WidgetSpellButtonRecord {
    WidgetSpellButtonRecord {
        spell_id: 0.0,
        tooltip: format!("button {index}"),
        icon: 134400,
        enabled_state: YELLOW,
        cooldown_type: 0,
    }
}

fn button_header(round: &Round) -> ButtonHeaderRecord {
    ButtonHeaderRecord {
        shown_state: SHOWN,
        header_text: format!("{} header", round.label),
        tooltip: String::new(),
        tooltip_loc: 0,
        buttons: (1..=round.lines).map(header_button).collect(),
        widget_size_setting: 0.0,
        texture_kit: String::new(),
        frame_texture_kit: String::new(),
        has_timer: false,
        order_index: 5.0,
        widget_tag: String::new(),
        in_anim_type: 0,
        out_anim_type: 0,
        widget_scale: 0,
        layout_direction: 0,
        model_scene_layer: 0,
        scripted_animation_effect_id: 0.0,
    }
}

/// (widgetID, Enum.UIWidgetVisualizationType, record).
fn visualizations(round: &Round) -> [(f64, u32, WidgetVisualization); 5] {
    use WidgetVisualization as V;
    [
        (770_001.0, 8, V::TextWithState(text_with_state(round))),
        (770_002.0, 10, V::BulletTextList(bullet_text_list(round))),
        (770_003.0, 22, V::Spacer(spacer(round))),
        (770_004.0, 13, V::SpellDisplay(spell_display(round))),
        (770_005.0, 30, V::ButtonHeader(button_header(round))),
    ]
}

fn feed(env: &WowLuaEnv, round: &Round) {
    for (widget_id, widget_type, record) in visualizations(round) {
        let entry = WidgetSetEntry {
            widget_id,
            widget_type,
            unit_token: None,
        };
        env.update_ui_widget(TOP_CENTER_SET, entry, record)
            .expect("host widget update");
    }
}

const OBSERVE: &str = r#"
local frames = UIWidgetTopCenterContainerFrame.widgetFrames
local text, bullets, space, spell, header =
    frames[770001], frames[770002], frames[770003], frames[770004], frames[770005]
return text.Text:GetText(), bullets.linePool:GetNumActive(), space:GetHeight(),
    spell.Spell.Text:GetText(), header.HeaderText:GetText(), header.buttonPool:GetNumActive()
"#;

type Observed = (String, f64, f64, String, String, f64);

fn expected(round: &Round) -> Observed {
    let label = round.label;
    let lines = round.lines as f64;
    (
        format!("{label} text"),
        lines,
        round.size,
        format!("{label} spell"),
        format!("{label} header"),
        lines,
    )
}

/// The host announces the empty top-center set before the UI loads, so the
/// container's `OnLoad` finds it through `GetWidgetSetInfo`.
fn forever_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    let empty = WidgetSetRecord {
        layout_direction: 0,
        vertical_padding: 0.0,
        widgets: vec![],
    };
    env.replace_ui_widget_set(TOP_CENTER_SET, Some(empty))
        .expect("empty top-center set");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    let game = wow_ui_sim::screen::ScreenKind::Game;
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
    env
}

#[test]
fn forever_top_center_widgets_render_the_supplied_records() {
    crate::common::with_timeout(120, || {
        let env = forever_game_env();
        for round in [
            Round { label: "First", size: 24.0, lines: 2 },
            Round { label: "Second", size: 40.0, lines: 3 },
        ] {
            feed(&env, &round);
            let observed: Observed = env.eval(OBSERVE).expect("read the widget frames");
            assert_eq!(observed, expected(&round));
        }
    });
}
