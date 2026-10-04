
## Active goal
Prove C01–C10 in isolated p1210-r1 with focused behavioral tests and per-batch specs/commits. Exclude vendor/cache changes, wiki, coverage JSON, push/merge/deploy/PR and agents. Final fmt/check; four jobs.
Step 1: inspected existing worktree.
Step 2: read C01–C10 batch/source/triage evidence and existing creation, update-mode and forbidden-aspect tests.
Step 3: mapped existing behavior tests; cache read-only inspection confirms public group path uses managed templates and preallocated native frames, not mock-provider proof.
Step 4: C01 adds tainted addon combat construction, nondefault size/visibility/parent readback. Proof ledger: ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_creation', '--', '--test-threads=1']; ebade248f + new test; first invocation.
Step 5: C01 test exit 0; log /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C01-patch_12_1_0_aura_creation.log.
Proof ledger C01: ebade248f799b1eb8788e2ccd3e81e1cd1d60bfd + current scoped test changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'aura_button_icon_and_overlay_border_follow_real_initializer_order', '--', '--test-threads=1']; no prior equivalent invocation.
C01: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C01-aura_button_icon_and_overlay_border_follow_real_initializer_order.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10461 filtered out; finished in 0.92s
## C01 — 3e7380868958d50052668893d9635a467a74a35b
Added combat creation test and reused real initializer: 2 PASS / 0 FAIL. L9/L37/L207 bounded-coverage for creation only; managed presentation/security unproved. Readability manually inspected, no suppressions.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
Step C02: locating actual migrated helper definitions and cursor state; C03 existing full lifecycle/dependency tests located for reuse.
C02 helper lookup ongoing; native IsMouseOver offset order differs from cached top/bottom/left/right forwarding; must prove with unequal offsets.
C02 inspected mouse fixtures and dialog behavior; C03 inspected startup-loader enabled boundary.
C02 added real SharedXML mouse hit/offset/hidden readback and addon success/one-time/missing/disabled loader+dialog tests; no mocks or vendor edits.
Proof ledger C02: 3e7380868958d50052668893d9635a467a74a35b + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_migrated', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C02: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C02-patch_12_1_0_migrated.log; test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10462 filtered out; finished in 0.99s
C02 failure boundary initially unlabelled; adding diagnostic assertion messages only, preserving expected behavior. Focused diagnostic rerun required.
Proof ledger C02-diagnostic: 3e7380868958d50052668893d9635a467a74a35b + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_migrated_loader', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C02-diagnostic: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C02-diagnostic-patch_12_1_0_migrated_loader.log; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10463 filtered out; finished in 0.28s
C02 diagnostic isolated failure to disabled fixture (first block succeeded); investigate registration precondition before treating it as producer failure.
C02 fixture correction: DisableAddOn only mutates registered inventory. Register real host fixture before disabling; assertions unchanged, added enable-state check. No producer bug demonstrated.
Proof ledger C02-green: 3e7380868958d50052668893d9635a467a74a35b + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_migrated', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C02-green: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C02-green-patch_12_1_0_migrated.log; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10462 filtered out; finished in 0.48s
## C02 — ad0ef3ab35f5a2874e567c646f1fa958b59f0f58
L11/L12 bounded-coverage for relocated real calls/removal; 2 PASS / 0 FAIL final; initial 1 PASS / 1 FAIL and diagnostic 0/1 were unregistered disabled fixture, corrected without weakening assertions. Native mouse offset naming/scaling unproved; no row reclassification. Readability manually inspected.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C03 read actual bootstrap producer; reuse lifecycle/dependency tests, inspect disabled startup ownership boundary before adding proof.
Proof ledger C03: ad0ef3ab35f5a2874e567c646f1fa958b59f0f58 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'lod_bootstrap_lifecycle_', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C03: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C03-lod_bootstrap_lifecycle_.log; test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10461 filtered out; finished in 0.36s
Proof ledger C03: ad0ef3ab35f5a2874e567c646f1fa958b59f0f58 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'bootstrap_owner_dependency_waits_for_full_load', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C03: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C03-bootstrap_owner_dependency_waits_for_full_load.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10463 filtered out; finished in 0.40s
## C03 — e05eb7b9e109b452e0e46f2265eff05ef42a4fd2
Reused four load_order tests: 4 PASS / 0 FAIL. L55 bounded-coverage; L56 audit-pending (binary disabled-startup proof not run under allowed commands); L57 partial-development-green for loader mechanism, actual full migration unproved. No row reclassification.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C04 reuse existing five mode tests; known managed-aura test failure will not earn proof.
Proof ledger C04: e05eb7b9e109b452e0e46f2265eff05ef42a4fd2 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'on_update_modes::', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C04: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C04-on_update_modes-.log; test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 10459 filtered out; finished in 0.95s
## C04 — bd4d6007f38fe451bc431f2b3a8bf0ba5ef0bc8c
Reused on_update_modes:: 4 PASS / 1 FAIL. L58/L59 bounded-coverage for native dispatch only; managed integration unproved. Failure log: Dirty flags were not fully cleared during update pass (remaining flags: 18). Root cause: OnShow requests FullAuraRebuild=19, fixture keeps only custom phase1; remaining18 cannot clear. No producer fix/reclassification.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C05 existing runtime_template_local_mixin_and_key_value_apply proves nested local string lookup but not table identity/cross-addon isolation; add concrete full addon fixtures.
C05 new two-addon XML test asserts payload identity, live shared updates, literal/runtime template routes, nested local/global mixins and last local entry precedence.
Proof ledger C05: bd4d6007f38fe451bc431f2b3a8bf0ba5ef0bc8c + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_xml', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C05: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C05-patch_12_1_0_xml.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10464 filtered out; finished in 0.15s
## C05 — d8fb7b207df53da9038c74fb82d80e9dbf89ce9f
Added real two-addon fixture: 1 PASS / 0 FAIL. L64/L65/L66/L67 bounded-coverage for local identity, nested resolution, entry precedence and no leakage. Invalid path/type and delegation security exhaustive scope unproved. Readability manual review: linear fixture, single short loop, no warnings/suppressions.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C06 generic aspect tests already behaviorally cover mutation rejection and listener/callback preservation; reuse instead of duplicating.
Proof ledger C06: d8fb7b207df53da9038c74fb82d80e9dbf89ce9f + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'event_registration_aspect_', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C06: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C06-event_registration_aspect_.log; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10460 filtered out; finished in 0.49s
C06 added actual custom container intrinsic aspect with tainted RegisterEvent/UnitEvent/AllEvents rejection, listener readback and preserved caller taint.
Proof ledger C06: d8fb7b207df53da9038c74fb82d80e9dbf89ce9f + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_container_rejects', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C06: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C06-patch_12_1_0_aura_container_rejects.log; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10465 filtered out; finished in 0.62s
C06 new intrinsic test failed; added diagnostic messages without changing assertions. Generic event rejection proof remains 5/5, intrinsic row not yet credited.
Proof ledger C06-diagnostic: d8fb7b207df53da9038c74fb82d80e9dbf89ce9f + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_container_rejects', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C06-diagnostic: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C06-diagnostic-patch_12_1_0_aura_container_rejects.log; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10465 filtered out; finished in 1.17s
C06 failure reproduction (excluded from commit per instruction):
```rust
//! Aura-container intrinsic event registration restriction through real templates.
#![cfg(feature = "retail-12-1-0")]

#[test]
fn patch_12_1_0_aura_container_rejects_addon_event_registration() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
        &["Blizzard_AuraContainer"], &[], |env, _| {
            env.exec(r#"
                local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                local function addon()
                    assert(bit.band(container:GetForbiddenAspects(), Enum.ForbiddenAspect.EventRegistrations) ~= 0,
                        'AuraContainer lacks intrinsic EventRegistrations aspect: ' .. tostring(container:GetForbiddenAspects()))
                    for _, operation in ipairs({
                        function() container:RegisterEvent('PLAYER_LOGIN') end,
                        function() container:RegisterUnitEvent('UNIT_HEALTH', 'player') end,
                        function() container:RegisterAllEvents() end,
                    }) do
                        local ok, err = pcall(operation)
                        assert(not ok and type(err) == 'string', 'intrinsic aspect must reject addon registrations')
                        assert(not container:IsEventRegistered('PLAYER_LOGIN'), 'PLAYER_LOGIN registration changed')
                        assert(not container:IsEventRegistered('UNIT_HEALTH'), 'UNIT_HEALTH registration changed')
                        assert(debug.getstacktaint() == 'AuditEventsAddon', 'caller taint changed')
                    end
                end
                debug.setobjecttaint(addon, 'AuditEventsAddon')
                addon()
                assert(issecure())
            "#).unwrap();
        },
    );
}

```
L88 finding: forbidden mask0 on actual cached AuraContainer route; cache intrinsic XML lacks EventRegistrations declaration. Requires intrinsic model correctness, not shim; recommended modelable / audit-pending. No production fix.

