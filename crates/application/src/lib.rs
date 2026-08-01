//! Application layer: use cases and ports.
//!
//! F1–F5 prior · F6 Coverage + Review completo.

pub mod assets;
pub mod catalog;
pub mod coverage;
pub mod error;
pub mod factory;
pub mod integrations;
pub mod jobs;
pub mod plans;
pub mod ports;
pub mod settings;

pub use assets::{
    approve_asset, edit_asset_metadata, get_asset_preview, list_library_assets, list_waiting_review,
    mark_asset_duplicate, regenerate_asset, reject_asset, AssetDto, AssetPreviewDto,
};
pub use catalog::{
    ensure_concept, ensure_representation, ensure_theme, list_concepts, list_representations,
    list_themes, ConceptDto, RepresentationDto, ThemeDto,
};
pub use coverage::{get_coverage_report, CoverageIssue, CoverageReport, CoverageSummary};
pub use error::AppError;
pub use factory::{
    list_image_providers, preview_manual_batch, propose_needs_from_script,
    run_automatic_from_plan, select_image_provider, submit_manual_batch, AutomaticRunResult,
    ImageProvider, ManualBatchPreview, ManualNeed, ManualNeedResult, ProposeNeedsInput,
    ProposeNeedsResult,
};
pub use integrations::{
    generate_image_bytes, get_integration_config_dto, list_connector_budgets,
    list_image_providers_with_config, list_script_ai_providers, load_integration_config,
    propose_needs_with_config, record_usage, save_integration_config,
    select_image_provider_with_config, update_connector_budget, update_integration_config,
    ConnectorBudgetDto, ConnectorBudgetUpdate, ConnectorLedger, GeneratedImage, ImageProviderInfo,
    IntegrationConfig, IntegrationConfigDto, IntegrationConfigUpdate, ScriptAiProviderInfo,
};
pub use jobs::{
    colored_stub_bmp, generate_stub_asset, media_writer_for, resolve_under_media_root,
    FsMediaWriter, GenerateStubInput, GenerateStubResult, JobDto, MediaWriter, STUB_PNG,
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
