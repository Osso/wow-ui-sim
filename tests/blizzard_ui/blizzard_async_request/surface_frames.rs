//! Frame surface probes for `Blizzard_AsyncRequest`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AsyncRequest";

#[cfg(feature = "optional-tests")]
#[test]
fn create_async_request_returns_initialized_anonymous_frame() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let surface: AsyncRequestFrameSurface = env
            .eval(
                r#"
                local request = AsyncRequests:CreateAsyncRequest({
                    requestFunction = function() end,
                    responseEventName = "ASYNC_REQUEST_TEST_RESPONSE",
                    responseEventCallback = function() end,
                })

                return request:GetObjectType(),
                       request:GetParent() == nil,
                       type(request.Init),
                       type(request.StartRequest),
                       type(request.StopRequest),
                       request.isRunning,
                       request.timeoutTimer == nil
                "#,
            )
            .expect("CreateAsyncRequest frame surface probe must run cleanly");

        assert_async_request_frame_surface(surface);
    });
}

type AsyncRequestFrameSurface = (String, bool, String, String, String, bool, bool);

fn assert_async_request_frame_surface(surface: AsyncRequestFrameSurface) {
    let (
        object_type,
        has_nil_parent,
        init_type,
        start_request_type,
        stop_request_type,
        is_running,
        has_nil_timeout_timer,
    ) = surface;

    assert_eq!(object_type, "Frame", "async requests must be Frame objects");
    assert!(
        has_nil_parent,
        "async request frames must be anonymous frames with nil parent"
    );
    assert_eq!(init_type, "function", "`Init` must be mixed into the frame");
    assert_eq!(
        start_request_type, "function",
        "`StartRequest` must be mixed into the frame"
    );
    assert_eq!(
        stop_request_type, "function",
        "`StopRequest` must be mixed into the frame"
    );
    assert!(
        !is_running,
        "`Init` must seed new async requests as not running"
    );
    assert!(
        has_nil_timeout_timer,
        "`Init` must seed new async requests without a timeout timer"
    );
}
