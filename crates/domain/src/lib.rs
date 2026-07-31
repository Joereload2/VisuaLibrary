//! Visual Library domain crate.
//!
//! Pure domain types and policies. No I/O, no SQLite, no Tauri.

pub mod asset;
pub mod found;
pub mod plan;

pub use asset::{
    approve, is_library_visible, mark_duplicate, reject, supersede, AssetStatus,
    AssetTransitionError,
};
pub use found::{decide_acquisition, field_matches, AcquisitionDecision};
pub use plan::{approve_plan, can_run_automatic, CoveragePlanStatus, PlanError};

/// Product display name (shared constant for shell / about).
pub const PRODUCT_NAME: &str = "Visual Library";

/// Scaffold health check used by workspace smoke tests.
pub fn scaffold_ok() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_ok_is_true() {
        assert!(scaffold_ok());
    }

    #[test]
    fn product_name_is_visual_library() {
        assert_eq!(PRODUCT_NAME, "Visual Library");
    }
}
