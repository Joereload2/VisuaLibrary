use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use visual_library_domain::AssetStatus;

use crate::assets::AssetDto;
use crate::error::AppError;
use crate::integrations::{
    generate_image_bytes, select_image_provider_with_config, IntegrationConfig,
};
use crate::jobs::JobDto;
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;

/// Legacy 1x1 transparent PNG (kept for hash tests / fallback).
pub const STUB_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

const STUB_SIZE: u32 = 128;

/// Solid-color BMP unique per seed so regenerate is visibly different in Review.
pub fn colored_stub_bmp(seed: &str) -> Vec<u8> {
    let (r, g, b) = color_from_seed(seed);
    let w = STUB_SIZE;
    let h = STUB_SIZE;
    let row_stride = ((w * 3 + 3) / 4) * 4;
    let pixel_bytes = row_stride * h;
    let file_size = 14 + 40 + pixel_bytes;
    let mut out = Vec::with_capacity(file_size as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes()); // pixel offset
                                                 // BITMAPINFOHEADER
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes()); // bottom-up
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&pixel_bytes.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    let pad = vec![0u8; (row_stride - w * 3) as usize];
    for y in 0..h {
        // Subtle gradient so it looks like a real preview tile.
        let fy = (y as f32) / (h as f32);
        let rr = (r as f32 * (0.55 + 0.45 * fy)) as u8;
        let gg = (g as f32 * (0.55 + 0.45 * (1.0 - fy))) as u8;
        let bb = b;
        for _x in 0..w {
            out.push(bb);
            out.push(gg);
            out.push(rr);
        }
        out.extend_from_slice(&pad);
    }
    out
}

