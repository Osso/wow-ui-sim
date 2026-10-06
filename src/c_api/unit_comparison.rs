//! Shared comparison-permission policy for UnitIsUnit and C_Secrets.
//! INFERRED token policy retained from the existing UnitIsUnit boundary.

fn is_base_comparison_token(unit: &str) -> bool {
    matches!(
        unit,
        "player"
            | "pet"
            | "vehicle"
            | "mouseover"
            | "target"
            | "softenemy"
            | "softfriend"
            | "softinteract"
            | "focus"
            | "none"
            | "npc"
            | "questnpc"
    )
}

fn is_group_comparison_token(unit: &str) -> bool {
    const GROUP_TOKEN_LIMITS: [(&str, u8); 4] =
        [("party", 4), ("partypet", 4), ("raid", 40), ("raidpet", 40)];
    GROUP_TOKEN_LIMITS.iter().any(|(prefix, limit)| {
        let Some(suffix) = unit.strip_prefix(prefix) else {
            return false;
        };
        let Ok(index) = suffix.parse::<u8>() else {
            return false;
        };
        let canonical = suffix == index.to_string();
        canonical && (1..=*limit).contains(&index)
    })
}

fn is_restricted_comparison_counterpart(unit: &str) -> bool {
    unit.starts_with("nameplate") || unit.ends_with("target")
}

pub(crate) fn permitted(lhs: Option<&str>, rhs: Option<&str>) -> bool {
    let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
        // Inferred compatibility policy: missing tokens retain false, not denial.
        return true;
    };
    if is_base_comparison_token(lhs) || is_base_comparison_token(rhs) {
        return true;
    }
    let lhs_allows = is_group_comparison_token(lhs) && !is_restricted_comparison_counterpart(rhs);
    let rhs_allows = is_group_comparison_token(rhs) && !is_restricted_comparison_counterpart(lhs);
    lhs_allows || rhs_allows
}
