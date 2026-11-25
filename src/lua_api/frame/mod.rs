//! Minimal frame helpers on the rilua path.

pub(crate) mod methods;

pub(crate) use methods::methods_hierarchy::{
    propagate_strata_level_pub, recalculate_strata_level_pub,
};
