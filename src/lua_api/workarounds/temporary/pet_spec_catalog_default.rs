//! User-approved simulator default, not native evidence.
//! Retire when a pet specialization catalog supplies the initial observation.

use crate::c_api::c_stable_info::pet_spec_infos::PetSpecInfo;

pub(crate) fn default_catalog() -> Vec<PetSpecInfo> {
    Vec::new()
}