fn color_from_seed(seed: &str) -> (u8, u8, u8) {
    use sha2::{Digest, Sha256};
    let dig = Sha256::digest(seed.as_bytes());
    // Keep mid-high channel so the tile is visible on dark UI.
    (
        80 + (dig[0] % 160),
        80 + (dig[1] % 160),
        80 + (dig[2] % 160),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateStubInput {
    pub concept_id: String,
    pub representation_id: String,
    pub prompt: Option<String>,
    /// Selected image provider id (multi-provider; one per generate).
    pub provider: Option<String>,
    pub orientation: Option<String>,
    pub style: Option<String>,
    pub idempotency_key: Option<String>,
}

/// Normalize persisted provider ids (never store synthetic fallback labels as catalog ids).
pub fn normalize_provider_id(provider: Option<&str>) -> Option<String> {
    match provider.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) if s.starts_with("stub_fallback_from_") => Some("stub".into()),
        Some(s) => Some(s.to_string()),
        None => None,
    }
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

/// Enqueue + run generate_asset: job ends in `waiting_review` (D-019), never approved.
/// Image bytes come from the selected provider adapter (stub today; remote when connected).
pub fn generate_stub_asset(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    input: GenerateStubInput,
    cfg: &mut IntegrationConfig,
) -> Result<GenerateStubResult, AppError> {
    // Idempotent short-circuit only while the produced asset is still usable
    // (waiting_review). Rejected / superseded / duplicate must allow a new generate.
    // If the prior key is still stored but unusable, mint a retry key (UNIQUE on jobs).
    let mut idempotency_key = input.idempotency_key.clone();
    if let Some(key) = input.idempotency_key.as_deref() {
        if let Some(existing) = jobs.get_by_idempotency_key(key)? {
            if existing.status == "waiting_review" {
                if let Some(out) = existing.outputs_json.as_deref() {
                    if let Ok(parsed) = serde_json::from_str::<GenerateStubResult>(out) {
                        let asset_ok = assets
                            .get(&parsed.asset_id)?
                            .map(|a| a.status == AssetStatus::WaitingReview.as_str())
                            .unwrap_or(false);
                        if asset_ok {
                            return Ok(parsed);
                        }
                        idempotency_key = Some(format!("{key}:retry:{}", now()));
                    }
                }
            } else if existing.status == "queued" || existing.status == "running" {
                // In-flight: do not enqueue a twin under the same key.
                return Err(AppError::Validation(format!(
                    "job en curso para clave de idempotencia {key}"
                )));
            } else {
                // Terminal non-waiting job still holds the unique key — retry suffix.
                idempotency_key = Some(format!("{key}:retry:{}", now()));
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

    let preferred = normalize_provider_id(input.provider.as_deref());
    let selected = select_image_provider_with_config(preferred.as_deref(), cfg)?;
    let provider_id = selected.id.clone();
    let prompt_text = {
        let raw = input
            .prompt
            .clone()
            .unwrap_or_else(|| "Educational visual illustration, single subject, clear".into());
        crate::factory::with_no_text_guard(&raw)
    };
    // Remote failure: default = fail loud (no silent stub tile in Review).
    // Opt-in: cfg.allow_stub_fallback_on_image_error (Settings / advanced).
    let (generated, bill_provider, fallback_note) = match generate_image_bytes(
        &provider_id,
        &prompt_text,
        &asset_id,
        cfg,
    ) {
        Ok(g) => (g, provider_id.clone(), None),
        Err(e) if provider_id != "stub" && cfg.allow_stub_fallback_on_image_error => {
            let mut g = generate_image_bytes("stub", &prompt_text, &asset_id, cfg)?;
            g.provider_id = "stub".into();
            let note = format!(
                "FALLBACK stub: provider `{provider_id}` falló ({e}). \
                     No es imagen real del provider pedido."
            );
            (g, "stub".into(), Some(note))
        }
        Err(e) if provider_id != "stub" => {
            return Err(AppError::Validation(format!(
                    "No se pudo generar con `{provider_id}`: {e}\n\n\
                     Qué hacer: arranca OmniRoute / revisa model y base URL (Settings → Keys → Probar e2e), \
                     o elige provider `stub` en la need solo para probar el flujo.\n\
                     (No se sustituye en silencio por un tile local — evita confusiones en Review.)"
                )));
        }
        Err(e) => return Err(e),
    };
    // Track spend / free quota even when free (unit cost 0).
    let _usage = crate::integrations::record_usage(cfg, &bill_provider, 1)?;
    let ext = generated.format.as_str();
    // Path must match the provider that actually produced bytes (not the preferred one).
    let rel_path = format!("assets/{bill_provider}/{asset_id}.{ext}");

    let payload = serde_json::json!({
        "concept_id": input.concept_id,
        "representation_id": input.representation_id,
        "asset_id": asset_id,
        "generation_request_id": req_id,
        "relative_path": rel_path,
        "provider": bill_provider,
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
        idempotency_key,
        outputs_json: None,
        created_at: ts.clone(),
        updated_at: ts.clone(),
        started_at: None,
        finished_at: None,
    };
    jobs.insert(&job)?;

    // Run immediately (in-process worker foundation).
    jobs.update(&job_id, "running", 1, None, None, Some(&ts), None)?;

    media.write_asset_file(&rel_path, &generated.bytes)?;
    let hash = sha256_hex(&generated.bytes);

    let asset = AssetDto {
        id: asset_id.clone(),
        concept_id: input.concept_id,
        representation_id: input.representation_id,
        status: AssetStatus::WaitingReview.as_str().into(),
        storage_path: rel_path.clone(),
        content_hash: Some(hash),
        width: Some(generated.width),
        height: Some(generated.height),
        mime: Some(generated.mime.clone()),
        format: Some(generated.format.clone()),
        orientation: Some(
            input
                .orientation
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "any".into()),
        ),
        style: input.style.clone(),
        provider: Some(bill_provider.clone()),
        prompt: input.prompt,
        generation_request_id: Some(req_id),
        review_notes: fallback_note,
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
        let full = resolve_under_media_root(&self.media_root, relative_path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Storage(e.to_string()))?;
        }
        std::fs::write(&full, bytes).map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(full)
    }
}

/// Reject absolute paths and `..` components; keep writes under media_root.
pub fn resolve_under_media_root(
    media_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, AppError> {
    let rel = Path::new(relative_path);
    if rel.is_absolute() {
        return Err(AppError::Validation(
            "storage_path debe ser relativo al media_root".into(),
        ));
    }
    for component in rel.components() {
        use std::path::Component;
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => {
                return Err(AppError::Validation(
                    "storage_path no puede contener '..' ni prefijos especiales".into(),
                ));
            }
        }
    }
    let full = media_root.join(rel);
    // Soft check without requiring root to exist yet: prefix via components.
    let root_canon = media_root
        .canonicalize()
        .unwrap_or_else(|_| media_root.to_path_buf());
    // If parent of full exists, verify; otherwise join is enough after component filter.
    if let Ok(parent) = full.parent().unwrap_or(media_root).canonicalize() {
        if !parent.starts_with(&root_canon) && parent != root_canon {
            return Err(AppError::Validation(
                "storage_path escapa del media_root".into(),
            ));
        }
    }
    Ok(full)
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

    #[test]
    fn normalize_provider_strips_legacy_fallback_label() {
        assert_eq!(
            normalize_provider_id(Some("stub_fallback_from_omniroute")).as_deref(),
            Some("stub")
        );
        assert_eq!(
            normalize_provider_id(Some("omniroute")).as_deref(),
            Some("omniroute")
        );
        assert_eq!(normalize_provider_id(Some("  ")), None);
        assert_eq!(normalize_provider_id(None), None);
    }

    #[test]
    fn default_config_disallows_silent_stub_fallback() {
        let cfg = crate::integrations::IntegrationConfig::default();
        assert!(
            !cfg.allow_stub_fallback_on_image_error,
            "silent stub fallback must be off by default"
        );
    }

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
    fn path_safety_rejects_escape() {
        let root = PathBuf::from("/tmp/media-root-test");
        assert!(resolve_under_media_root(&root, "../etc/passwd").is_err());
        assert!(resolve_under_media_root(&root, "/abs/path.png").is_err());
        assert!(resolve_under_media_root(&root, "assets/stub/a.png").is_ok());
    }

    #[test]
    fn idempotency_skips_when_asset_no_longer_waiting() {
        let mem = MemAll::default();
        mem.concepts.lock().unwrap().push(ConceptDto {
            id: "c1".into(),
            key: "k".into(),
            name: "K".into(),
            description: None,
            status: "active".into(),
        });
        mem.reps.lock().unwrap().push(RepresentationDto {
            id: "r1".into(),
            concept_id: "c1".into(),
            key: "hero".into(),
            name: "Hero".into(),
            orientation_default: "any".into(),
            status: "active".into(),
        });
        // First generate
        let mut cfg = crate::integrations::IntegrationConfig::default();
        let first = generate_stub_asset(
            &mem,
            &mem,
            &mem,
            &mem,
            GenerateStubInput {
                concept_id: "c1".into(),
                representation_id: "r1".into(),
                prompt: None,
                provider: Some("stub".into()),
                orientation: None,
                style: None,
                idempotency_key: Some("idem-1".into()),
            },
            &mut cfg,
        )
        .unwrap();
        // Reject asset (leave job outputs pointing at it)
        {
            let mut g = mem.assets.lock().unwrap();
            let a = g.get_mut(&first.asset_id).unwrap();
            a.status = "rejected".into();
        }
        let second = generate_stub_asset(
            &mem,
            &mem,
            &mem,
            &mem,
            GenerateStubInput {
                concept_id: "c1".into(),
                representation_id: "r1".into(),
                prompt: None,
                provider: Some("stub".into()),
                orientation: None,
                style: None,
                idempotency_key: Some("idem-1".into()),
            },
            &mut cfg,
        )
        .unwrap();
        assert_ne!(first.asset_id, second.asset_id);
        assert_eq!(second.asset_status, "waiting_review");
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

        let mut cfg = crate::integrations::IntegrationConfig::default();
        let res = generate_stub_asset(
            &store,
            &store,
            &store,
            &store,
            GenerateStubInput {
                concept_id: "c1".into(),
                representation_id: "r1".into(),
                prompt: Some("test".into()),
                provider: Some("stub".into()),
                orientation: None,
                style: None,
                idempotency_key: Some("idem-1".into()),
            },
            &mut cfg,
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
                provider: Some("stub".into()),
                orientation: None,
                style: None,
                idempotency_key: Some("idem-1".into()),
            },
            &mut cfg,
        )
        .unwrap();
        assert_eq!(res.asset_id, res2.asset_id);
    }

    fn list_waiting_review_count(store: &MemAll) -> usize {
        store.list_by_status("waiting_review").unwrap().len()
    }
}
