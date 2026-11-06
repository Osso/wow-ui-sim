use super::*;

fn render_gradient_frame(frame: &crate::widget::Frame) -> QuadBatch {
    let mut batch = QuadBatch::new();
    let mut fonts = WowFontSystem::new_without_casc();
    let mut atlas = GlyphAtlas::new();
    emit_widget_text_quads(
        &mut WidgetTextRenderer {
            batch: &mut batch,
            font_sys: &mut fonts,
            glyph_atlas: &mut atlas,
        },
        frame,
        WidgetTextLayout {
            text: frame.text.as_deref().unwrap(),
            bounds: Rectangle::new(Point::ORIGIN, Size::new(200.0, 600.0)),
            justify_h: crate::widget::TextJustify::Left,
            justify_v: crate::widget::TextJustify::Left,
            word_wrap: true,
            max_lines: 0,
            alpha: 0.5,
        },
    );
    batch
}

fn glyph_alphas(batch: &QuadBatch) -> Vec<f32> {
    batch
        .vertices
        .chunks_exact(4)
        .filter(|q| q[0].tex_index == GLYPH_ATLAS_TEX_INDEX)
        .map(|q| q[0].color[3])
        .collect()
}

#[test]
fn fontstring_gradient_wraps_multiplies_alpha_replaces_and_clears() {
    let env = crate::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        T4Description = CreateFrame('Frame'):CreateFontString('T4Description')
        T4Description:SetSize(200, 600)
        T4Description:SetText(string.rep('H', 80))
        T4Description:SetTextColor(1, 0.25, 0.5, 0.8)
        T4Description:SetAlphaGradient(0, 30)
    "#,
    )
    .unwrap();
    let render = || {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("T4Description").unwrap();
        render_gradient_frame(state.widgets.get(id).unwrap())
    };
    let first = render();
    let alphas = glyph_alphas(&first);
    assert_eq!(alphas.len(), 80, "all wrapped glyphs must exist");
    for (index, alpha) in alphas.iter().enumerate() {
        let expected = 0.4 * ((30.0 - index as f32) / 30.0).clamp(0.0, 1.0);
        assert!(
            (alpha - expected).abs() < 0.0001,
            "glyph {index}: {alpha} != {expected}"
        );
    }
    env.exec("T4Description:SetAlphaGradient(80, 30)").unwrap();
    let complete = render();
    assert!(
        glyph_alphas(&complete)
            .iter()
            .all(|a| (*a - 0.4).abs() < 0.0001)
    );
    env.exec("T4Description:SetAlphaGradient(0, 0)").unwrap();
    assert!(glyph_alphas(&render()).iter().all(|a| *a == 0.0));
    env.exec("T4Description:ClearAlphaGradient()").unwrap();
    assert_eq!(
        glyph_alphas(&render()),
        glyph_alphas(&complete),
        "clear restores ordinary text alpha"
    );
    assert_eq!(
        first
            .vertices
            .iter()
            .map(|v| v.position)
            .collect::<Vec<_>>(),
        complete
            .vertices
            .iter()
            .map(|v| v.position)
            .collect::<Vec<_>>(),
        "gradient cannot move text"
    );
}

#[test]
fn fontstring_gradient_counts_unicode_space_and_hard_line_break() {
    let env = crate::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        T4Unicode = CreateFrame('Frame'):CreateFontString('T4Unicode')
        T4Unicode:SetSize(200, 600)
        T4Unicode:SetText('Hé H\nH')
        T4Unicode:SetAlphaGradient(0, 6)
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("T4Unicode").unwrap();
    let batch = render_gradient_frame(state.widgets.get(id).unwrap());
    let alphas = glyph_alphas(&batch);
    let positions = [0.0, 1.0, 3.0, 5.0];
    assert_eq!(alphas.len(), positions.len());
    for (alpha, position) in alphas.iter().zip(positions) {
        assert!((alpha - 0.5 * (6.0 - position) / 6.0).abs() < 0.0001);
    }
}

