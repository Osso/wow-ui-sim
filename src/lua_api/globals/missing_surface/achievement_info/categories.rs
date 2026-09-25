#[derive(Clone, Copy)]
pub(super) enum CategoryListKind {
    Achievement,
    Guild,
    Statistics,
}

#[derive(Clone, Copy)]
pub(super) struct CategoryBucket {
    pub(super) category_id: i32,
    pub(super) name: &'static str,
    pub(super) parent_id: i32,
    pub(super) flags: i32,
    pub(super) achievement_ids: &'static [i32],
}

impl CategoryBucket {
    const fn new(
        category_id: i32,
        name: &'static str,
        parent_id: i32,
        flags: i32,
        achievement_ids: &'static [i32],
    ) -> Self {
        Self {
            category_id,
            name,
            parent_id,
            flags,
            achievement_ids,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct AchievementCriterion {
    pub(super) id: i32,
    pub(super) name: &'static str,
    pub(super) required_quantity: i32,
}

impl AchievementCriterion {
    const fn new(id: i32, name: &'static str, required_quantity: i32) -> Self {
        Self {
            id,
            name,
            required_quantity,
        }
    }
}

const GENERAL_ACHIEVEMENT_IDS: &[i32] = &[6, 7, 8, 9, 10, 11];
const EXPLORATION_ACHIEVEMENT_IDS: &[i32] = &[42, 776];
const PVP_ACHIEVEMENT_IDS: &[i32] = &[513, 558];
const REPUTATION_ACHIEVEMENT_IDS: &[i32] = &[948];
const REPUTATION_EXALTED_ACHIEVEMENT_IDS: &[i32] = &[1017];

const ACHIEVEMENT_CATEGORIES: &[CategoryBucket] = &[
    CategoryBucket::new(92, "General", -1, 0, GENERAL_ACHIEVEMENT_IDS),
    CategoryBucket::new(96, "Quests", -1, 0, &[]),
    CategoryBucket::new(97, "Exploration", -1, 0, EXPLORATION_ACHIEVEMENT_IDS),
    CategoryBucket::new(15522, "Character", -1, 0, &[]),
    CategoryBucket::new(95, "Player vs. Player", -1, 0, PVP_ACHIEVEMENT_IDS),
    CategoryBucket::new(168, "Dungeons & Raids", -1, 0, &[]),
    CategoryBucket::new(169, "Professions", -1, 0, &[]),
    CategoryBucket::new(201, "Reputation", -1, 0, REPUTATION_ACHIEVEMENT_IDS),
    CategoryBucket::new(
        202,
        "Exalted Reputations",
        201,
        0,
        REPUTATION_EXALTED_ACHIEVEMENT_IDS,
    ),
    CategoryBucket::new(155, "World Events", -1, 0, &[]),
    CategoryBucket::new(15117, "Expansion Features", -1, 0, &[]),
    CategoryBucket::new(15246, "Collections", -1, 0, &[]),
    CategoryBucket::new(81, "Feats of Strength", -1, 0, &[]),
];

const GUILD_CATEGORY_ID: i32 = 15076;
const GUILD_CATEGORIES: &[CategoryBucket] = &[
    CategoryBucket::new(15076, "Guild", -1, 0, &[]),
    CategoryBucket::new(15088, "Guild Summary", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15077, "General", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15078, "Quests", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15079, "Player vs. Player", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15080, "Dungeons & Raids", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15089, "Professions", GUILD_CATEGORY_ID, 0, &[]),
    CategoryBucket::new(15093, "Guild Feats of Strength", GUILD_CATEGORY_ID, 0, &[]),
];

const STATISTICS_CATEGORIES: &[CategoryBucket] = &[
    CategoryBucket::new(130, "Statistics", -1, 0, &[]),
    CategoryBucket::new(1, "General", 130, 0, &[]),
    CategoryBucket::new(122, "Deaths", 130, 0, &[]),
    CategoryBucket::new(124, "Player vs. Player", 130, 0, &[]),
    CategoryBucket::new(128, "Wealth", 130, 0, &[]),
];

const AMBASSADOR_CRITERIA: &[AchievementCriterion] = &[
    AchievementCriterion::new(5331, "Exalted with Stormwind", 1),
    AchievementCriterion::new(5330, "Exalted with Ironforge", 1),
    AchievementCriterion::new(5328, "Exalted with Darnassus", 1),
    AchievementCriterion::new(5329, "Exalted with Gnomeregan", 1),
    AchievementCriterion::new(5332, "Exalted with Exodar", 1),
];

const VETERAN_CRITERIA: &[AchievementCriterion] =
    &[AchievementCriterion::new(13253, "Honorable kills", 100)];

pub(super) fn categories_for_view(is_guild_view: bool) -> &'static [CategoryBucket] {
    if is_guild_view {
        GUILD_CATEGORIES
    } else {
        ACHIEVEMENT_CATEGORIES
    }
}

pub(super) fn achievement_categories() -> &'static [CategoryBucket] {
    ACHIEVEMENT_CATEGORIES
}

pub(super) fn categories_for(kind: CategoryListKind) -> &'static [CategoryBucket] {
    match kind {
        CategoryListKind::Achievement => ACHIEVEMENT_CATEGORIES,
        CategoryListKind::Guild => GUILD_CATEGORIES,
        CategoryListKind::Statistics => STATISTICS_CATEGORIES,
    }
}

