# Animation factory

`SimpleAnimGroup:CreateAnimation(animationType?, name?, templateName?)` returns one non-nil animation object. Both pinned revisions have the same optional arguments and return contract; PTR adds only the `AddAnimations` forbidden-aspect check on the receiving group. See the [pinned register](../../data/patch-api/sources/12.1.5-register.json).

## What it must do

- [x] Omitted/explicit-nil arguments create distinct generic animation objects; an explicit type/name produces the requested object type/name. Each result belongs to its group and owning region, with default order one and duration zero.
- [x] The named result retains configured order/duration/alpha values and advances on its owning group's ordered timeline; completion callbacks identify that result and fire once.
- [ ] A template declared in an XML addon loaded through `load_addon` supplies duration, order, delays, and smoothing to the runtime factory result. An inline XML control checks that the same configuration is supported by the loader.
- [x] Run the same ordinary factory tests under PTR and earlier retail. Both pass optional-argument and named-lifecycle cases and reproduce the same template failure.

## How it works

- [Animation query lifecycle](animation-query-lifecycle.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/creation.rs`: existing runtime factory and owner registration; unchanged by this test slice.
- `src/loader/xml_file.rs`, `src/loader/helpers_anim.rs`: real XML loading and animation configuration; unchanged by this test slice.

## Tests asserting this spec

- `tests/animation_factory.rs`: direct optional arguments, named result lifecycle, and XML template boundary.
- `tests/animation_group.rs`: existing basic creation/configuration/parent/enumeration coverage, reused rather than replaced.
- `tests/xml_animation_group_onload.rs`: existing loader-based animation-group template fixture.

Focused results: **12 passed, 1 failed per profile** (two new ordinary tests and ten existing tests pass; the new template regression fails). Root `cargo fmt` completed before testing. Logs: `/tmp/animation-factory-ptr.log` and `/tmp/animation-factory-retail.log`.

Commands (executed separately for `ptr` and `retail`):

```text
cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- animation_factory:: animation_group::create_animation_returns_handle animation_group::animation_duration animation_group::animation_from_to_alpha animation_group::animation_order animation_group::animation_smoothing animation_group::animation_get_parent animation_group::animation_get_region_parent animation_group::group_get_animations xml_animation_group_onload:: --nocapture
```

The `animation_order` filter also selects the existing sequence-order test. No broad suite or final gate ran.

## Known gaps (current cycle)

- [ ] Reproduced on PTR and retail: the inline loader-created control has duration `1.75`, but the named runtime result created with `FactoryTimingTemplate` has duration `0`. The template test stops at that first mismatch; inherited order/delays/smoothing remain unproven. Current `create_animation` does not read `templateName`, and `xml_file.rs` ignores top-level `Animation` declarations. Production files remain unchanged.
- [ ] Default order/duration and timeline expectations describe simulator behavior, not independent native-client observations.

## Out of scope

Production changes, new security enforcement, native error/coercion claims, forbidden-aspect proof, broad suites, readability/check gates, audit artifact credit, delegation, publishing, and deployment are excluded from this bounded test task.
