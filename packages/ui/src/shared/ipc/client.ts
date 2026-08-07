/**
 * Thin IPC wrapper — F1–F6 Tauri commands (catalog, factory, review, plans, coverage).
 */

export type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

export type AppPathsDto = {
  app_data_root: string;
  db_path: string;
  media_root: string;
  exports_dir: string;
  tmp_dir: string;
  logs_dir: string;
};

export type SettingsDto = {
  media_root: string;
};

export type ConnectorBudgetDto = {
  provider_id: string;
  unit_cost_cents: number;
  budget_limit_cents: number;
  spent_cents: number;
  available_budget_cents?: number | null;
  free_quota: number;
  free_used: number;
  free_remaining?: number | null;
  currency: string;
  period: string;
  is_free: boolean;
  can_afford_one: boolean;
};

export type ConnectorBudgetUpdate = {
  provider_id: string;
  unit_cost_cents?: number | null;
  budget_limit_cents?: number | null;
  free_quota?: number | null;
  period?: string | null;
  is_free?: boolean | null;
  reset_usage?: boolean | null;
};

export type IntegrationConfigDto = {
  script_ai_provider: string;
  default_image_provider: string;
  enabled_image_providers: string[];
  xai_api_key_set: boolean;
  openai_api_key_set: boolean;
  stability_api_key_set: boolean;
  xai_api_key_hint: string;
  openai_api_key_hint: string;
  stability_api_key_hint: string;
  omniroute_base_url: string;
  omniroute_api_key_set: boolean;
  omniroute_api_key_hint: string;
  omniroute_image_model: string;
  omniroute_chat_model: string;
  omniroute_prefer_free: boolean;
  /** System prompt for OmniRoute/Claude script→needs (editable). */
  needs_system_prompt: string;
  /** If true, remote image errors become stub tiles (discouraged). Default false. */
  allow_stub_fallback_on_image_error?: boolean;
  connector_budgets: ConnectorBudgetDto[];
};

export type IntegrationConfigUpdate = {
  script_ai_provider?: string | null;
  default_image_provider?: string | null;
  enabled_image_providers?: string[] | null;
  xai_api_key?: string | null;
  openai_api_key?: string | null;
  stability_api_key?: string | null;
  omniroute_base_url?: string | null;
  omniroute_api_key?: string | null;
  omniroute_image_model?: string | null;
  omniroute_chat_model?: string | null;
  omniroute_prefer_free?: boolean | null;
  needs_system_prompt?: string | null;
  allow_stub_fallback_on_image_error?: boolean | null;
  connector_budget_updates?: ConnectorBudgetUpdate[] | null;
};

export type ScriptAiProviderInfo = {
  id: string;
  name: string;
  description: string;
  status: string;
  status_detail: string;
};

export type ConceptDto = {
  id: string;
  key: string;
  name: string;
  description?: string | null;
  status: string;
};

export type ThemeDto = {
  id: string;
  name: string;
  description?: string | null;
  status: string;
};

export type RepresentationDto = {
  id: string;
  concept_id: string;
  key: string;
  name: string;
  orientation_default: string;
  status: string;
};

export type AssetDto = {
  id: string;
  concept_id: string;
  representation_id: string;
  status: string;
  storage_path: string;
  content_hash?: string | null;
  width?: number | null;
  height?: number | null;
  mime?: string | null;
  format?: string | null;
  orientation?: string | null;
  style?: string | null;
  provider?: string | null;
  prompt?: string | null;
  review_notes?: string | null;
  duplicate_of_asset_id?: string | null;
  approved_at?: string | null;
  rejected_at?: string | null;
  reject_reason?: string | null;
  created_at: string;
  /** FacelessStudio package handoff (auto write-back on approve). */
  package_id?: string | null;
  package_path?: string | null;
  beat_id?: string | null;
  package_concept_key?: string | null;
};

export type ApproveAssetResult = {
  asset: AssetDto;
  package_writeback?: WritePackageImagesResult | null;
  package_writeback_error?: string | null;
};

export type CoverageSummary = {
  concepts_total: number;
  concepts_under_covered: number;
  concepts_over_covered: number;
  concepts_missing_representations: number;
  waiting_review: number;
  approved_assets: number;
  draft_plans: number;
  approved_plans: number;
};

export type CoverageIssue = {
  code: string;
  severity: string;
  title: string;
  detail: string;
  cta_flow: string;
  related_id?: string | null;
};

export type CoverageReport = {
  summary: CoverageSummary;
  issues: CoverageIssue[];
};

