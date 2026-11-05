//! Global surface probes for `Blizzard_AsyncRequest`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AsyncRequest";

#[cfg(feature = "optional-tests")]
#[test]
fn async_request_exports_only_public_namespace_entry_point() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let surface: AsyncRequestSurface = env
            .eval(
                r#"
                return type(AsyncRequests),
                       type(AsyncRequests.CreateAsyncRequest),
                       type(AsyncRequestMixin)
                "#,
            )
            .expect("AsyncRequest global surface probe must run cleanly");

        assert_async_request_surface(surface);
    });
}

type AsyncRequestSurface = (String, String, String);

fn assert_async_request_surface(surface: AsyncRequestSurface) {
    let (namespace_type, create_type, mixin_type) = surface;

    assert_eq!(
        namespace_type, "table",
        "`AsyncRequests` must be the public namespace table"
    );
    assert_eq!(
        create_type, "function",
        "`AsyncRequests.CreateAsyncRequest` must be the public factory function"
    );
    assert_eq!(
        mixin_type, "nil",
        "`AsyncRequestMixin` must remain file-local; CreateAsyncRequest is the public entry point"
    );
}
