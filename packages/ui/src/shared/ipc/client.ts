/**
 * Thin IPC wrapper — Foundation 1–3 commands.
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

export async function invokeApproveAsset(assetId: string): Promise<AssetDto> {
  return invokeRequired<AssetDto>("approve_asset_cmd", {
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
  message: string;
};

export type ManualBatchPreview = {
  results: ManualNeedResult[];
  found_count: number;
  generate_count: number;
  skipped_count: number;
};

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

export async function invokeCoverageReport(): Promise<CoverageReport> {
  return invokeRequired<CoverageReport>("get_coverage_report_cmd");
}