## C06 — b7dafca9e3f323fce2b612f6f0a5f22649226837
L83 bounded-coverage: 5 PASS / 0 FAIL reused generic tests. L88 new proof 0 PASS / 1 FAIL (twice incl diagnostic); AuraContainer lacks intrinsic EventRegistrations aspect: 0. Failing test archived in this result, removed from commit, not weakened. L88 recommended implemented-needs-proof -> modelable, status audit-pending. Cache intrinsic XML does not apply aspect; no producer fix.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
Proof ledger C07: b7dafca9e3f323fce2b612f6f0a5f22649226837 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'native_children_inherit_only_hierarchy_aspects_at_creation', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C07: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C07-native_children_inherit_only_hierarchy_aspects_at_creation.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10464 filtered out; finished in 0.14s
## C07 — 84be218c78e580085a47d15b436e2bcd1885d9c2
Reused native_children_inherit_only_hierarchy_aspects_at_creation: 1 PASS / 0 FAIL. L90 bounded-coverage for rejection and zero-baseline state preservation; existing nonempty anchors/every aspect bit unproved. No reclassification.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C08/C09 read real managed group options, sorting and provider lifecycle; preparing nonempty native aura fixtures with actual frames, callbacks and templates.
C07 committed reused rejection proof; C08/C09 choose existing ManagedAuraContainer + CustomAuraContainerTemplate route (same as existing actual managed test), enumerate real owned frames and assert visibility instead of counting preallocation as aura presentation.
C08/C09 native fixture design: three nonempty helpful auras, different sources/durations; real dirty dispatch, group max-count/filter transitions; sort order read from actual frame positions.
C08/C09 added independent real-path tests before next build: C08 three native auras, multiple groups and filter/max-count presentation; C09 callback taint/identity, two addon templates, actual size/icon state. C09 sorting still needs presentation proof, not credited by callback test.
Proof ledger C08: 84be218c78e580085a47d15b436e2bcd1885d9c2 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_groups', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C08: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C08-patch_12_1_0_aura_groups.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10466 filtered out; finished in 0.55s
Proof ledger C09: 84be218c78e580085a47d15b436e2bcd1885d9c2 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_group_options', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C09: exit 101; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C09-patch_12_1_0_aura_group_options.log; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10466 filtered out; finished in 0.35s
C08 real native presentation test PASS; C09 original custom KeyValue public read failed. Investigate partition ownership before classifying as producer bug; callback/template source only promises template application, not public access to forbidden-private KeyValues.
## C08 — 3ce443cfb40c2b06c1f23904d8c6294d60038b36
Added public real-managed-group test: 1 PASS / 0 FAIL. L119/L120/L122/L123/L125/L126 bounded-coverage for independent nonempty native filters, group key lifecycle, options/cap changes and actual frame visibility/ownership. Plain AuraContainer, secret/combat/removal-event integration and pixels unproved. Readability manually inspected.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
C09 L129 failure reproduction preserved; excluded from commit, not weakened:
```rust
//! Real AuraButton callback/template composition; no mock frame provider.
#![cfg(feature = "retail-12-1-0")]

#[test]
fn patch_12_1_0_aura_group_options_initialize_every_real_templated_button() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
        &["Blizzard_AuraContainer"], &[], |env, _| {
            let root = tempfile::tempdir().unwrap();
            let toc = root.path().join("AuditAuraTemplates.toc");
            std::fs::write(&toc, "Templates.xml\n").unwrap();
            std::fs::write(root.path().join("Templates.xml"), r#"<Ui>
                <AuraButton name="AuditAuraFirst" virtual="true"><KeyValues>
                    <KeyValue key="auditFirst" type="number" value="17"/>
                </KeyValues></AuraButton>
                <AuraButton name="AuditAuraSecond" virtual="true"><KeyValues>
                    <KeyValue key="auditSecond" type="number" value="29"/>
                </KeyValues></AuraButton>
            </Ui>"#).unwrap();
            let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
            assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
            env.exec(r#"
                local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
                local callbacks, seen = 0, {}
                local function initialize(frame)
                    assert(debug.getstacktaint() == 'AuditInitAddon', 'callback retains addon taint')
                    assert(frame:GetObjectType() == 'AuraButton' and frame:GetParent() == container)
                    assert(frame.auditFirst == 17 and frame.auditSecond == 29, 'both templates precede callback')
                    assert(not seen[frame], 'callback exactly once per created frame')
                    seen[frame] = true
                    callbacks = callbacks + 1
                    frame:SetSize(23, 31)
                    local icon = frame:CreateTexture(nil, 'BACKGROUND')
                    icon:SetAllPoints(frame)
                    frame:SetIcon(icon)
                    assert(frame:GetIcon() == icon, 'CustomAuraButtonTemplate methods retained')
                end
                debug.setobjecttaint(initialize, 'AuditInitAddon')
                container:AddAuraGroup('templated:group', 'HELPFUL', {
                    templateNames={'AuditAuraFirst', 'AuditAuraSecond'}, initializeFrame=initialize,
                })
                local count = container:GetAuraGroupFrameCount('templated:group')
                assert(count > 0 and callbacks == count, 'every created real button initialized')
                for index=1,count do
                    local frame = container:GetAuraGroupFrame('templated:group', index)
                    assert(seen[frame] and frame:GetWidth() == 23 and frame:GetHeight() == 31)
                    assert(frame:GetIcon():GetNumPoints() > 0)
                end
                assert(issecure())
            "#).unwrap();
        },
    );
}

```
Observed: both templates precede callback assertion failed. Missing public fields might be forbidden-partition ownership rather than absent template application; root cause not yet established. Keep L129 audit-pending, do not count callbacks after this failure. Next independent tests cover L128 without additional templates and L127 actual visible order.

