//! Global surface for `Blizzard_ArdenwealdGardening`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArdenwealdGardening";

#[cfg(feature = "optional-tests")]
#[test]
fn ardenweald_gardening_exports_public_globals() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let surface: GlobalSurfaceProbe = env
            .eval(
                r#"
                return type(ArdenwealdGardening),
                       type(ArdenwealdGardening.Create),
                       type(ArdenwealdGardeningButtonMixin),
                       type(ArdenwealdGardeningButtonMixin.OnEnter),
                       type(ArdenwealdGardeningButtonMixin.OnLeave),
                       type(ArdenwealdGardeningSecondsFormatter)
                "#,
            )
            .expect("Ardenweald Gardening global surface probe must run cleanly");

        assert_global_surface(surface);
    });
}

type GlobalSurfaceProbe = (String, String, String, String, String, String);

fn assert_global_surface(surface: GlobalSurfaceProbe) {
    let (namespace_type, create_type, mixin_type, on_enter_type, on_leave_type, formatter_type) =
        surface;

    assert_gardening_namespace_surface(namespace_type, create_type);
    assert_gardening_button_mixin_surface(mixin_type, on_enter_type, on_leave_type);
    assert_eq!(
        formatter_type, "nil",
        "`ArdenwealdGardeningSecondsFormatter` is intentionally file-local in Blizzard Lua"
    );
}

fn assert_gardening_namespace_surface(namespace_type: String, create_type: String) {
    assert_eq!(
        namespace_type, "table",
        "`ArdenwealdGardening` must be a namespace table"
    );
    assert_eq!(
        create_type, "function",
        "`ArdenwealdGardening.Create` must be a factory function"
    );
}

fn assert_gardening_button_mixin_surface(
    mixin_type: String,
    on_enter_type: String,
    on_leave_type: String,
) {
    assert_eq!(
        mixin_type, "table",
        "`ArdenwealdGardeningButtonMixin` must be a mixin table"
    );
    assert_eq!(
        on_enter_type, "function",
        "`ArdenwealdGardeningButtonMixin.OnEnter` must be exported"
    );
    assert_eq!(
        on_leave_type, "function",
        "`ArdenwealdGardeningButtonMixin.OnLeave` must be exported"
    );
}