#[test]
fn fontstring_gradient_continues_across_color_segments_and_paint_passes() {
    let env = crate::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        T4Styled = CreateFrame('Frame'):CreateFontString('T4Styled')
        T4Styled:SetSize(200, 600)
        T4Styled:SetText('HHHH')
        T4Styled:SetShadowColor(0, 0, 0, 0.4)
        T4Styled:SetShadowOffset(0, 0)
        T4Styled:SetFont('Fonts/FRIZQT__.TTF', 12, 'OUTLINE')
        T4Styled:SetAlphaGradient(0, 4)
    "#,
    )
    .unwrap();
    {
        // Feed the renderer's existing colored-segment representation directly.
        let mut state = env.state().borrow_mut();
        let id = state.widgets.get_id_by_name("T4Styled").unwrap();
        state.widgets.get_mut(id).unwrap().text_segments = vec![
            crate::widget::TextSegment {
                text: "HH".to_string(),
                color: crate::widget::Color::new(1.0, 0.0, 0.0, 1.0),
            },
            crate::widget::TextSegment {
                text: "HH".to_string(),
                color: crate::widget::Color::new(1.0, 1.0, 1.0, 1.0),
            },
        ];
    }
    let render = || {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("T4Styled").unwrap();
        render_gradient_frame(state.widgets.get(id).unwrap())
    };
    let gradient = render();
    env.exec("T4Styled:ClearAlphaGradient()").unwrap();
    let ordinary = render();
    assert!(!gradient.vertices.is_empty());
    assert_eq!(gradient.vertices.len(), ordinary.vertices.len());
    // Each segment has two glyphs, repeated for outline/shadow/foreground passes.
    let quads_per_segment = ordinary.vertices.len() / 4 / 2;
    assert!(quads_per_segment > 2, "shadow/outline must actually emit");
    for (index, (faded, plain)) in gradient
        .vertices
        .chunks_exact(4)
        .zip(ordinary.vertices.chunks_exact(4))
        .enumerate()
    {
        let position = (index / quads_per_segment * 2 + index % 2) as f32;
        for (faded, plain) in faded.iter().zip(plain) {
            assert_eq!(faded.position, plain.position);
            assert_eq!(&faded.color[..3], &plain.color[..3]);
            assert!((faded.color[3] - plain.color[3] * (4.0 - position) / 4.0).abs() < 0.0001);
        }
    }
}

#[cfg(any(feature = "client-wrath", feature = "client-mists"))]
mod consumer {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    /// Publish the profile's pinned `SOUNDKIT` table before the consumer files
    /// run, because the real client loads it first.
    ///
    /// `Blizzard_UIPanels_Game_Classic.toc:41` loads `Classic\QuestFrame.lua`,
    /// whose `:802` calls `PlaySound(SOUNDKIT.IG_WRITE_QUEST)`, and that addon
    /// depends on the always-loaded `Blizzard_SharedXML`, whose TOC selects the
    /// sound-kit file per profile (`Blizzard_SharedXML_Mists.toc:5`
    /// `## AllowLoadGameType: mists`, `:45` `Wrath\SoundKitConstants.lua`).
    /// The hand-picked file list below would otherwise omit that global.
    #[cfg(feature = "client-mists")]
    fn load_pinned_sound_kit_constants(root: &std::path::Path, env: &WowLuaEnv) {
        let addon_dir = root.join("mists/AddOns/Blizzard_SharedXML");
        let toc_path = crate::loader::find_toc_file(&addon_dir)
            .expect("pinned Blizzard_SharedXML TOC required");
        let toc = crate::toc::TocFile::from_file(&toc_path)
            .unwrap_or_else(|error| panic!("{}: {error}", toc_path.display()));
        let entries: Vec<&std::path::PathBuf> = toc
            .files
            .iter()
            .filter(|file| {
                file.file_name().and_then(|name| name.to_str()) == Some("SoundKitConstants.lua")
            })
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "{}: expected one SoundKitConstants entry, got {entries:?}",
            toc_path.display()
        );
        let path = addon_dir.join(entries[0]);
        let source = std::fs::read_to_string(&path).expect("pinned sound-kit table required");
        env.exec(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    }