export type GenerateStubResult = {
  job_id: string;
  job_status: string;
  asset_id: string;
  asset_status: string;
  storage_path: string;
};

async function getInvoke(): Promise<InvokeFn | null> {
  try {
    const mod = await import("@tauri-apps/api/core");
    return mod.invoke as InvokeFn;
  } catch {
    return null;
  }
}

async function invokeRequired<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await getInvoke();
  if (!invoke) {
    throw new Error("Tauri invoke unavailable");
  }
  return invoke<T>(cmd, args);
}

export async function invokeHealth(): Promise<string> {
  return invokeRequired<string>("health");
}

export async function invokeGetAppPaths(): Promise<AppPathsDto> {
  return invokeRequired<AppPathsDto>("get_app_paths");
}

export async function invokeGetSettings(): Promise<SettingsDto> {
  return invokeRequired<SettingsDto>("get_settings");
}

export async function invokeSetMediaRoot(mediaRoot: string): Promise<SettingsDto> {
  return invokeRequired<SettingsDto>("set_media_root", {
    args: { media_root: mediaRoot },
  });
}

export async function invokeGetIntegrationConfig(): Promise<IntegrationConfigDto> {
  return invokeRequired<IntegrationConfigDto>("get_integration_config_cmd");
}

export async function invokeUpdateIntegrationConfig(
  update: IntegrationConfigUpdate,
): Promise<IntegrationConfigDto> {
  return invokeRequired<IntegrationConfigDto>("update_integration_config_cmd", {
    args: update,
  });
}

export async function invokeListScriptAiProviders(): Promise<ScriptAiProviderInfo[]> {
  return invokeRequired<ScriptAiProviderInfo[]>("list_script_ai_providers_cmd");
}

export async function invokeListConcepts(): Promise<ConceptDto[]> {
  return invokeRequired<ConceptDto[]>("list_concepts_cmd");
}

export async function invokeEnsureConcept(
  key: string,
  name: string,
  description?: string,
): Promise<ConceptDto> {
  return invokeRequired<ConceptDto>("ensure_concept_cmd", {
    args: { key, name, description: description ?? null },
  });
}

export async function invokeListThemes(): Promise<ThemeDto[]> {
  return invokeRequired<ThemeDto[]>("list_themes_cmd");
}

export async function invokeEnsureRepresentation(
  conceptId: string,
  key: string,
  name: string,
  orientationDefault?: string,
): Promise<RepresentationDto> {
  return invokeRequired<RepresentationDto>("ensure_representation_cmd", {
    args: {
      concept_id: conceptId,
      key,
      name,
      orientation_default: orientationDefault ?? "any",
    },
  });
}

export async function invokeListRepresentations(
  conceptId: string,
): Promise<RepresentationDto[]> {
  return invokeRequired<RepresentationDto[]>("list_representations_cmd", {
    args: { concept_id: conceptId },
  });
}

export async function invokeGenerateStub(input: {
  conceptId: string;
  representationId: string;
  prompt?: string;
  idempotencyKey?: string;
}): Promise<GenerateStubResult> {
  return invokeRequired<GenerateStubResult>("generate_stub_asset_cmd", {
    args: {
      concept_id: input.conceptId,
      representation_id: input.representationId,
      prompt: input.prompt ?? null,
      idempotency_key: input.idempotencyKey ?? null,
    },
  });
}

export async function invokeListWaitingReview(): Promise<AssetDto[]> {
  return invokeRequired<AssetDto[]>("list_waiting_review_cmd");
}

export async function invokeListLibraryAssets(): Promise<AssetDto[]> {
  return invokeRequired<AssetDto[]>("list_library_assets_cmd");
}

export async function invokeApproveAsset(assetId: string): Promise<ApproveAssetResult> {
  return invokeRequired<ApproveAssetResult>("approve_asset_cmd", {
    args: { asset_id: assetId },
  });
}

export async function invokeRejectAsset(
  assetId: string,
  reason?: string,
): Promise<AssetDto> {
  return invokeRequired<AssetDto>("reject_asset_cmd", {
    args: { asset_id: assetId, reason: reason ?? null },
  });
}

export type ManualNeed = {
  concept_key: string;
  concept_name?: string | null;
  representation_key: string;
  representation_name?: string | null;
  prompt?: string | null;
  orientation?: string | null;
  style?: string | null;
  provider?: string | null;
  script_excerpt?: string | null;
  ai_instructions?: string | null;
  pedagogical_intent?: string | null;
  included?: boolean | null;
  /** 1–3 variants from same base prompt (default 3). */
  variant_count?: number | null;
  /** If FOUND in Library, still generate variants (asked at that moment). */
  also_generate_if_found?: boolean | null;
  /** Production package handoff (FacelessStudio). */
  package_id?: string | null;
  beat_id?: string | null;
  package_path?: string | null;
};