C09 added independent L128 callback proof without extra templates and L127 nondefault AuraInstanceIDOnly sorting/readback by actual rendered-frame icon identities and x positions, then reverse transition; original L129 test remains excluded.
Proof ledger C09-independent: 3ce443cfb40c2b06c1f23904d8c6294d60038b36 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_group_options', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C09-independent: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C09-independent-patch_12_1_0_aura_group_options.log; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10466 filtered out; finished in 1.29s
C10 reuses exact modern/legacy state deletion, zero-return and cached deprecated-wrapper forwarding tests; no duplicate tests needed.
Proof ledger C10: 3ce443cfb40c2b06c1f23904d8c6294d60038b36 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'direct_modern_and_legacy_removers_share_real_state', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C10: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C10-direct_modern_and_legacy_removers_share_real_state.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10467 filtered out; finished in 0.29s
Proof ledger C10: 3ce443cfb40c2b06c1f23904d8c6294d60038b36 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C10: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C10-actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10467 filtered out; finished in 0.31s
C09 independent L127/L128 green 2/2; C10 modern/legacy and cached vendor forwarding green 2/2. Investigating L129 failed public KeyValue access against actual template partition to bound any reclassification.
C09 next independent L129 proof uses externally observable template size/alpha rather than assuming private KeyValues are public. Original public-field failure retained verbatim; no producer change or assertion weakening.
Proof ledger C09-templates: 3ce443cfb40c2b06c1f23904d8c6294d60038b36 + current scoped changes; ['python3', 'scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--test', 'integration', 'patch_12_1_0_aura_group_options_apply_additional', '--', '--test-threads=1']; first invocation on relevant scope unless explicit retest noted.
C09-templates: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-C09-templates-patch_12_1_0_aura_group_options_apply_additional.log; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10468 filtered out; finished in 0.44s
## C09 — 184c07ec822b9327c1f17fd6e9ab615fb4fc67e8
L127/L128/L129 bounded-coverage: 3 PASS / 0 FAIL independent final tests. Initial public KeyValue-field assumption failed 0/1, archived outside commit; root partition ownership unproved, not credited. Additional-template size/alpha BEFORE callback and base methods proven independently, no producer failure demonstrated for source L129. Readability manually inspected; explicit short fixture/helper and concrete input-output order, no suppressions. No reclassification.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
## C10 — 4c9d702c715087e3520e890e819d587187219977
Reused modern/legacy shared-state and actual cached deprecated wrapper tests: 2 PASS / 0 FAIL. L181 bounded-coverage for zero-result deletion/readback and tainted legacy forwarding. Playback/acquisition/historical aliases unproved; no reclassification.
Producer fixes: none. Coverage JSON unchanged; recommendations only.
Final integration: ten batch commits complete; 8 new passing tests plus 17 reused passes; known C04 managed fixture fail and excluded C06 intrinsic aspect finding retained. Auditing spec inventories/row IDs before final fmt/check.
Final docs audit found inaccurate inventory paths and C01 L37 date ID; correcting descriptions only, existing test proofs remain valid.
Documentation correction commit f79c0f898aa04988fef7b6741d046d73b216923f: corrected L37 ID to prose-2026-06-18-037 and inventory paths. No Rust/producer changes; prior focused proof scopes remain valid.
Final proof ledger: f79c0f898aa04988fef7b6741d046d73b216923f; ['cargo', 'fmt']; first final invocation, default profile, four jobs; no broad test suite.
Final fmt: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-final-fmt.log
Final proof ledger: f79c0f898aa04988fef7b6741d046d73b216923f; ['cargo', 'fmt', '--check']; first final invocation, default profile, four jobs; no broad test suite.
Final fmt-check: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-final-fmt-check.log
Final proof ledger: f79c0f898aa04988fef7b6741d046d73b216923f; ['cargo', 'check', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b100', '--jobs', '4']; first final invocation, default profile, four jobs; no broad test suite.
Final check: exit 0; /home/osso-test/.cache/wow-ui-sim-audit/p1210-r1-final-check.log
Final state: ## p1210-r1 (clean). All tracked edits are tests/patch_12_1_0_*.rs or docs/specs/patch-12-1-0-*.md; no producer/vendor/wiki/coverage changes, no push/merge/deploy/PR/model/agent actions.
f79c0f898 Correct audit row ID and producer inventory paths
4c9d702c7 Prove 12.1.0 C10 aura-sound-remove-proof
184c07ec8 Prove 12.1.0 C09 aura-group-options
3ce443cfb Prove 12.1.0 C08 aura-groups
84be218c7 Prove 12.1.0 C07 inheritance
b7dafca9e Prove 12.1.0 C06 events
d8fb7b207 Prove 12.1.0 C05 xml
bd4d6007f Prove 12.1.0 C04 onupdate
e05eb7b9e Prove 12.1.0 C03 bootstrap
ad0ef3ab3 Prove 12.1.0 C02 framexml-migrations
3e7380868 Prove 12.1.0 C01 aura-creation

## Final focused proof inventory
25 unique selected passing tests; 1 known existing failed test. Separate excluded intrinsic finding is not a committed failing test. Earlier diagnostic/fixture failures are not added to final counts.
- PASS `forbidden_aspect_creation::aura_button_icon_and_overlay_border_follow_real_initializer_order`
- PASS `forbidden_aspect_creation::event_registration_aspect_leaves_zero_mask_callback_delivery_unchanged`
- PASS `forbidden_aspect_creation::event_registration_aspect_leaves_zero_mask_mutations_and_delivery_unchanged`
- PASS `forbidden_aspect_creation::event_registration_aspect_preserves_callbacks_when_replacement_is_rejected`
- PASS `forbidden_aspect_creation::event_registration_aspect_preserves_existing_listener_delivery`
- PASS `forbidden_aspect_creation::event_registration_aspect_rejects_all_mutations_without_caller_exception`
- PASS `forbidden_aspect_creation::native_children_inherit_only_hierarchy_aspects_at_creation`
- PASS `load_order::bootstrap_owner_dependency_waits_for_full_load`
- PASS `load_order::lod_bootstrap_lifecycle_eager_files_keep_literal_toc_order`
- PASS `load_order::lod_bootstrap_lifecycle_preserves_mixed_stream_order`
- PASS `load_order::lod_bootstrap_lifecycle_publishes_once_before_full_load`
- PASS `on_update_modes::on_update_modes_obey_ancestor_visibility_and_one_shots`
- PASS `on_update_modes::on_update_modes_publish_numeric_contract_and_default`
- PASS `on_update_modes::on_update_modes_reset_before_handlers_and_preserve_rearming`
- PASS `on_update_modes::on_update_modes_xml_names_select_numeric_modes`
- PASS `patch_12_1_0_aura_creation::patch_12_1_0_aura_creation_in_combat_preserves_parent_and_frame_state`
- PASS `patch_12_1_0_aura_group_options::patch_12_1_0_aura_group_options_apply_additional_template_size_and_alpha`
- PASS `patch_12_1_0_aura_group_options::patch_12_1_0_aura_group_options_initialize_each_native_button_without_extra_templates`
- PASS `patch_12_1_0_aura_group_options::patch_12_1_0_aura_group_options_sort_direction_changes_actual_icon_positions`
- PASS `patch_12_1_0_aura_groups::patch_12_1_0_aura_groups_filter_limit_and_refresh_actual_frames`
- PASS `patch_12_1_0_framexml_migrations::patch_12_1_0_migrated_loader_handles_success_missing_and_disabled_addons`
- PASS `patch_12_1_0_framexml_migrations::patch_12_1_0_migrated_mouse_helper_queries_real_frame_with_offsets`
- PASS `patch_12_1_0_xml::patch_12_1_0_xml_preserves_private_identity_nested_mixins_and_addon_isolation`
- PASS `private_aura_sound_removal::actual_cached_deprecated_file_keeps_legacy_removal_durable_after_bootstrap`
- PASS `private_aura_sound_removal::direct_modern_and_legacy_removers_share_real_state`
- FAIL `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases`

## Per-row recommendation matrix
Recommendations only; coverage JSON and triage files not modified. All statuses are bounded simulator proof, not native acceptance.
| Batch | Exact row ID | Proven scope | Unproved scope | Recommended status/classification |
|---|---|---|---|---|
| C01 | `prose-undated-009` | Construction, combat addon taint and actual CustomAuraButton initialization | Managed secrecy and pixels | bounded-coverage |
| C01 | `prose-2026-06-18-037` | Construction, combat addon taint and actual CustomAuraButton initialization | Managed secrecy and pixels | bounded-coverage |
| C01 | `prose-2026-07-23-207` | Construction, combat addon taint and actual CustomAuraButton initialization | Managed secrecy and pixels | bounded-coverage |
| C02 | `prose-undated-011` | Real migrated loader/dialog and native mouse-offset forwarding; old helper absence | Native scaling/offset naming parity | bounded-coverage |
| C02 | `prose-undated-012` | Real migrated loader/dialog and native mouse-offset forwarding; old helper absence | Native scaling/offset naming parity | bounded-coverage |
| C03 | `prose-2026-06-18-055` | Real phased LoD lifecycle, order, later full-load/idempotence and dependencies | Disabled startup and complete vendor UIParent migration | bounded-coverage |
| C03 | `prose-2026-06-18-056` | No executed disabled-startup proof | Disabled startup and complete vendor UIParent migration | audit-pending |
| C03 | `prose-2026-06-18-057` | Real phased LoD lifecycle, order, later full-load/idempotence and dependencies | Disabled startup and complete vendor UIParent migration | partial-development-green |
| C04 | `prose-2026-06-18-058` | Numeric modes, default/rejection, visibility, one-shot/rearm and XML dispatch | Known managed dirty-phase fixture fails mask18 | bounded-coverage |
| C04 | `prose-2026-06-18-059` | Numeric modes, default/rejection, visibility, one-shot/rearm and XML dispatch | Known managed dirty-phase fixture fails mask18 | bounded-coverage |
| C05 | `prose-2026-06-18-064` | Local table identity/live updates, block/global nested mixins, precedence and cross-addon isolation | Exhaustive invalid-path/security combinations | bounded-coverage |
| C05 | `prose-2026-06-18-065` | Local table identity/live updates, block/global nested mixins, precedence and cross-addon isolation | Exhaustive invalid-path/security combinations | bounded-coverage |
| C05 | `prose-2026-06-18-066` | Local table identity/live updates, block/global nested mixins, precedence and cross-addon isolation | Exhaustive invalid-path/security combinations | bounded-coverage |
| C05 | `prose-2026-06-18-067` | Local table identity/live updates, block/global nested mixins, precedence and cross-addon isolation | Exhaustive invalid-path/security combinations | bounded-coverage |
| C06 | `prose-2026-06-23-083` | Generic aspect mutations reject; original listeners/callback delivery preserved | Intrinsic AuraContainer lacks mask (0 observed) | bounded-coverage |
| C06 | `prose-2026-06-23-088` | No intrinsic enforcement proof; actual mask0 finding | Intrinsic AuraContainer lacks mask (0 observed) | audit-pending; reclassify modelable |
| C07 | `prose-2026-06-23-090` | Implicit forbidden-aspect parent/anchor acquisition rejects without zero-baseline mutation | All bits and preexisting nonempty anchor tuple preservation | bounded-coverage |
| C08 | `prose-2026-07-07-119` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C08 | `prose-2026-07-07-120` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C08 | `prose-2026-07-07-122` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C08 | `prose-2026-07-07-123` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C08 | `prose-2026-07-07-125` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C08 | `prose-2026-07-07-126` | Native nonempty managed group filters, arbitrary keys, options/cap transitions and visible real frames | Plain AuraContainer and combat-secret/removal-event lifecycle | bounded-coverage |
| C09 | `prose-2026-07-07-127` | Actual icon position ordering normal/reverse, every frame callback taint, template size/alpha and base methods | Arbitrary-field partition publication, other sorts/secret lifetime | bounded-coverage |
| C09 | `prose-2026-07-07-128` | Actual icon position ordering normal/reverse, every frame callback taint, template size/alpha and base methods | Arbitrary-field partition publication, other sorts/secret lifetime | bounded-coverage |
| C09 | `prose-2026-07-07-129` | Actual icon position ordering normal/reverse, every frame callback taint, template size/alpha and base methods | Arbitrary-field partition publication, other sorts/secret lifetime | bounded-coverage |
| C10 | `prose-2026-07-21-181` | Modern/legacy zero-result removal on real ID set and cached vendor forwarding | Playback, acquisition and historical timing | bounded-coverage |
Final gates at f79c0f898: cargo fmt, cargo fmt --check and cargo check default target b100/jobs4 all exit0, no warnings. No test rerun needed: later changes are additive test cases/docs/formatting only, no producer changes intersect prior proof scopes.
