//! Native R88o.1 public shape with explicitly supplied simulator rows/cache.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::{
    WowLuaEnv,
    state::{
        AchievementEligibilityCriterion, AchievementEligibilityInputs, AchievementEligibilityNode,
        AchievementEligibilityOperation,
    },
};

fn attach(inputs: &mut AchievementEligibilityInputs, achievement: i32, root: i32) {
    inputs.achievement_tree_ids.insert(achievement, root);
    inputs.groups.insert(
        root,
        Some(AchievementEligibilityNode {
            id: root,
            children: Vec::new(),
        }),
    );
}

fn env_with_records() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    {
        let mut sim = env.state().borrow_mut();
        for id in [-12, i32::MIN] {
            let mut row = sim.achievements[&6].clone();
            row.achievement_id = id;
            sim.achievements.insert(id, row);
            attach(&mut sim.achievement_eligibility, id, id);
            sim.achievement_eligibility.groups.insert(id, None);
        }
        sim.world.earned_achievements.insert(6);
    }
    env
}

#[test]
fn achievement_eligible_api_checks_arguments_arity_and_number_conversion() {
    let env = env_with_records();
    env.exec(
        r##"
        local getter = assert(IsAchievementEligible, "legacy ordinary global")
        assert(rawget(__secureenv, "IsAchievementEligible") == getter, "initial secure table copy")
        local function usage(...)
            local ok, err = pcall(getter, ...)
            local expected = "Usage: IsAchievementEligible(achievementID)"
            assert(not ok and tostring(err):sub(-#expected) == expected,
                "wrong result or usage: " .. tostring(err))
        end
        usage()
        usage(nil)
        for _, input in ipairs({false, true, {}, function() end, "nonnumeric"}) do
            usage(input)
        end
        local function result(expected, ...)
            local function check(...)
                assert(select("#", ...) == 1 and type((...)) == "boolean" and (...) == expected)
            end
            check(getter(...))
        end
        result(true, 12.9, {}, false)
        result(false, -12.9)
        result(false, " -12.9 ")
        result(false, 2147483648)
        result(false, -2147483648.9)
        result(false, 0/0)
        result(false, math.huge)
        result(true, 2147483647.9)
        "##,
    )
    .expect("R88o.1 one boolean, R88o.1 ordinary Usage and i32 masked conversion");
}

#[test]
fn achievement_eligible_api_distinguishes_all_record_and_leaf_shapes() {
    let env = env_with_records();
    setup_tree_fixtures(&env);
    env.exec(
        r##"
        local function result(id, expected)
            local function check(...)
                assert(select("#", ...) == 1 and type((...)) == "boolean" and (...) == expected,
                    "ID " .. id .. " was " .. tostring((...)))
            end
            check(IsAchievementEligible(id))
        end
        result(99001, true)  -- no Achievement row, even though host has a tree id/null group
        result(7, true)      -- existing Achievement, configured tree id has no runtime group
        result(8, false)     -- group exists with null tree
        result(9, false)     -- root's CriteriaTree row is missing
        result(10, true)      -- an empty all node
        result(11, true)      -- default tree operation
        result(42, true)      -- leaf's Criteria record is missing
        result(776, true)     -- zero world-state ID bypasses the cache
        result(558, true)     -- cache absence does not compare as zero
        result(6, false)      -- earned; nested aggregate sees cached zero mismatch
        "##,
    )
    .expect("R88o.1 distinguishes missing and null independently from earned/display state");
    env.state()
        .borrow_mut()
        .achievement_eligibility
        .world_states
        .insert(1, 5);
    let matches: bool = env.eval("return IsAchievementEligible(6)").unwrap();
    assert!(
        matches,
        "same earned record passes when the nested leaf matches"
    );
}

fn setup_tree_fixtures(env: &WowLuaEnv) {
    use AchievementEligibilityOperation::{All, DefaultTrue, Leaf};
    let mut sim = env.state().borrow_mut();
    let inputs = &mut sim.achievement_eligibility;
    for id in [99001, 7, 8, 9, 10, 11, 42, 776, 558, 6] {
        attach(inputs, id, id);
    }
    inputs.groups.remove(&7);
    inputs.groups.insert(99001, None);
    inputs.groups.insert(8, None);
    inputs.criteria_tree.extend([
        (10, All),
        (11, DefaultTrue),
        (42, Leaf { criterion_id: 420 }),
        (776, Leaf { criterion_id: 7760 }),
        (558, Leaf { criterion_id: 5580 }),
        (6, All),
        (61, DefaultTrue),
        (62, Leaf { criterion_id: 620 }),
    ]);
    inputs.criteria.extend([
        (
            7760,
            AchievementEligibilityCriterion {
                world_state_id: 0,
                required_value: 1,
            },
        ),
        (
            5580,
            AchievementEligibilityCriterion {
                world_state_id: 2,
                required_value: 10,
            },
        ),
        (
            620,
            AchievementEligibilityCriterion {
                world_state_id: 1,
                required_value: 5,
            },
        ),
    ]);
    inputs
        .groups
        .get_mut(&6)
        .unwrap()
        .as_mut()
        .unwrap()
        .children = vec![
        AchievementEligibilityNode {
            id: 61,
            children: vec![],
        },
        AchievementEligibilityNode {
            id: 62,
            children: vec![],
        },
    ];
    inputs.world_states.extend([(0, 0), (1, 0)]);
}
