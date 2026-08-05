mod error_dto;

use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use tauri::{Manager, State};
use visual_library_application::{
    add_plan_item, approve_asset, approve_coverage_plan, create_plan, edit_asset_metadata,
    ensure_concept, ensure_representation, ensure_theme, generate_stub_asset, get_asset_preview,
    get_coverage_report, get_integration_config_dto, get_plan_with_items,
    get_settings as load_settings, list_concepts, list_image_providers_with_config,
    list_library_assets, list_omniroute_model_catalog, list_plans, list_representations,
    list_script_ai_providers, list_themes, list_waiting_review, load_integration_config,
    mark_asset_duplicate, media_writer_for, preview_manual_batch, probe_omniroute,
    propose_needs_with_config, regenerate_asset, reject_asset, run_automatic_from_plan,
    save_integration_config, submit_manual_batch, update_integration_config, update_media_root,
    validate_media_root, AppPathsDto, AssetDto, AssetPreviewDto, AutomaticRunResult, ConceptDto,
    CoverageReport, GenerateStubInput, GenerateStubResult, ImageProviderInfo, IntegrationConfigDto,
    IntegrationConfigUpdate, ManualBatchPreview, ManualNeed, OmniRouteModelCatalog,
    OmniRouteProbeResult, PlanDto, PlanItemDto, PlanWithItemsDto, ProposeNeedsResult,
    RepresentationDto, ScriptAiProviderInfo, SettingsDto, ThemeDto, PRODUCT_NAME,
};
use visual_library_infrastructure::{bootstrap, infrastructure_health, Platform};

use crate::error_dto::CommandError;

pub struct AppState {
    pub platform: Arc<Platform>,
}

fn store(state: &AppState) -> &visual_library_infrastructure::SqliteSettingsStore {
    state.platform.settings.as_ref()
}

fn media_root(state: &AppState) -> Result<PathBuf, CommandError> {
    let s = load_settings(store(state), &state.platform.layout.media_root)
        .map_err(CommandError::from)?;
    Ok(PathBuf::from(s.media_root))
}

fn integrations(
    state: &AppState,
) -> Result<visual_library_application::IntegrationConfig, CommandError> {
    load_integration_config(store(state)).map_err(CommandError::from)
}

#[tauri::command]
fn health(state: State<'_, AppState>) -> String {
    let db = state
        .platform
        .layout
        .db_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("db");
    format!("{PRODUCT_NAME} | {} | db={db}", infrastructure_health())
}