export type PackageSummary = {
  package_id: string;
  title: string;
  path: string;
  package_dir: string;
  beats: number;
  script_status: string;
  meta_status: string;
  smoke: boolean;
};

export type PackageDetail = {
  summary: PackageSummary;
  script_text: string;
  full_text: string;
  beats: Array<{
    beat_id: string;
    role: string;
    spoken_text: string;
    visual_intent: string;
    concept_key: string;
    representation_key: string;
    est_duration_sec: number;
  }>;
};

export type WritePackageImageItem = {
  beat_id: string;
  source_path: string;
  asset_id?: string | null;
  concept_key?: string | null;
};

export type WritePackageImagesResult = {
  package_id: string;
  package_path: string;
  written: Array<{ beat_id: string; dest_relative: string; asset_id?: string | null }>;
  image_count: number;
  notes: string;
};

export type ManualNeedResult = {
  index: number;
  decision: string;
  concept_id: string;
  concept_key: string;
  representation_id: string;
  representation_key: string;
  found_asset_id?: string | null;
  generate?: GenerateStubResult | null;
  generates?: GenerateStubResult[];
  variants_planned?: number;
  matiz_labels?: string[];
  message: string;
  selected_provider?: string | null;
};

export type ManualBatchPreview = {
  results: ManualNeedResult[];
  found_count: number;
  generate_count: number;
  skipped_count: number;
  /** Needs blocked because variants already wait in Review. */
  pending_review_count?: number;
  variant_images?: number;
};

export type ImageProvider = {
  id: string;
  name: string;
  description?: string;
  /** Legacy flag; prefer status / can_afford_one from integrations. */
  available?: boolean;
  enabled?: boolean;
  status?: string;
  status_detail?: string;
  cost_score: number;
  quality_score: number;
  availability_score: number;
  kind: string;
  notes?: string;
  can_afford_one?: boolean;
  is_free?: boolean;
};

export type ProposeNeedsResult = {
  needs: ManualNeed[];
  script_instructions: string;
  method: string;
  notes: string;
};

export async function invokeProposeNeedsFromScript(
  script: string,
  maxNeeds?: number,
  extraInstructions?: string | null,
): Promise<ProposeNeedsResult> {
  return invokeRequired<ProposeNeedsResult>("propose_needs_from_script_cmd", {
    args: {
      script,
      max_needs: maxNeeds ?? null,
      extra_instructions: extraInstructions?.trim() ? extraInstructions.trim() : null,
    },
  });
}

export async function invokeListPackages(packagesRoot?: string): Promise<PackageSummary[]> {
  return invokeRequired<PackageSummary[]>("list_packages_cmd", {
    args: packagesRoot ? { packages_root: packagesRoot } : {},
  });
}

export async function invokeLoadPackageDetail(packagePath: string): Promise<PackageDetail> {
  return invokeRequired<PackageDetail>("load_package_detail_cmd", {
    args: { package_path: packagePath, max_needs: null },
  });
}

export async function invokeProposeNeedsFromPackage(
  packagePath: string,
  maxNeeds?: number,
): Promise<ProposeNeedsResult> {
  return invokeRequired<ProposeNeedsResult>("propose_needs_from_package_cmd", {
    args: { package_path: packagePath, max_needs: maxNeeds ?? null },
  });
}

export async function invokeWritePackageImages(
  packagePath: string,
  items: WritePackageImageItem[],
): Promise<WritePackageImagesResult> {
  return invokeRequired<WritePackageImagesResult>("write_package_images_cmd", {
    args: { package_path: packagePath, items },
  });
}

export async function invokeListImageProviders(): Promise<ImageProvider[]> {
  return invokeRequired<ImageProvider[]>("list_image_providers_cmd");
}

export type OmniRouteProbeResult = {
  base_url: string;
  models_ok: boolean;
  models_detail: string;
  images_ok: boolean;
  images_detail: string;
  chat_ok: boolean;
  chat_detail: string;
  overall_ok: boolean;
  summary: string;
};

/** Probe local OmniRoute gateway (models + optional image/chat). */
export async function invokeProbeOmniroute(opts?: {
  tryImage?: boolean;
  tryChat?: boolean;
}): Promise<OmniRouteProbeResult> {
  return invokeRequired<OmniRouteProbeResult>("probe_omniroute_cmd", {
    args: {
      try_image: opts?.tryImage ?? true,
      try_chat: opts?.tryChat ?? true,
    },
  });
}

