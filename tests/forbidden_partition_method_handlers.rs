//! `<OnLoad method="X"/>` handlers built on the fast path called
//! `self[method](self, ...)` on the public object only. A template whose
//! secure mixin installs its methods on the frame's forbidden partition
//! (`TargetFrameAuraContainerTemplate`: `<Mixin source="secure"/>` plus
//! `<ForbiddenAspects>`) therefore raised "attempt to call field '?'" from
//! its OnLoad, which aborted the creation of every frame inheriting
//! `TargetFrameTemplate`: TargetFrame and FocusFrame came out 0x0, MEDIUM,
//! without their mixins. The same handler ran as OnUpdate every frame.
//! Separately, the shared bootstrap re-created its partition registry on
//! every re-execution, so partitions created before EnvironmentCleanup were
//! orphaned in the public environment.
//!
//! Blizzard_UnitFrame's TOC does not declare Blizzard_AuraContainer (the
//! secure addon its aura containers run on); startup loads it earlier by
//! manifest order, so the closure names it.

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const ROOTS: &[&str] = &["Blizzard_AuraContainer", "Blizzard_UnitFrame"];

#[test]
fn target_frame_template_chain_survives_the_aura_container_on_load() {
    with_blizzard_addon_closure(ROOTS, &[], |env, _| {
        let (created, create_err, width, strata, mixin, focus): (bool, String, f64, String, String, String) = env
            .eval(
                r#"
                local created, create_err = pcall(CreateFrame, "Button", nil, UIParent, "TargetFrameAuraContainerTemplate")
                return created, tostring(create_err), TargetFrame:GetWidth(), TargetFrame:GetFrameStrata(),
                    type(TargetFrame.OnLoad_TargetFrameInstance), type(FocusFrame)
                "#,
            )
            .expect("probe");
        assert!(created, "a frame from TargetFrameAuraContainerTemplate can be created: {create_err}");
        assert_eq!(width, 232.0, "TargetFrameTemplate's <Size> reached TargetFrame");
        assert_eq!(strata, "LOW", "TargetFrameTemplate's frameStrata reached TargetFrame");
        assert_eq!(mixin, "function", "TargetFrameInstanceMixin was applied");
        assert_eq!(focus, "table", "TargetFrame.xml ran on to FocusFrame");
    });
}

#[test]
fn method_handlers_dispatch_to_the_forbidden_partition_after_the_restore() {
    with_blizzard_addon_closure(ROOTS, &[], |env, _| {
        // The restore re-executes the shared bootstrap; the partition
        // registered at load must still be the one the handler resolves.
        env.restore_post_cleanup_globals();
        let (has_on_update, hit, self_is_partition): (bool, bool, bool) = env
            .eval(
                r#"
                local auras = TargetFrame.TargetFrameContent.TargetFrameContentContextual.Auras
                local partition = GetForbiddenObjectTable(auras)
                local has = type(rawget(partition, "OnUpdate")) == "function"
                local hit, self_is_partition = false, false
                rawset(partition, "OnUpdate", function(self) hit = true; self_is_partition = (self == partition) end)
                auras:GetScript("OnUpdate")(auras, 0.016)
                return has, hit, self_is_partition
                "#,
            )
            .expect("probe");
        assert!(has_on_update, "the private mixin's OnUpdate sits on the partition registered at load");
        assert!(hit, "the <OnUpdate method> handler reaches the partition's method");
        assert!(self_is_partition, "the partition is what the private method receives as self");
    });
}