    fn load_cached_consumer(env: &WowLuaEnv) {
        let root = dirs::home_dir()
            .unwrap()
            .join(".cache/wow-ui-sim/blizzard-ui");
        // Original whole public sources; no copied/reconstructed callback or vendor patch.
        // Wrath: FrameXML/QuestInfo.lua:3-16,479-492; QuestFrame.lua:4-6.
        // Mists: AddOns/Blizzard_UIPanels_Game/Classic/QuestFrame.lua:4-6,799-814.
        // Mists helper: AddOns/Blizzard_UIPanels_Game/Wrath/QuestInfo.lua:919-945.
        // Explicit Classic fixture does NOT establish Mists automatic branch selection.
        #[cfg(feature = "client-mists")]
        load_pinned_sound_kit_constants(&root, env);
        #[cfg(feature = "client-wrath")]
        let files = [
            "wrath/FrameXML/QuestFrame.lua",
            "wrath/FrameXML/QuestInfo.lua",
        ];
        #[cfg(feature = "client-mists")]
        let files = [
            "mists/AddOns/Blizzard_UIPanels_Game/Classic/QuestFrame.lua",
            "mists/AddOns/Blizzard_UIPanels_Game/Wrath/QuestInfo.lua",
        ];
        for file in files {
            let path = root.join(file);
            let source = std::fs::read_to_string(&path).expect("original profile cache required");
            env.exec(&source)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }

    fn prepare_consumer() -> WowLuaEnv {
        let env = WowLuaEnv::new().unwrap();
        load_cached_consumer(&env);
        env.exec(
            r#"
            QuestInfoFrame = CreateFrame('Frame')
            QuestFrameAcceptButton = CreateFrame('Button')
            QuestInfoFrame.acceptButton = QuestFrameAcceptButton
            QuestFrameAcceptButton:Disable()
            T4Fade = CreateFrame('Frame')
            T4Fade:SetAlpha(0)
            T4Dependent = CreateFrame('Frame')
            T4Dependent:SetAlpha(0)
            QuestDetailScrollChildFrame = CreateFrame('Frame')
            QuestDetailScrollChildFrame.alphaDependentText = {T4Dependent}
            QuestInfoDescriptionText = QuestInfoFrame:CreateFontString('QuestInfoDescriptionText')
            QuestInfoDescriptionText:SetSize(200, 600)
            QuestInfoDescriptionText:SetText(string.rep('H', 80))
            QuestInfoDescriptionText:SetAlphaGradient(0, QUEST_DESCRIPTION_GRADIENT_LENGTH)
            T4Fade.fading = 1
            T4Fade.fadingProgress = 0
            -- Select immediate dependent-control reveal, not the separate fade scheduler.
            QUEST_FADING_DISABLE = '1'
            SetCVar('instantQuestText', '1')
        "#,
        )
        .unwrap();
        #[cfg(feature = "client-wrath")]
        env.exec("T4Fade:SetScript('OnUpdate', QuestInfoFadingFrame_OnUpdate)")
            .unwrap();
        #[cfg(feature = "client-mists")]
        env.exec("T4Fade:SetScript('OnUpdate', QuestFrameDetailPanel_OnUpdate)")
            .unwrap();
        env
    }

    fn update(env: &WowLuaEnv, elapsed: f32) {
        env.exec(&format!("T4Fade:GetScript('OnUpdate')(T4Fade, {elapsed})"))
            .unwrap();
    }

    #[test]
    fn cached_quest_consumer_progress_changes_rendered_glyph_alpha() {
        let env = prepare_consumer();
        let render = || {
            let state = env.state().borrow();
            let id = state
                .widgets
                .get_id_by_name("QuestInfoDescriptionText")
                .unwrap();
            render_gradient_frame(state.widgets.get(id).unwrap())
        };
        let before = glyph_alphas(&render());
        assert_eq!(before.len(), 80);
        update(&env, 0.1);
        let after = glyph_alphas(&render());
        assert_eq!(before.len(), after.len());
        assert!(
            after.iter().sum::<f32>() > before.iter().sum::<f32>(),
            "consumer progress must reveal glyphs"
        );
        assert!(
            after.iter().any(|a| *a == 0.0),
            "unreached wrapped tail stays hidden"
        );
        env.exec("assert(T4Fade.fading and not QuestFrameAcceptButton:IsEnabled())")
            .unwrap();
    }

    #[test]
    fn cached_quest_consumer_finishes_and_enables_accept_including_instant_progress() {
        for instant in [false, true] {
            let env = prepare_consumer();
            if instant {
                env.exec("T4Fade.fadingProgress = 1024").unwrap();
            }
            update(&env, if instant { 0.0 } else { 2.0 });
            env.exec("assert(T4Fade.fading == nil, 'completion branch not reached'); assert(QuestFrameAcceptButton:IsEnabled())").unwrap();
            #[cfg(feature = "client-wrath")]
            env.exec("assert(T4Fade:GetAlpha() == 1)").unwrap();
            #[cfg(feature = "client-mists")]
            env.exec("assert(T4Dependent:GetAlpha() == 1)").unwrap();
            assert!(
                glyph_alphas(&{
                    let state = env.state().borrow();
                    let id = state
                        .widgets
                        .get_id_by_name("QuestInfoDescriptionText")
                        .unwrap();
                    render_gradient_frame(state.widgets.get(id).unwrap())
                })
                .iter()
                .all(|a| *a == 0.5)
            );
        }
    }
}
