#![cfg(all(
    feature = "optional-tests",
    any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever",
        feature = "client-mists"
    )
))]

#[test]
fn selected_actionbar_event_consumer_uses_original_fanout() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &["Blizzard_ActionBar"],
        &[],
        |env, loaded| {
            assert!(
                loaded.iter().any(|name| name == "Blizzard_ActionBar"),
                "ActionBar must load before its event consumer"
            );
            env.exec(
                r#"
                assert(type(ActionBarButtonEventsFrameMixin) == "table")
                assert(ActionBarButtonEventsFrame:IsEventRegistered("ACTIONBAR_SLOT_CHANGED"))
                local actionBarLoadSource = debug.getinfo(ActionBarMixin.ActionBar_OnLoad, "S").source:gsub("\\", "/")
                assert(actionBarLoadSource:find("Blizzard_ActionBar/Shared/ActionBar.lua", 1, true), actionBarLoadSource)
                assert(type(StanceBar.actionButtons) == "table" and #StanceBar.actionButtons == StanceBar.numButtons,
                    "selected StanceBar owner must create its buttons during XML OnLoad")

                probe = { events = 0, artUpdates = 0, cooldownUpdates = 0, framesVisited = 0 }
                local button = {
                    OnEvent = function(_, event, slot)
                        assert(event == "ACTIONBAR_SLOT_CHANGED")
                        assert(slot == 17)
                        probe.events = probe.events + 1
                    end,
                    UpdateButtonArt = function()
                        probe.artUpdates = probe.artUpdates + 1
                    end,
                }
                ActionBarButtonEventsFrame.frames = { button }
                ActionButton_UpdateCooldownNumberHidden = function(frame)
                    assert(frame == button)
                    probe.cooldownUpdates = probe.cooldownUpdates + 1
                end
                "#,
            )
            .expect("ActionBar event frame should be ready");

            env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(17.0)])
                .expect("registered ActionBar event should dispatch");
            env.exec(
                r#"
                ActionBarButtonEventsFrame:OnCountdownForCooldownsChanged()
                ActionBarButtonEventsFrame:ForEachFrame(function(frame)
                    assert(frame == ActionBarButtonEventsFrame.frames[1])
                    probe.framesVisited = probe.framesVisited + 1
                end)
                "#,
            )
            .expect("selected ActionBar frame methods should visit their registered button");

            let (
                event_count,
                art_updates,
                cooldown_updates,
                frames_visited,
                on_event_source,
                countdown_source,
                iterator_source,
                instance_on_event_source,
                instance_countdown_source,
                instance_iterator_source,
            ): (
                i64,
                i64,
                i64,
                i64,
                String,
                String,
                String,
                String,
                String,
                String,
            ) = env
                .eval(
                    r#"
                    local function source(fn)
                        return debug.getinfo(fn, "S").source:gsub("\\", "/")
                    end
                    return probe.events,
                        probe.artUpdates,
                        probe.cooldownUpdates,
                        probe.framesVisited,
                        source(ActionBarButtonEventsFrameMixin.OnEvent),
                        source(ActionBarButtonEventsFrameMixin.OnCountdownForCooldownsChanged),
                        source(ActionBarButtonEventsFrameMixin.ForEachFrame),
                        source(ActionBarButtonEventsFrame.OnEvent),
                        source(ActionBarButtonEventsFrame.OnCountdownForCooldownsChanged),
                        source(ActionBarButtonEventsFrame.ForEachFrame)
                    "#,
                )
                .expect("ActionBar event dispatch should be observable");

            assert_eq!(
                event_count, 1,
                "original ActionBar fanout should deliver the registered event once"
            );
            assert_eq!(
                art_updates, 0,
                "the original OnEvent does not add a second UpdateButtonArt pass"
            );
            assert_eq!(cooldown_updates, 1);
            assert_eq!(frames_visited, 1);
            for source in [
                on_event_source,
                countdown_source,
                iterator_source,
                instance_on_event_source,
                instance_countdown_source,
                instance_iterator_source,
            ] {
                assert!(
                    source.contains("Blizzard_ActionBar/Shared/ActionButton.lua"),
                    "ActionBarButtonEventsFrame methods must keep their original owner, got {source}"
                );
            }
        },
    );
}
