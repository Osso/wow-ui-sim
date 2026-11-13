#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

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

                local probe = { events = 0, artUpdates = 0 }
                ActionBarButtonEventsFrame.frames = {
                    {
                        OnEvent = function(_, event, slot)
                            assert(event == "ACTIONBAR_SLOT_CHANGED")
                            assert(slot == 17)
                            probe.events = probe.events + 1
                        end,
                        UpdateButtonArt = function()
                            probe.artUpdates = probe.artUpdates + 1
                        end,
                    },
                }
                "#,
            )
            .expect("ActionBar event frame should be ready");

            env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(17.0)])
                .expect("registered ActionBar event should dispatch");

            let (event_count, art_updates, on_event_source, countdown_source, iterator_source): (
                i64,
                i64,
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
                        source(ActionBarButtonEventsFrameMixin.OnEvent),
                        source(ActionBarButtonEventsFrameMixin.OnCountdownForCooldownsChanged),
                        source(ActionBarButtonEventsFrameMixin.ForEachFrame)
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
            for source in [on_event_source, countdown_source, iterator_source] {
                assert!(
                    source.contains("Blizzard_ActionBar/Shared/ActionButton.lua"),
                    "ActionBarButtonEventsFrame methods must keep their original owner, got {source}"
                );
            }
        },
    );
}