pub(super) fn find_category(category_id: i32) -> Option<&'static CategoryBucket> {
    ACHIEVEMENT_CATEGORIES
        .iter()
        .chain(GUILD_CATEGORIES.iter())
        .chain(STATISTICS_CATEGORIES.iter())
        .find(|category| category.category_id == category_id)
}

pub(super) fn category_id_for_achievement(achievement_id: i32) -> Option<i32> {
    category_for_achievement(achievement_id).map(|category| category.category_id)
}

fn category_for_achievement(achievement_id: i32) -> Option<&'static CategoryBucket> {
    ACHIEVEMENT_CATEGORIES
        .iter()
        .find(|category| category.achievement_ids.contains(&achievement_id))
}

fn category_achievement_position(achievement_id: i32) -> Option<(&'static [i32], usize)> {
    let category = category_for_achievement(achievement_id)?;
    let position = category
        .achievement_ids
        .iter()
        .position(|&id| id == achievement_id)?;
    Some((category.achievement_ids, position))
}

pub(super) fn previous_achievement_id(achievement_id: i32) -> Option<i32> {
    let (achievement_ids, position) = category_achievement_position(achievement_id)?;
    position
        .checked_sub(1)
        .and_then(|index| achievement_ids.get(index))
        .copied()
}

pub(super) fn next_achievement_id(achievement_id: i32) -> Option<i32> {
    let (achievement_ids, position) = category_achievement_position(achievement_id)?;
    achievement_ids.get(position + 1).copied()
}

pub(super) fn collect_category_achievement_ids(category_id: i32) -> Vec<i32> {
    let mut achievement_ids = Vec::new();
    append_category_achievement_ids(category_id, &mut achievement_ids);
    achievement_ids
}

fn append_category_achievement_ids(category_id: i32, achievement_ids: &mut Vec<i32>) {
    let Some(category) = find_category(category_id) else {
        return;
    };
    achievement_ids.extend(category.achievement_ids.iter().copied());
    for child in ACHIEVEMENT_CATEGORIES
        .iter()
        .filter(|child| child.parent_id == category_id)
    {
        append_category_achievement_ids(child.category_id, achievement_ids);
    }
}

pub(super) fn criteria_for_achievement(
    achievement_id: i32,
) -> Option<&'static [AchievementCriterion]> {
    match achievement_id {
        513 => Some(VETERAN_CRITERIA),
        948 => Some(AMBASSADOR_CRITERIA),
        _ => None,
    }
}

pub(super) fn criterion_at(
    achievement_id: i32,
    criterion_index: i32,
) -> Option<&'static AchievementCriterion> {
    let index = usize::try_from(criterion_index.checked_sub(1)?).ok()?;
    criteria_for_achievement(achievement_id)?.get(index)
}

pub(super) fn criterion_by_id(
    achievement_id: i32,
    criterion_id: i32,
) -> Option<&'static AchievementCriterion> {
    criteria_for_achievement(achievement_id)?
        .iter()
        .find(|criterion| criterion.id == criterion_id)
}