export type OmniRouteModelCatalog = {
  base_url: string;
  ok: boolean;
  detail: string;
  chat_models: string[];
  image_models: string[];
};

/** List chat/image model ids for Settings dropdowns. */
export async function invokeListOmnirouteModels(): Promise<OmniRouteModelCatalog> {
  return invokeRequired<OmniRouteModelCatalog>("list_omniroute_models_cmd");
}

export async function invokePreviewManualBatch(
  needs: ManualNeed[],
): Promise<ManualBatchPreview> {
  return invokeRequired<ManualBatchPreview>("preview_manual_batch_cmd", {
    args: { needs, batch_id: null },
  });
}

export async function invokeSubmitManualBatch(
  needs: ManualNeed[],
  batchId?: string,
): Promise<ManualBatchPreview> {
  return invokeRequired<ManualBatchPreview>("submit_manual_batch_cmd", {
    args: { needs, batch_id: batchId ?? null },
  });
}

export type PlanDto = {
  id: string;
  name: string;
  description?: string | null;
  status: string;
  approved_at?: string | null;
};

export type PlanItemDto = {
  id: string;
  plan_id: string;
  concept_key?: string | null;
  representation_key?: string | null;
  action: string;
  status: string;
  priority: number;
};

export type PlanWithItemsDto = {
  plan: PlanDto;
  items: PlanItemDto[];
};

export type AutomaticRunResult = {
  plan_id: string;
  plan_status: string;
  items_touched: number;
  batch: ManualBatchPreview;
};

export async function invokeListPlans(): Promise<PlanDto[]> {
  return invokeRequired<PlanDto[]>("list_plans_cmd");
}

export async function invokeCreatePlan(
  name: string,
  description?: string,
): Promise<PlanDto> {
  return invokeRequired<PlanDto>("create_plan_cmd", {
    args: { name, description: description ?? null },
  });
}

export async function invokeGetPlan(planId: string): Promise<PlanWithItemsDto> {
  return invokeRequired<PlanWithItemsDto>("get_plan_cmd", {
    args: { plan_id: planId },
  });
}

export async function invokeAddPlanItem(input: {
  planId: string;
  conceptKey: string;
  representationKey: string;
  orientation?: string;
  style?: string;
}): Promise<PlanItemDto> {
  return invokeRequired<PlanItemDto>("add_plan_item_cmd", {
    args: {
      plan_id: input.planId,
      concept_key: input.conceptKey,
      representation_key: input.representationKey,
      orientation: input.orientation ?? "any",
      style: input.style ?? "any",
    },
  });
}

export async function invokeApprovePlan(planId: string): Promise<PlanDto> {
  return invokeRequired<PlanDto>("approve_plan_cmd", {
    args: { plan_id: planId },
  });
}

export async function invokeRunAutomaticPlan(
  planId: string,
): Promise<AutomaticRunResult> {
  return invokeRequired<AutomaticRunResult>("run_automatic_plan_cmd", {
    args: { plan_id: planId },
  });
}

export async function invokeEditMetadata(input: {
  assetId: string;
  reviewNotes?: string;
  orientation?: string;
  style?: string;
  prompt?: string;
}): Promise<AssetDto> {
  return invokeRequired<AssetDto>("edit_asset_metadata_cmd", {
    args: {
      asset_id: input.assetId,
      review_notes: input.reviewNotes ?? null,
      orientation: input.orientation ?? null,
      style: input.style ?? null,
      prompt: input.prompt ?? null,
    },
  });
}

export async function invokeMarkDuplicate(
  assetId: string,
  ofAssetId: string,
): Promise<AssetDto> {
  return invokeRequired<AssetDto>("mark_duplicate_cmd", {
    args: { asset_id: assetId, of_asset_id: ofAssetId },
  });
}

export async function invokeRegenerateAsset(
  assetId: string,
): Promise<GenerateStubResult> {
  return invokeRequired<GenerateStubResult>("regenerate_asset_cmd", {
    args: { asset_id: assetId },
  });
}

export type AssetPreviewDto = {
  asset_id: string;
  mime: string;
  data_url: string;
  storage_path: string;
  width?: number | null;
  height?: number | null;
};

export async function invokeAssetPreview(assetId: string): Promise<AssetPreviewDto> {
  return invokeRequired<AssetPreviewDto>("get_asset_preview_cmd", {
    args: { asset_id: assetId },
  });
}

export async function invokeCoverageReport(): Promise<CoverageReport> {
  return invokeRequired<CoverageReport>("get_coverage_report_cmd");
}