#[tauri::command]
fn get_app_paths(state: State<'_, AppState>) -> Result<AppPathsDto, CommandError> {
    let mut dto = state.platform.layout.to_dto();
    let settings = load_settings(store(&state), &state.platform.layout.media_root)
        .map_err(CommandError::from)?;
    dto.media_root = settings.media_root;
    Ok(dto)
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<SettingsDto, CommandError> {
    load_settings(store(&state), &state.platform.layout.media_root).map_err(CommandError::from)
}

// note: State deref to AppState

#[derive(Debug, Deserialize)]
struct SetMediaRootArgs {
    media_root: String,
}

#[tauri::command]
fn set_media_root(
    state: State<'_, AppState>,
    args: SetMediaRootArgs,
) -> Result<SettingsDto, CommandError> {
    let path = validate_media_root(&args.media_root).map_err(CommandError::from)?;
    std::fs::create_dir_all(&path).map_err(|e| {
        CommandError::from(visual_library_application::AppError::Storage(e.to_string()))
    })?;
    update_media_root(store(&state), path.to_string_lossy().as_ref()).map_err(CommandError::from)
}

#[tauri::command]
fn validate_media_root_cmd(path: String) -> Result<String, CommandError> {
    let p = validate_media_root(&path).map_err(CommandError::from)?;
    Ok(p.to_string_lossy().into_owned())
}

#[tauri::command]
fn list_themes_cmd(state: State<'_, AppState>) -> Result<Vec<ThemeDto>, CommandError> {
    list_themes(store(&state)).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct EnsureThemeArgs {
    name: String,
    description: Option<String>,
}

#[tauri::command]
fn ensure_theme_cmd(
    state: State<'_, AppState>,
    args: EnsureThemeArgs,
) -> Result<ThemeDto, CommandError> {
    ensure_theme(store(&state), &args.name, args.description.as_deref()).map_err(CommandError::from)
}

#[tauri::command]
fn list_concepts_cmd(state: State<'_, AppState>) -> Result<Vec<ConceptDto>, CommandError> {
    list_concepts(store(&state)).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct EnsureConceptArgs {
    key: String,
    name: String,
    description: Option<String>,
}

#[tauri::command]
fn ensure_concept_cmd(
    state: State<'_, AppState>,
    args: EnsureConceptArgs,
) -> Result<ConceptDto, CommandError> {
    ensure_concept(
        store(&state),
        &args.key,
        &args.name,
        args.description.as_deref(),
    )
    .map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct ListRepresentationsArgs {
    concept_id: String,
}

#[tauri::command]
fn list_representations_cmd(
    state: State<'_, AppState>,
    args: ListRepresentationsArgs,
) -> Result<Vec<RepresentationDto>, CommandError> {
    list_representations(store(&state), &args.concept_id).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct EnsureRepresentationArgs {
    concept_id: String,
    key: String,
    name: String,
    orientation_default: Option<String>,
}

#[tauri::command]
fn ensure_representation_cmd(
    state: State<'_, AppState>,
    args: EnsureRepresentationArgs,
) -> Result<RepresentationDto, CommandError> {
    ensure_representation(
        store(&state),
        &args.concept_id,
        &args.key,
        &args.name,
        args.orientation_default.as_deref(),
    )
    .map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct GenerateStubArgs {
    concept_id: String,
    representation_id: String,
    prompt: Option<String>,
    idempotency_key: Option<String>,
}

#[tauri::command]
fn generate_stub_asset_cmd(
    state: State<'_, AppState>,
    args: GenerateStubArgs,
) -> Result<GenerateStubResult, CommandError> {
    let root = media_root(&state)?;
    let writer = media_writer_for(&root);
    let mut cfg = integrations(&state)?;
    let res = generate_stub_asset(
        store(&state),
        store(&state),
        store(&state),
        &writer,
        GenerateStubInput {
            concept_id: args.concept_id,
            representation_id: args.representation_id,
            prompt: args.prompt,
            provider: Some(cfg.default_image_provider.clone()),
            orientation: None,
            style: None,
            idempotency_key: args.idempotency_key,
        },
        &mut cfg,
    )
    .map_err(CommandError::from)?;
    let _ = save_integration_config(store(&state), &cfg);
    Ok(res)
}

#[tauri::command]
fn list_waiting_review_cmd(state: State<'_, AppState>) -> Result<Vec<AssetDto>, CommandError> {
    list_waiting_review(store(&state)).map_err(CommandError::from)
}

#[tauri::command]
fn list_library_assets_cmd(state: State<'_, AppState>) -> Result<Vec<AssetDto>, CommandError> {
    list_library_assets(store(&state)).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct AssetIdArgs {
    asset_id: String,
}

#[tauri::command]
fn approve_asset_cmd(
    state: State<'_, AppState>,
    args: AssetIdArgs,
) -> Result<AssetDto, CommandError> {
    approve_asset(store(&state), &args.asset_id).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct RejectAssetArgs {
    asset_id: String,
    reason: Option<String>,
}

#[tauri::command]
fn reject_asset_cmd(
    state: State<'_, AppState>,
    args: RejectAssetArgs,
) -> Result<AssetDto, CommandError> {
    reject_asset(store(&state), &args.asset_id, args.reason.as_deref()).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct EditMetadataArgs {
    asset_id: String,
    review_notes: Option<String>,
    orientation: Option<String>,
    style: Option<String>,
    prompt: Option<String>,
}

#[tauri::command]
fn edit_asset_metadata_cmd(
    state: State<'_, AppState>,
    args: EditMetadataArgs,
) -> Result<AssetDto, CommandError> {
    edit_asset_metadata(
        store(&state),
        &args.asset_id,
        args.review_notes.as_deref(),
        args.orientation.as_deref(),
        args.style.as_deref(),
        args.prompt.as_deref(),
    )
    .map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct MarkDuplicateArgs {
    asset_id: String,
    of_asset_id: String,
}

#[tauri::command]
fn mark_duplicate_cmd(
    state: State<'_, AppState>,
    args: MarkDuplicateArgs,
) -> Result<AssetDto, CommandError> {
    mark_asset_duplicate(store(&state), &args.asset_id, &args.of_asset_id)
        .map_err(CommandError::from)
}

#[tauri::command]
fn regenerate_asset_cmd(
    state: State<'_, AppState>,
    args: AssetIdArgs,
) -> Result<GenerateStubResult, CommandError> {
    let root = media_root(&state)?;
    let writer = media_writer_for(&root);
    let mut cfg = integrations(&state)?;
    let res = regenerate_asset(
        store(&state),
        store(&state),
        store(&state),
        &writer,
        &args.asset_id,
        &mut cfg,
    )
    .map_err(CommandError::from)?;
    let _ = save_integration_config(store(&state), &cfg);
    Ok(res)
}

#[tauri::command]
fn get_asset_preview_cmd(
    state: State<'_, AppState>,
    args: AssetIdArgs,
) -> Result<AssetPreviewDto, CommandError> {
    let root = media_root(&state)?;
    get_asset_preview(store(&state), &root, &args.asset_id).map_err(CommandError::from)
}

#[tauri::command]
fn get_coverage_report_cmd(state: State<'_, AppState>) -> Result<CoverageReport, CommandError> {
    get_coverage_report(store(&state), store(&state), store(&state)).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct ManualBatchArgs {
    needs: Vec<ManualNeed>,
    batch_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProposeNeedsArgs {
    script: String,
    max_needs: Option<usize>,
    /// Optional brief from Factory “Instrucciones” merged into the chat user message.
    extra_instructions: Option<String>,
}

#[tauri::command]
fn propose_needs_from_script_cmd(
    state: State<'_, AppState>,
    args: ProposeNeedsArgs,
) -> Result<ProposeNeedsResult, CommandError> {
    let mut cfg = integrations(&state)?;
    let res = propose_needs_with_config(
        &args.script,
        args.max_needs,
        &cfg,
        args.extra_instructions.as_deref(),
    )
    .map_err(CommandError::from)?;
    // Persist usage if OmniRoute chat billed free units (best-effort; cfg may be unchanged).
    if res.method.starts_with("omniroute_chat") {
        let _ = visual_library_application::record_usage(&mut cfg, "omniroute", 1);
        let _ = save_integration_config(store(&state), &cfg);
    }
    Ok(res)
}

#[tauri::command]
fn list_image_providers_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<ImageProviderInfo>, CommandError> {
    let cfg = integrations(&state)?;
    Ok(list_image_providers_with_config(&cfg))
}

#[tauri::command]
fn list_script_ai_providers_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<ScriptAiProviderInfo>, CommandError> {
    let cfg = integrations(&state)?;
    Ok(list_script_ai_providers(&cfg))
}

#[tauri::command]
fn get_integration_config_cmd(
    state: State<'_, AppState>,
) -> Result<IntegrationConfigDto, CommandError> {
    get_integration_config_dto(store(&state)).map_err(CommandError::from)
}

#[tauri::command]
fn update_integration_config_cmd(
    state: State<'_, AppState>,
    args: IntegrationConfigUpdate,
) -> Result<IntegrationConfigDto, CommandError> {
    update_integration_config(store(&state), args).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct ProbeOmniRouteArgs {
    /// If true, runs a tiny image generation (may consume free quota).
    try_image: Option<bool>,
    /// If true, runs a tiny chat completion.
    try_chat: Option<bool>,
}

#[tauri::command]
fn probe_omniroute_cmd(
    state: State<'_, AppState>,
    args: Option<ProbeOmniRouteArgs>,
) -> Result<OmniRouteProbeResult, CommandError> {
    let cfg = integrations(&state)?;
    let try_image = args.as_ref().and_then(|a| a.try_image).unwrap_or(true);
    let try_chat = args.as_ref().and_then(|a| a.try_chat).unwrap_or(true);
    Ok(probe_omniroute(&cfg, try_image, try_chat))
}

#[tauri::command]
fn list_omniroute_models_cmd(
    state: State<'_, AppState>,
) -> Result<OmniRouteModelCatalog, CommandError> {
    let cfg = integrations(&state)?;
    Ok(list_omniroute_model_catalog(&cfg))
}

#[tauri::command]
fn preview_manual_batch_cmd(
    state: State<'_, AppState>,
    args: ManualBatchArgs,
) -> Result<ManualBatchPreview, CommandError> {
    let cfg = integrations(&state)?;
    preview_manual_batch(store(&state), store(&state), &args.needs, &cfg)
        .map_err(CommandError::from)
}

#[tauri::command]
fn submit_manual_batch_cmd(
    state: State<'_, AppState>,
    args: ManualBatchArgs,
) -> Result<ManualBatchPreview, CommandError> {
    let root = media_root(&state)?;
    let writer = media_writer_for(&root);
    let mut cfg = integrations(&state)?;
    let res = submit_manual_batch(
        store(&state),
        store(&state),
        store(&state),
        &writer,
        &args.needs,
        args.batch_id.as_deref(),
        &mut cfg,
    )
    .map_err(CommandError::from)?;
    let _ = save_integration_config(store(&state), &cfg);
    Ok(res)
}

#[derive(Debug, Deserialize)]
struct CreatePlanArgs {
    name: String,
    description: Option<String>,
}

#[tauri::command]
fn create_plan_cmd(
    state: State<'_, AppState>,
    args: CreatePlanArgs,
) -> Result<PlanDto, CommandError> {
    create_plan(store(&state), &args.name, args.description.as_deref(), None)
        .map_err(CommandError::from)
}

#[tauri::command]
fn list_plans_cmd(state: State<'_, AppState>) -> Result<Vec<PlanDto>, CommandError> {
    list_plans(store(&state)).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct PlanIdArgs {
    plan_id: String,
}

#[tauri::command]
fn get_plan_cmd(
    state: State<'_, AppState>,
    args: PlanIdArgs,
) -> Result<PlanWithItemsDto, CommandError> {
    get_plan_with_items(store(&state), &args.plan_id).map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
struct AddPlanItemArgs {
    plan_id: String,
    concept_key: String,
    representation_key: String,
    orientation: Option<String>,
    style: Option<String>,
}

#[tauri::command]
fn add_plan_item_cmd(
    state: State<'_, AppState>,
    args: AddPlanItemArgs,
) -> Result<PlanItemDto, CommandError> {
    add_plan_item(
        store(&state),
        &args.plan_id,
        &args.concept_key,
        &args.representation_key,
        None,
        None,
        args.orientation.as_deref(),
        args.style.as_deref(),
    )
    .map_err(CommandError::from)
}

#[tauri::command]
fn approve_plan_cmd(state: State<'_, AppState>, args: PlanIdArgs) -> Result<PlanDto, CommandError> {
    approve_coverage_plan(store(&state), &args.plan_id).map_err(CommandError::from)
}

#[tauri::command]
fn run_automatic_plan_cmd(
    state: State<'_, AppState>,
    args: PlanIdArgs,
) -> Result<AutomaticRunResult, CommandError> {
    let root = media_root(&state)?;
    let writer = media_writer_for(&root);
    run_automatic_from_plan(
        store(&state),
        store(&state),
        store(&state),
        store(&state),
        &writer,
        &args.plan_id,
    )
    .map_err(CommandError::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./.visual-library-data"));
            let platform = bootstrap(app_data).map_err(|e| e.to_string())?;
            app.manage(AppState {
                platform: Arc::new(platform),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            get_app_paths,
            get_settings,
            set_media_root,
            validate_media_root_cmd,
            get_integration_config_cmd,
            update_integration_config_cmd,
            probe_omniroute_cmd,
            list_omniroute_models_cmd,
            list_script_ai_providers_cmd,
            list_themes_cmd,
            ensure_theme_cmd,
            list_concepts_cmd,
            ensure_concept_cmd,
            list_representations_cmd,
            ensure_representation_cmd,
            generate_stub_asset_cmd,
            list_waiting_review_cmd,
            list_library_assets_cmd,
            approve_asset_cmd,
            reject_asset_cmd,
            edit_asset_metadata_cmd,
            mark_duplicate_cmd,
            regenerate_asset_cmd,
            get_asset_preview_cmd,
            get_coverage_report_cmd,
            propose_needs_from_script_cmd,
            list_image_providers_cmd,
            preview_manual_batch_cmd,
            submit_manual_batch_cmd,
            create_plan_cmd,
            list_plans_cmd,
            get_plan_cmd,
            add_plan_item_cmd,
            approve_plan_cmd,
            run_automatic_plan_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running Visual Library");
}
