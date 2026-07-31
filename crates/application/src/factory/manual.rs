use serde::{Deserialize, Serialize};
use visual_library_domain::{decide_acquisition, field_matches, AcquisitionDecision};

use crate::catalog::{ensure_concept, ensure_representation};
use crate::error::AppError;
use crate::jobs::{generate_stub_asset, GenerateStubInput, GenerateStubResult, MediaWriter};
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;

/// One structured visual need (Manual Factory input).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualNeed {
    pub concept_key: String,
    pub concept_name: Option<String>,
    pub representation_key: String,
    pub representation_name: Option<String>,
    pub prompt: Option<String>,
    pub orientation: Option<String>,
    pub style: Option<String>,
    pub provider: Option<String>,
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
    pub generate: Option<GenerateStubResult>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualBatchPreview {
    pub results: Vec<ManualNeedResult>,
    pub found_count: usize,
    pub generate_count: usize,
    pub skipped_count: usize,
}

fn orient(n: &ManualNeed) -> String {
    n.orientation.as_deref().unwrap_or("any").trim().to_string()
}

fn style(n: &ManualNeed) -> String {
    n.style.as_deref().unwrap_or("any").trim().to_string()
}

fn resolve_need(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    index: usize,
    need: &ManualNeed,
) -> Result<ManualNeedResult, AppError> {
    let ckey = need.concept_key.trim();
    let rkey = need.representation_key.trim();
    if ckey.is_empty() || rkey.is_empty() {
        return Ok(ManualNeedResult {
            index,
            decision: AcquisitionDecision::Skipped.as_str().into(),
            concept_id: String::new(),
            concept_key: ckey.into(),
            representation_id: String::new(),
            representation_key: rkey.into(),
            found_asset_id: None,
            generate: None,
            message: "concept_key y representation_key son obligatorios".into(),
        });
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

    let decision = decide_acquisition(sufficient);
    match decision {
        AcquisitionDecision::Found => {
            let id = candidate.map(|a| a.id).unwrap_or_default();
            Ok(ManualNeedResult {
                index,
                decision: decision.as_str().into(),
                concept_id: concept.id,
                concept_key: concept.key,
                representation_id: rep.id,
                representation_key: rep.key,
                found_asset_id: Some(id.clone()),
                generate: None,
                message: format!("FOUND asset {id}"),
            })
        }
        AcquisitionDecision::Generate => Ok(ManualNeedResult {
            index,
            decision: decision.as_str().into(),
            concept_id: concept.id,
            concept_key: concept.key,
            representation_id: rep.id,
            representation_key: rep.key,
            found_asset_id: None,
            generate: None,
            message: "GENERATE — no hay approved suficientemente bueno".into(),
        }),
        AcquisitionDecision::Skipped => Ok(ManualNeedResult {
            index,
            decision: decision.as_str().into(),
            concept_id: concept.id,
            concept_key: concept.key,
            representation_id: rep.id,
            representation_key: rep.key,
            found_asset_id: None,
            generate: None,
            message: "SKIPPED".into(),
        }),
    }
}

/// Preview only: resolve FOUND vs GENERATE without writing new assets.
pub fn preview_manual_batch(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    needs: &[ManualNeed],
) -> Result<ManualBatchPreview, AppError> {
    if needs.is_empty() {
        return Err(AppError::Validation("lista de necesidades vacía".into()));
    }
    let mut results = Vec::with_capacity(needs.len());
    let mut found_count = 0usize;
    let mut generate_count = 0usize;
    let mut skipped_count = 0usize;

    for (i, need) in needs.iter().enumerate() {
        let r = resolve_need(catalog, assets, i, need)?;
        match r.decision.as_str() {
            "found" => found_count += 1,
            "generate" => generate_count += 1,
            _ => skipped_count += 1,
        }
        results.push(r);
    }

    Ok(ManualBatchPreview {
        results,
        found_count,
        generate_count,
        skipped_count,
    })
}

/// Submit: only GENERATE items create stub assets → waiting_review.
pub fn submit_manual_batch(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    needs: &[ManualNeed],
    batch_id: Option<&str>,
) -> Result<ManualBatchPreview, AppError> {
    let mut preview = preview_manual_batch(catalog, assets, needs)?;
    let batch = batch_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("batch_{}", now_secs()));

    for r in preview.results.iter_mut() {
        if r.decision != "generate" {
            continue;
        }
        let need = &needs[r.index];
        let gen = generate_stub_asset(
            catalog,
            assets,
            jobs,
            media,
            GenerateStubInput {
                concept_id: r.concept_id.clone(),
                representation_id: r.representation_id.clone(),
                prompt: need.prompt.clone(),
                idempotency_key: Some(format!(
                    "manual:{}:{}:{}",
                    batch, r.concept_key, r.representation_key
                )),
            },
        )?;
        r.generate = Some(gen);
        r.message = "GENERATE → waiting_review".into();
    }

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

    #[test]
    fn preview_generate_when_empty() {
        let m = Mem::default();
        let needs = vec![ManualNeed {
            concept_key: "tree".into(),
            concept_name: Some("Tree".into()),
            representation_key: "hero".into(),
            representation_name: None,
            prompt: Some("p".into()),
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: Some("stub".into()),
        }];
        let p = preview_manual_batch(&m, &m, &needs).unwrap();
        assert_eq!(p.generate_count, 1);
        assert_eq!(p.found_count, 0);
    }

    #[test]
    fn preview_found_when_approved_exists() {
        let m = Mem::default();
        let c = ensure_concept(&m, "tree", "Tree", None).unwrap();
        let r = ensure_representation(&m, &c.id, "hero", "Hero", Some("landscape")).unwrap();
        m.assets.lock().unwrap().push(AssetDto {
            id: "a1".into(),
            concept_id: c.id,
            representation_id: r.id,
            status: "approved".into(),
            storage_path: "x.png".into(),
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
        let needs = vec![ManualNeed {
            concept_key: "tree".into(),
            concept_name: None,
            representation_key: "hero".into(),
            representation_name: None,
            prompt: None,
            orientation: Some("landscape".into()),
            style: Some("any".into()),
            provider: None,
        }];
        let p = preview_manual_batch(&m, &m, &needs).unwrap();
        assert_eq!(p.found_count, 1);
        assert_eq!(p.generate_count, 0);
        assert_eq!(p.results[0].found_asset_id.as_deref(), Some("a1"));
    }

    #[test]
    fn submit_only_generates_missing() {
        let m = Mem::default();
        let needs = vec![ManualNeed {
            concept_key: "rock".into(),
            concept_name: None,
            representation_key: "detail".into(),
            representation_name: None,
            prompt: Some("gen".into()),
            orientation: Some("any".into()),
            style: Some("any".into()),
            provider: None,
        }];
        let out = submit_manual_batch(&m, &m, &m, &m, &needs, Some("b1")).unwrap();
        assert_eq!(out.generate_count, 1);
        assert!(out.results[0].generate.is_some());
        assert_eq!(
            out.results[0].generate.as_ref().unwrap().asset_status,
            "waiting_review"
        );
        // FOUND path: no second asset if we already have approved after first... first was generate
        assert_eq!(m.assets.lock().unwrap().len(), 1);
        assert_eq!(m.assets.lock().unwrap()[0].status, "waiting_review");
    }
}
