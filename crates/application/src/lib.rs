//! Application layer: use cases and ports.
//!
//! F1–F5 prior · F6 Coverage + Review completo.

pub mod assets;
pub mod catalog;
pub mod coverage;
pub mod error;
pub mod factory;
pub mod jobs;
pub mod plans;
pub mod ports;
pub mod settings;

pub use assets::{
    approve_asset, edit_asset_metadata, list_library_assets, list_waiting_review,
    mark_asset_duplicate, regenerate_asset, reject_asset, AssetDto,
};
pub use catalog::{
    ensure_concept, ensure_representation, ensure_theme, list_concepts, list_representations,
    list_themes, ConceptDto, RepresentationDto, ThemeDto,
};
pub use coverage::{get_coverage_report, CoverageIssue, CoverageReport, CoverageSummary};
pub use error::AppError;
pub use factory::{
    preview_manual_batch, run_automatic_from_plan, submit_manual_batch, AutomaticRunResult,
    ManualBatchPreview, ManualNeed, ManualNeedResult,
};
pub use jobs::{
    generate_stub_asset, media_writer_for, FsMediaWriter, GenerateStubInput, GenerateStubResult,
    JobDto, MediaWriter, STUB_PNG,
};
pub use plans::{
    add_plan_item, approve_coverage_plan, create_plan, get_plan_with_items, list_plans, PlanDto,
    PlanItemDto, PlanWithItemsDto,
};
pub use settings::{
    get_settings, update_media_root, validate_media_root, AppPathsDto, SettingsDto,
};
pub use visual_library_domain::PRODUCT_NAME;

/// Scaffold health string (kept for smoke).
pub fn health_message() -> String {
    format!("{PRODUCT_NAME} application layer ready")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_message_mentions_product() {
        assert!(health_message().contains("Visual Library"));
    }
}
