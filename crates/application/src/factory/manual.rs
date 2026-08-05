use serde::{Deserialize, Serialize};
use visual_library_domain::{decide_acquisition, field_matches, AcquisitionDecision};

use crate::catalog::{ensure_concept, ensure_representation};
use crate::error::AppError;
use crate::factory::variants::{
    apply_matiz_to_prompt, clamp_variant_count, matiz_specs, with_no_text_guard,
};
use crate::integrations::{select_image_provider_with_config, IntegrationConfig};
use crate::jobs::{generate_stub_asset, GenerateStubInput, GenerateStubResult, MediaWriter};
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;

/// One structured visual need — DB-aligned requirement (+ variants).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualNeed {
    pub concept_key: String,
    pub concept_name: Option<String>,
    pub representation_key: String,
    pub representation_name: Option<String>,
    /// Base prompt (template + human edits). Variants append matiz suffixes.
    pub prompt: Option<String>,
    pub orientation: Option<String>,
    pub style: Option<String>,
    /// Preferred image provider id; re-selected if unavailable.
    pub provider: Option<String>,
    /// Excerpt of the script this need illustrates.
    pub script_excerpt: Option<String>,
    /// AI/heuristic instructions for this segment (how to teach/visualize).
    pub ai_instructions: Option<String>,
    pub pedagogical_intent: Option<String>,
    /// Human include flag after proposal (default true).
    pub included: Option<bool>,
    /// How many image variants to produce from same base prompt (1–3, default 3).
    pub variant_count: Option<u8>,
    /// If Library FOUND: also generate variants to enrich channel (asked at that moment).
    pub also_generate_if_found: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualNeedResult {
    pub index: usize,
    pub decision: String,
    pub concept_id: String,
    pub concept_key: String,
    pub representation_id: String,
    pub representation_key: String,
    pub found_asset_id: Option<String>,
    /// First generated asset (compat).
    pub generate: Option<GenerateStubResult>,
    /// All generated variants for this need.
    pub generates: Vec<GenerateStubResult>,
    pub variants_planned: usize,
    pub matiz_labels: Vec<String>,
    pub message: String,
    pub selected_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualBatchPreview {
    pub results: Vec<ManualNeedResult>,
    pub found_count: usize,
    pub generate_count: usize,
    pub skipped_count: usize,
    /// Assets already waiting review (blocked re-generate).
    pub pending_review_count: usize,
    /// Total variant images planned/produced across needs.
    pub variant_images: usize,
}

fn orient(n: &ManualNeed) -> String {
    n.orientation.as_deref().unwrap_or("any").trim().to_string()
}

fn style(n: &ManualNeed) -> String {
    n.style.as_deref().unwrap_or("any").trim().to_string()
}

fn empty_result(
    index: usize,
    ckey: &str,
    rkey: &str,
    decision: &str,
    message: &str,
) -> ManualNeedResult {
    ManualNeedResult {
        index,
        decision: decision.into(),
        concept_id: String::new(),
        concept_key: ckey.into(),
        representation_id: String::new(),
        representation_key: rkey.into(),
        found_asset_id: None,
        generate: None,
        generates: vec![],
        variants_planned: 0,
        matiz_labels: vec![],
        message: message.into(),
        selected_provider: None,
    }
}

fn resolve_need(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    index: usize,
    need: &ManualNeed,
    cfg: &IntegrationConfig,
) -> Result<ManualNeedResult, AppError> {
    let ckey = need.concept_key.trim();
    let rkey = need.representation_key.trim();
    let vcount = clamp_variant_count(need.variant_count);

    if need.included == Some(false) {
        return Ok(empty_result(
            index,
            ckey,
            rkey,
            AcquisitionDecision::Skipped.as_str(),
            "excluido por el usuario (included=false)",
        ));
    }

    if ckey.is_empty() || rkey.is_empty() {
        return Ok(empty_result(
            index,
            ckey,
            rkey,
            AcquisitionDecision::Skipped.as_str(),
            "concept_key y representation_key son obligatorios",
        ));
    }

    let concept = ensure_concept(
        catalog,
        ckey,
        need.concept_name.as_deref().unwrap_or(ckey),
        None,
    )?;
    let rep = ensure_representation(
        catalog,
        &concept.id,
        rkey,
        need.representation_name.as_deref().unwrap_or(rkey),
        Some(&orient(need)),
    )?;

    let o = orient(need);
    let s = style(need);
    let candidate = assets.find_approved_match(&rep.id, &o, &s)?;

    let sufficient = candidate.as_ref().is_some_and(|a| {
        field_matches(&o, a.orientation.as_deref()) && field_matches(&s, a.style.as_deref())
    });

    // Waiting review: do not flood with more of the same need.
    if let Some(waiting) = find_waiting_match(assets, &rep.id, &o, &s)? {
        // Even FOUND+enrich is blocked if variants already wait in Review.
        return Ok(ManualNeedResult {
            index,
            decision: "pending_review".into(),
            concept_id: concept.id,
            concept_key: concept.key,
            representation_id: rep.id,
            representation_key: rep.key,
            found_asset_id: Some(waiting.id.clone()),
            generate: None,
            generates: vec![],
            variants_planned: 0,
            matiz_labels: vec![],
            message: format!(
                "PENDING_REVIEW asset {} — ya hay variantes en cola (no re-generate)",
                waiting.id
            ),
            selected_provider: None,
        });
    }

    let labels: Vec<String> = matiz_specs(vcount)
        .iter()
        .map(|(l, _)| (*l).to_string())
        .collect();

    let decision = decide_acquisition(sufficient);
    match decision {
        AcquisitionDecision::Found => {
            let id = candidate.map(|a| a.id).unwrap_or_default();
            let enrich = need.also_generate_if_found == Some(true);
            if enrich {
                let provider = select_image_provider_with_config(need.provider.as_deref(), cfg)?;
                Ok(ManualNeedResult {
                    index,
                    decision: "found_enrich".into(),
                    concept_id: concept.id,
                    concept_key: concept.key,
                    representation_id: rep.id,
                    representation_key: rep.key,
                    found_asset_id: Some(id.clone()),
                    generate: None,
                    generates: vec![],
                    variants_planned: vcount,
                    matiz_labels: labels,
                    message: format!(
                        "FOUND {id} + enriquecer con {vcount} variante(s) (provider `{}`)",
                        provider.id
                    ),
                    selected_provider: Some(provider.id),
                })
            } else {
                Ok(ManualNeedResult {
                    index,
                    decision: "found".into(),
                    concept_id: concept.id,
                    concept_key: concept.key,
                    representation_id: rep.id,
                    representation_key: rep.key,
                    found_asset_id: Some(id.clone()),
                    generate: None,
                    generates: vec![],
                    variants_planned: 0,
                    matiz_labels: vec![],
                    message: format!(
                        "FOUND asset {id} (Library). Marca «también generar variantes» si quieres enriquecer."
                    ),
                    selected_provider: None,
                })
            }
        }
        AcquisitionDecision::Generate => {
            let provider = select_image_provider_with_config(need.provider.as_deref(), cfg)?;
            Ok(ManualNeedResult {
                index,
                decision: "generate".into(),
                concept_id: concept.id,
                concept_key: concept.key,
                representation_id: rep.id,
                representation_key: rep.key,
                found_asset_id: None,
                generate: None,
                generates: vec![],
                variants_planned: vcount,
                matiz_labels: labels,
                message: format!(
                    "GENERATE {vcount} variante(s) via `{}` (matices literal/metáfora + estilo)",
                    provider.id
                ),
                selected_provider: Some(provider.id),
            })
        }
        AcquisitionDecision::Skipped => Ok(empty_result(
            index,
            &concept.key,
            &rep.key,
            "skipped",
            "SKIPPED",
        )),
    }
}

fn find_waiting_match(
    assets: &impl AssetStore,
    representation_id: &str,
    orientation: &str,
    style: &str,
) -> Result<Option<crate::assets::AssetDto>, AppError> {
    let waiting = assets.list_by_status("waiting_review")?;
    Ok(waiting.into_iter().find(|a| {
        a.representation_id == representation_id
            && field_matches(orientation, a.orientation.as_deref())
            && field_matches(style, a.style.as_deref())
    }))
}

/// Preview only: resolve FOUND vs GENERATE without writing new assets.
pub fn preview_manual_batch(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    needs: &[ManualNeed],
    cfg: &IntegrationConfig,
) -> Result<ManualBatchPreview, AppError> {
    if needs.is_empty() {
        return Err(AppError::Validation("lista de necesidades vacía".into()));
    }
    let mut results = Vec::with_capacity(needs.len());
    let mut found_count = 0usize;
    let mut generate_count = 0usize;
    let mut skipped_count = 0usize;
    let mut pending_review_count = 0usize;
    let mut variant_images = 0usize;

    for (i, need) in needs.iter().enumerate() {
        let r = resolve_need(catalog, assets, i, need, cfg)?;
        match r.decision.as_str() {
            "found" => found_count += 1,
            "found_enrich" => {
                found_count += 1;
                generate_count += 1;
                variant_images += r.variants_planned;
            }
            "generate" => {
                generate_count += 1;
                variant_images += r.variants_planned;
            }
            "pending_review" => pending_review_count += 1,
            _ => skipped_count += 1,
        }
        results.push(r);
    }

    Ok(ManualBatchPreview {
        results,
        found_count,
        generate_count,
        skipped_count,
        pending_review_count,
        variant_images,
    })
}

fn should_generate(decision: &str) -> bool {
    matches!(decision, "generate" | "found_enrich")
}

/// Submit: GENERATE / found_enrich create variant stubs → waiting_review.
pub fn submit_manual_batch(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    needs: &[ManualNeed],
    batch_id: Option<&str>,
    cfg: &mut IntegrationConfig,
) -> Result<ManualBatchPreview, AppError> {
    let mut preview = preview_manual_batch(catalog, assets, needs, cfg)?;
    let batch = batch_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("batch_{}", now_secs()));

    let mut variant_images = 0usize;

    for r in preview.results.iter_mut() {
        if !should_generate(&r.decision) {
            continue;
        }
        let need = &needs[r.index];
        let provider = select_image_provider_with_config(need.provider.as_deref(), cfg)?;
        r.selected_provider = Some(provider.id.clone());
        let vcount = r.variants_planned.max(1);
        let specs = matiz_specs(vcount);
        let mut base = need.prompt.clone().unwrap_or_else(|| {
            format!(
                "Educational illustration for concept {} / {}",
                r.concept_key, r.representation_key
            )
        });
        // Fold AI instructions into generation when present (so UI edits matter).
        if let Some(ai) = need
            .ai_instructions
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            base = format!("{base}\n\nSegment instructions:\n{ai}");
        }

        let mut gens = Vec::with_capacity(specs.len());
        let mut labels = Vec::with_capacity(specs.len());
        for (vi, (label, suffix)) in specs.iter().enumerate() {
            let prompt =
                with_no_text_guard(&apply_matiz_to_prompt(&base, suffix, vi + 1, specs.len()));
            let gen = generate_stub_asset(
                catalog,
                assets,
                jobs,
                media,
                GenerateStubInput {
                    concept_id: r.concept_id.clone(),
                    representation_id: r.representation_id.clone(),
                    prompt: Some(prompt),
                    provider: Some(provider.id.clone()),
                    orientation: need.orientation.clone(),
                    style: need.style.clone(),
                    idempotency_key: Some(format!(
                        "manual:{}:{}:{}:v{}",
                        batch,
                        r.concept_key,
                        r.representation_key,
                        vi + 1
                    )),
                },
                cfg,
            )
            .map_err(|e| {
                AppError::Validation(format!(
                    "Need #{} {}/{} variante {}/{} (provider `{}`): {e}",
                    r.index + 1,
                    r.concept_key,
                    r.representation_key,
                    vi + 1,
                    specs.len(),
                    provider.id
                ))
            })?;
            labels.push((*label).to_string());
            gens.push(gen);
        }
        variant_images += gens.len();
        r.generate = gens.first().cloned();
        r.generates = gens;
        r.matiz_labels = labels;
        r.message = format!(
            "{} → {} variante(s) waiting_review via `{}`",
            r.decision.to_uppercase(),
            r.generates.len(),
            provider.id
        );
    }

    preview.variant_images = variant_images;
    Ok(preview)
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetDto;
    use crate::catalog::{ConceptDto, RepresentationDto, ThemeDto};
    use crate::jobs::JobDto;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Mem {
        concepts: Mutex<Vec<ConceptDto>>,
        reps: Mutex<Vec<RepresentationDto>>,
        assets: Mutex<Vec<AssetDto>>,
        jobs: Mutex<Vec<JobDto>>,
    }

    impl CatalogStore for Mem {
        fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
            Ok(vec![])
        }
        fn ensure_theme(&self, name: &str, _d: Option<&str>) -> Result<ThemeDto, AppError> {
            Ok(ThemeDto {
                id: "t".into(),
                name: name.into(),
                description: None,
                status: "active".into(),
            })
        }
        fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError> {
            Ok(self.concepts.lock().unwrap().clone())
        }
        fn ensure_concept(
            &self,
            key: &str,
            name: &str,
            _d: Option<&str>,
        ) -> Result<ConceptDto, AppError> {
            let mut g = self.concepts.lock().unwrap();
            if let Some(c) = g.iter().find(|c| c.key == key) {
                return Ok(c.clone());
            }
            let c = ConceptDto {
                id: format!("c_{}", key),
                key: key.into(),
                name: name.into(),
                description: None,
                status: "active".into(),
            };
            g.push(c.clone());
            Ok(c)
        }
        fn list_representations(
            &self,
            concept_id: &str,
        ) -> Result<Vec<RepresentationDto>, AppError> {
            Ok(self
                .reps
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.concept_id == concept_id)
                .cloned()
                .collect())
        }
        fn ensure_representation(
            &self,
            concept_id: &str,
            key: &str,
            name: &str,
            orientation_default: &str,
        ) -> Result<RepresentationDto, AppError> {
            let mut g = self.reps.lock().unwrap();
            if let Some(r) = g
                .iter()
                .find(|r| r.concept_id == concept_id && r.key == key)
            {
                return Ok(r.clone());
            }
            let r = RepresentationDto {
                id: format!("r_{concept_id}_{key}"),
                concept_id: concept_id.into(),
                key: key.into(),
                name: name.into(),
                orientation_default: orientation_default.into(),
                status: "active".into(),
            };
            g.push(r.clone());
            Ok(r)
        }
    }

    impl AssetStore for Mem {
        fn insert(&self, asset: &AssetDto) -> Result<(), AppError> {
            self.assets.lock().unwrap().push(asset.clone());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<AssetDto>, AppError> {
            Ok(self
                .assets
                .lock()
                .unwrap()
                .iter()
                .find(|a| a.id == id)
                .cloned())
        }
        fn list_by_status(&self, status: &str) -> Result<Vec<AssetDto>, AppError> {
            Ok(self
                .assets
                .lock()
                .unwrap()
                .iter()
                .filter(|a| a.status == status)
                .cloned()
                .collect())
        }
        fn update_status(
            &self,
            _id: &str,
            _status: &str,
            _a: Option<&str>,
            _r: Option<&str>,
            _rr: Option<&str>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        fn find_approved_match(
            &self,
            representation_id: &str,
            orientation: &str,
            style: &str,
        ) -> Result<Option<AssetDto>, AppError> {
            Ok(self
                .assets
                .lock()
                .unwrap()
                .iter()
                .filter(|a| {
                    a.status == "approved"
                        && a.representation_id == representation_id
                        && field_matches(orientation, a.orientation.as_deref())
                        && field_matches(style, a.style.as_deref())
                })
                .cloned()
                .next())
        }
        fn update_metadata(
            &self,
            _id: &str,
            _: Option<&str>,
            _: Option<&str>,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<(), AppError> {
            Ok(())
        }
        fn set_duplicate_of(&self, _: &str, _: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    impl JobStore for Mem {
        fn insert(&self, job: &JobDto) -> Result<(), AppError> {
            self.jobs.lock().unwrap().push(job.clone());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<JobDto>, AppError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| j.id == id)
                .cloned())
        }
        fn get_by_idempotency_key(&self, key: &str) -> Result<Option<JobDto>, AppError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| j.idempotency_key.as_deref() == Some(key))
                .cloned())
        }
        fn update(
            &self,
            id: &str,
            status: &str,
            attempts: i64,
            _e: Option<&str>,
            outputs_json: Option<&str>,
            started_at: Option<&str>,
            finished_at: Option<&str>,
        ) -> Result<(), AppError> {
            let mut g = self.jobs.lock().unwrap();
            let j = g.iter_mut().find(|j| j.id == id).unwrap();
            j.status = status.into();
            j.attempts = attempts;
            if let Some(o) = outputs_json {
                j.outputs_json = Some(o.into());
            }
            if let Some(s) = started_at {
                j.started_at = Some(s.into());
            }
            if let Some(f) = finished_at {
                j.finished_at = Some(f.into());
            }
            Ok(())
        }
    }

    impl MediaWriter for Mem {
        fn write_asset_file(
            &self,
            relative_path: &str,
            _bytes: &[u8],
        ) -> Result<PathBuf, AppError> {
            Ok(PathBuf::from(relative_path))
        }
    }

    fn sample_need(concept: &str, rep: &str, variants: u8) -> ManualNeed {
        ManualNeed {
            concept_key: concept.into(),
            concept_name: Some(concept.into()),
            representation_key: rep.into(),
            representation_name: Some(rep.into()),
            prompt: Some("educational illustration base".into()),
            orientation: Some("any".into()),
            style: Some("didactic".into()),
            provider: Some("stub".into()),
            script_excerpt: Some("excerpt".into()),
            ai_instructions: Some("Enseñar con claridad visual".into()),
            pedagogical_intent: Some("clarify".into()),
            included: Some(true),
            variant_count: Some(variants),
            also_generate_if_found: Some(false),
        }
    }

    #[test]
    fn preview_generate_when_empty() {
        let m = Mem::default();
        let cfg = IntegrationConfig::default();
        let p = preview_manual_batch(&m, &m, &[sample_need("tree", "hero", 3)], &cfg).unwrap();
        assert_eq!(p.generate_count, 1);
        assert_eq!(p.results[0].variants_planned, 3);
        assert_eq!(p.variant_images, 3);
    }

    #[test]
    fn submit_creates_three_variants() {
        let m = Mem::default();
        let mut cfg = IntegrationConfig::default();
        let out = submit_manual_batch(
            &m,
            &m,
            &m,
            &m,
            &[sample_need("rock", "detail", 3)],
            Some("b1"),
            &mut cfg,
        )
        .unwrap();
        // free stub usage recorded
        let stub = cfg
            .connector_ledgers
            .iter()
            .find(|l| l.provider_id == "stub")
            .unwrap();
        assert!(stub.free_used >= 3 || stub.spent_cents == 0);
        assert_eq!(out.results[0].generates.len(), 3);
        assert_eq!(m.assets.lock().unwrap().len(), 3);
        assert!(m
            .assets
            .lock()
            .unwrap()
            .iter()
            .all(|a| a.status == "waiting_review"));
    }

    #[test]
    fn found_without_enrich_does_not_generate() {
        let m = Mem::default();
        let c = ensure_concept(&m, "tree", "Tree", None).unwrap();
        let r = ensure_representation(&m, &c.id, "hero", "Hero", Some("landscape")).unwrap();
        m.assets.lock().unwrap().push(AssetDto {
            id: "a1".into(),
            concept_id: c.id,
            representation_id: r.id,
            status: "approved".into(),
            storage_path: "x.bmp".into(),
            content_hash: None,
            width: None,
            height: None,
            mime: None,
            format: None,
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: None,
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: Some("t".into()),
            rejected_at: None,
            created_at: "t".into(),
            updated_at: "t".into(),
        });
        let mut need = sample_need("tree", "hero", 3);
        need.orientation = Some("landscape".into());
        need.also_generate_if_found = Some(false);
        let cfg = IntegrationConfig::default();
        let p = preview_manual_batch(&m, &m, &[need], &cfg).unwrap();
        assert_eq!(p.found_count, 1);
        assert_eq!(p.generate_count, 0);
        assert_eq!(p.results[0].decision, "found");
    }

    #[test]
    fn found_enrich_generates_variants() {
        let m = Mem::default();
        let c = ensure_concept(&m, "tree", "Tree", None).unwrap();
        let r = ensure_representation(&m, &c.id, "hero", "Hero", Some("landscape")).unwrap();
        m.assets.lock().unwrap().push(AssetDto {
            id: "a1".into(),
            concept_id: c.id,
            representation_id: r.id,
            status: "approved".into(),
            storage_path: "x.bmp".into(),
            content_hash: None,
            width: None,
            height: None,
            mime: None,
            format: None,
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: None,
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: Some("t".into()),
            rejected_at: None,
            created_at: "t".into(),
            updated_at: "t".into(),
        });
        let mut need = sample_need("tree", "hero", 2);
        need.orientation = Some("landscape".into());
        need.also_generate_if_found = Some(true);
        let mut cfg = IntegrationConfig::default();
        let out = submit_manual_batch(&m, &m, &m, &m, &[need], Some("enr"), &mut cfg).unwrap();
        assert_eq!(out.results[0].decision, "found_enrich");
        assert_eq!(out.results[0].generates.len(), 2);
        // 1 approved + 2 waiting
        assert_eq!(m.assets.lock().unwrap().len(), 3);
    }

    #[test]
    fn excluded_need_is_skipped() {
        let m = Mem::default();
        let mut need = sample_need("x", "y", 1);
        need.included = Some(false);
        let cfg = IntegrationConfig::default();
        let p = preview_manual_batch(&m, &m, &[need], &cfg).unwrap();
        assert_eq!(p.skipped_count, 1);
    }

    #[test]
    fn pending_review_blocks_regenerate_and_enrich() {
        let m = Mem::default();
        let c = ensure_concept(&m, "tree", "Tree", None).unwrap();
        let r = ensure_representation(&m, &c.id, "hero", "Hero", Some("landscape")).unwrap();
        // Already waiting variants for this need.
        m.assets.lock().unwrap().push(AssetDto {
            id: "wait1".into(),
            concept_id: c.id.clone(),
            representation_id: r.id.clone(),
            status: "waiting_review".into(),
            storage_path: "w.bmp".into(),
            content_hash: None,
            width: None,
            height: None,
            mime: None,
            format: None,
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: Some("stub".into()),
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: None,
            rejected_at: None,
            created_at: "t".into(),
            updated_at: "t".into(),
        });
        // Also an approved FOUND candidate — enrich must still be blocked.
        m.assets.lock().unwrap().push(AssetDto {
            id: "lib1".into(),
            concept_id: c.id,
            representation_id: r.id,
            status: "approved".into(),
            storage_path: "a.bmp".into(),
            content_hash: None,
            width: None,
            height: None,
            mime: None,
            format: None,
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: None,
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: Some("t".into()),
            rejected_at: None,
            created_at: "t".into(),
            updated_at: "t".into(),
        });
        let mut need = sample_need("tree", "hero", 3);
        need.orientation = Some("landscape".into());
        need.also_generate_if_found = Some(true);
        let cfg = IntegrationConfig::default();
        let p = preview_manual_batch(&m, &m, &[need], &cfg).unwrap();
        assert_eq!(p.pending_review_count, 1);
        assert_eq!(p.generate_count, 0);
        assert_eq!(p.results[0].decision, "pending_review");
    }
}
