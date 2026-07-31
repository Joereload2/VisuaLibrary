use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use visual_library_domain::AssetStatus;

use crate::assets::AssetDto;
use crate::error::AppError;
use crate::jobs::JobDto;
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;

/// Minimal 1x1 PNG (transparent).
pub const STUB_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateStubInput {
    pub concept_id: String,
    pub representation_id: String,
    pub prompt: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateStubResult {
    pub job_id: String,
    pub job_status: String,
    pub asset_id: String,
    pub asset_status: String,
    pub storage_path: String,
}

pub trait MediaWriter {
    fn write_asset_file(&self, relative_path: &str, bytes: &[u8]) -> Result<PathBuf, AppError>;
}

fn now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn new_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{prefix}_{}_{}", d.as_secs(), d.subsec_nanos())
}

/// Enqueue + run generate_asset stub: job ends in `waiting_review` (D-019), never approved.
pub fn generate_stub_asset(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    input: GenerateStubInput,
) -> Result<GenerateStubResult, AppError> {
    if let Some(key) = input.idempotency_key.as_deref() {
        if let Some(existing) = jobs.get_by_idempotency_key(key)? {
            if existing.status == "waiting_review" {
                if let Some(out) = existing.outputs_json.as_deref() {
                    if let Ok(parsed) = serde_json::from_str::<GenerateStubResult>(out) {
                        return Ok(parsed);
                    }
                }
            }
        }
    }

    // Validate concept / representation exist by listing (cheap for foundation).
    let concepts = catalog.list_concepts()?;
    if !concepts.iter().any(|c| c.id == input.concept_id) {
        return Err(AppError::NotFound(format!("concept {}", input.concept_id)));
    }
    let reps = catalog.list_representations(&input.concept_id)?;
    if !reps.iter().any(|r| r.id == input.representation_id) {
        return Err(AppError::NotFound(format!(
            "representation {}",
            input.representation_id
        )));
    }

    let job_id = new_id("job");
    let asset_id = new_id("asset");
    let req_id = new_id("greq");
    let ts = now();
    let rel_path = format!("assets/stub/{asset_id}.png");

    let payload = serde_json::json!({
        "concept_id": input.concept_id,
        "representation_id": input.representation_id,
        "asset_id": asset_id,
        "generation_request_id": req_id,
        "relative_path": rel_path,
    });

    let job = JobDto {
        id: job_id.clone(),
        job_type: "generate_asset".into(),
        payload_json: payload.to_string(),
        status: "queued".into(),
        priority: 100,
        attempts: 0,
        max_attempts: 3,
        last_error: None,
        related_entity_type: Some("asset".into()),
        related_entity_id: Some(asset_id.clone()),
        idempotency_key: input.idempotency_key.clone(),
        outputs_json: None,
        created_at: ts.clone(),
        updated_at: ts.clone(),
        started_at: None,
        finished_at: None,
    };
    jobs.insert(&job)?;

    // Run immediately (in-process worker foundation).
    jobs.update(&job_id, "running", 1, None, None, Some(&ts), None)?;

    media.write_asset_file(&rel_path, STUB_PNG)?;
    let hash = sha256_hex(STUB_PNG);

    let asset = AssetDto {
        id: asset_id.clone(),
        concept_id: input.concept_id,
        representation_id: input.representation_id,
        status: AssetStatus::WaitingReview.as_str().into(),
        storage_path: rel_path.clone(),
        content_hash: Some(hash),
        width: Some(1),
        height: Some(1),
        mime: Some("image/png".into()),
        format: Some("png".into()),
        orientation: Some("any".into()),
        style: None,
        provider: Some("stub".into()),
        prompt: input.prompt,
        generation_request_id: Some(req_id),
        review_notes: None,
        reject_reason: None,
        duplicate_of_asset_id: None,
        approved_at: None,
        rejected_at: None,
        created_at: ts.clone(),
        updated_at: ts.clone(),
    };
    assets.insert(&asset)?;

    let result = GenerateStubResult {
        job_id: job_id.clone(),
        job_status: "waiting_review".into(),
        asset_id: asset_id.clone(),
        asset_status: AssetStatus::WaitingReview.as_str().into(),
        storage_path: rel_path,
    };
    let out = serde_json::to_string(&result)
        .map_err(|e| AppError::Internal(format!("serialize result: {e}")))?;
    let fin = now();
    jobs.update(
        &job_id,
        "waiting_review",
        1,
        None,
        Some(&out),
        Some(&ts),
        Some(&fin),
    )?;

    // Invariant: never library-visible after generate.
    debug_assert_ne!(result.asset_status, "approved");
    Ok(result)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// Filesystem writer rooted at media_root.
pub struct FsMediaWriter {
    pub media_root: PathBuf,
}

impl MediaWriter for FsMediaWriter {
    fn write_asset_file(&self, relative_path: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
        if relative_path.contains("..") {
            return Err(AppError::Validation(
                "storage_path no puede contener '..'".into(),
            ));
        }
        let full = self.media_root.join(relative_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Storage(e.to_string()))?;
        }
        std::fs::write(&full, bytes).map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(full)
    }
}

