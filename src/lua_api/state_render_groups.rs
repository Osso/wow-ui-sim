//! Keep top-level-owned render segments together across their local strata.

use super::super::state::SimState;
use std::collections::HashMap;

type Owners = HashMap<u64, u64>;

struct ToplevelGroup {
    strata: usize,
    ids: Vec<u64>,
}

impl SimState {
    pub(super) fn regroup_toplevel_subtrees(&self, buckets: Vec<Vec<u64>>) -> Vec<Vec<u64>> {
        let (owners, mut groups) = self.collect_toplevel_groups(&buckets);
        let mut grouped = vec![Vec::new(); buckets.len()];
        for (strata, ids) in buckets.into_iter().enumerate() {
            for id in ids {
                let Some(owner) = owners.get(&id) else {
                    grouped[strata].push(id);
                    continue;
                };
                if groups
                    .get(owner)
                    .is_some_and(|group| group.strata == strata)
                {
                    let group = groups.remove(owner).expect("eligible group exists");
                    grouped[strata].extend(group.ids);
                }
            }
        }
        debug_assert!(
            groups.is_empty(),
            "top-level group has no owner-strata anchor"
        );
        grouped
    }

    fn collect_toplevel_groups(
        &self,
        buckets: &[Vec<u64>],
    ) -> (Owners, HashMap<u64, ToplevelGroup>) {
        let mut owners = HashMap::new();
        let mut groups = HashMap::<u64, ToplevelGroup>::new();
        let mut raised_cache = HashMap::new();
        let mut unraised_cache = HashMap::new();
        // Existing raw buckets preserve local strata, level and region ordering.
        for &id in buckets.iter().flatten() {
            // Retain existing nested active-owner selection; unraised roots
            // also isolate their descendants from independent strata roots.
            let owner = self
                .nearest_toplevel_owner(id, &mut raised_cache, true)
                .or_else(|| self.nearest_toplevel_owner(id, &mut unraised_cache, false));
            let Some((owner, _)) = owner else { continue };
            owners.insert(id, owner);
            groups
                .entry(owner)
                .or_insert_with(|| {
                    let frame = self.widgets.get(owner).expect("top-level owner exists");
                    ToplevelGroup {
                        strata: self.frame_bucket_strata(frame).as_index(),
                        ids: Vec::new(),
                    }
                })
                .ids
                .push(id);
        }
        (owners, groups)
    }
}