pub fn media_writer_for(media_root: &Path) -> FsMediaWriter {
    FsMediaWriter {
        media_root: media_root.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ConceptDto, RepresentationDto, ThemeDto};
    use crate::ports::catalog::CatalogStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemAll {
        assets: Mutex<HashMap<String, AssetDto>>,
        jobs: Mutex<HashMap<String, JobDto>>,
        concepts: Mutex<Vec<ConceptDto>>,
        reps: Mutex<Vec<RepresentationDto>>,
        files: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl AssetStore for MemAll {
        fn insert(&self, asset: &AssetDto) -> Result<(), AppError> {
            self.assets
                .lock()
                .unwrap()
                .insert(asset.id.clone(), asset.clone());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<AssetDto>, AppError> {
            Ok(self.assets.lock().unwrap().get(id).cloned())
        }
        fn list_by_status(&self, status: &str) -> Result<Vec<AssetDto>, AppError> {
            Ok(self
                .assets
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.status == status)
                .cloned()
                .collect())
        }
        fn update_status(
            &self,
            id: &str,
            status: &str,
            approved_at: Option<&str>,
            rejected_at: Option<&str>,
            reject_reason: Option<&str>,
        ) -> Result<(), AppError> {
            let mut g = self.assets.lock().unwrap();
            let a = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            a.status = status.into();
            if let Some(t) = approved_at {
                a.approved_at = Some(t.into());
            }
            if let Some(t) = rejected_at {
                a.rejected_at = Some(t.into());
            }
            if let Some(r) = reject_reason {
                a.reject_reason = Some(r.into());
            }
            Ok(())
        }

        fn find_approved_match(
            &self,
            representation_id: &str,
            orientation: &str,
            style: &str,
        ) -> Result<Option<AssetDto>, AppError> {
            use visual_library_domain::field_matches;
            Ok(self
                .assets
                .lock()
                .unwrap()
                .values()
                .find(|a| {
                    a.status == "approved"
                        && a.representation_id == representation_id
                        && field_matches(orientation, a.orientation.as_deref())
                        && field_matches(style, a.style.as_deref())
                })
                .cloned())
        }

        fn update_metadata(
            &self,
            id: &str,
            review_notes: Option<&str>,
            orientation: Option<&str>,
            style: Option<&str>,
            prompt: Option<&str>,
        ) -> Result<(), AppError> {
            let mut g = self.assets.lock().unwrap();
            let a = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            if let Some(n) = review_notes {
                a.review_notes = Some(n.into());
            }
            if let Some(o) = orientation {
                a.orientation = Some(o.into());
            }
            if let Some(s) = style {
                a.style = Some(s.into());
            }
            if let Some(p) = prompt {
                a.prompt = Some(p.into());
            }
            Ok(())
        }

        fn set_duplicate_of(&self, id: &str, of_asset_id: &str) -> Result<(), AppError> {
            let mut g = self.assets.lock().unwrap();
            let a = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            a.duplicate_of_asset_id = Some(of_asset_id.into());
            Ok(())
        }
    }

    impl JobStore for MemAll {
        fn insert(&self, job: &JobDto) -> Result<(), AppError> {
            self.jobs
                .lock()
                .unwrap()
                .insert(job.id.clone(), job.clone());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<JobDto>, AppError> {
            Ok(self.jobs.lock().unwrap().get(id).cloned())
        }
        fn get_by_idempotency_key(&self, key: &str) -> Result<Option<JobDto>, AppError> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .values()
                .find(|j| j.idempotency_key.as_deref() == Some(key))
                .cloned())
        }
        fn update(
            &self,
            id: &str,
            status: &str,
            attempts: i64,
            last_error: Option<&str>,
            outputs_json: Option<&str>,
            started_at: Option<&str>,
            finished_at: Option<&str>,
        ) -> Result<(), AppError> {
            let mut g = self.jobs.lock().unwrap();
            let j = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            j.status = status.into();
            j.attempts = attempts;
            if let Some(e) = last_error {
                j.last_error = Some(e.into());
            }
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

    impl CatalogStore for MemAll {
        fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
            Ok(vec![])
        }
        fn ensure_theme(
            &self,
            _name: &str,
            _description: Option<&str>,
        ) -> Result<ThemeDto, AppError> {
            unimplemented!()
        }
        fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError> {
            Ok(self.concepts.lock().unwrap().clone())
        }
        fn ensure_concept(
            &self,
            _key: &str,
            _name: &str,
            _description: Option<&str>,
        ) -> Result<ConceptDto, AppError> {
            unimplemented!()
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
            _concept_id: &str,
            _key: &str,
            _name: &str,
            _orientation_default: &str,
        ) -> Result<RepresentationDto, AppError> {
            unimplemented!()
        }
    }

    impl MediaWriter for MemAll {
        fn write_asset_file(&self, relative_path: &str, bytes: &[u8]) -> Result<PathBuf, AppError> {
            self.files
                .lock()
                .unwrap()
                .insert(relative_path.to_string(), bytes.to_vec());
            Ok(PathBuf::from(relative_path))
        }
    }

    #[test]
    fn generate_ends_waiting_review_not_approved() {
        let store = MemAll::default();
        store.concepts.lock().unwrap().push(ConceptDto {
            id: "c1".into(),
            key: "k".into(),
            name: "N".into(),
            description: None,
            status: "active".into(),
        });
        store.reps.lock().unwrap().push(RepresentationDto {
            id: "r1".into(),
            concept_id: "c1".into(),
            key: "hero".into(),
            name: "Hero".into(),
            orientation_default: "any".into(),
            status: "active".into(),
        });

        let res = generate_stub_asset(
            &store,
            &store,
            &store,
            &store,
            GenerateStubInput {
                concept_id: "c1".into(),
                representation_id: "r1".into(),
                prompt: Some("test".into()),
                idempotency_key: Some("idem-1".into()),
            },
        )
        .unwrap();

        assert_eq!(res.job_status, "waiting_review");
        assert_eq!(res.asset_status, "waiting_review");
        assert_ne!(res.asset_status, "approved");
        assert_eq!(list_waiting_review_count(&store), 1);

        // idempotent
        let res2 = generate_stub_asset(
            &store,
            &store,
            &store,
            &store,
            GenerateStubInput {
                concept_id: "c1".into(),
                representation_id: "r1".into(),
                prompt: None,
                idempotency_key: Some("idem-1".into()),
            },
        )
        .unwrap();
        assert_eq!(res.asset_id, res2.asset_id);
    }

    fn list_waiting_review_count(store: &MemAll) -> usize {
        store.list_by_status("waiting_review").unwrap().len()
    }
}
