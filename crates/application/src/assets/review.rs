use visual_library_domain::{approve, mark_duplicate, reject, supersede, AssetStatus};

use crate::assets::AssetDto;
use crate::error::AppError;
use crate::jobs::{generate_stub_asset, GenerateStubInput, GenerateStubResult, MediaWriter};
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;

fn now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

pub fn list_waiting_review(store: &impl AssetStore) -> Result<Vec<AssetDto>, AppError> {
    store.list_by_status(AssetStatus::WaitingReview.as_str())
}

pub fn list_library_assets(store: &impl AssetStore) -> Result<Vec<AssetDto>, AppError> {
    store.list_by_status(AssetStatus::Approved.as_str())
}

pub fn approve_asset(store: &impl AssetStore, asset_id: &str) -> Result<AssetDto, AppError> {
    let asset = store
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    let from = AssetStatus::parse(&asset.status).ok_or_else(|| {
        AppError::Validation(format!("status de asset desconocido: {}", asset.status))
    })?;
    let to = approve(from).map_err(|e| AppError::Validation(e.to_string()))?;
    let ts = now();
    store.update_status(asset_id, to.as_str(), Some(&ts), None, None)?;
    store
        .get(asset_id)?
        .ok_or_else(|| AppError::Internal("asset desapareció tras approve".into()))
}

pub fn reject_asset(
    store: &impl AssetStore,
    asset_id: &str,
    reason: Option<&str>,
) -> Result<AssetDto, AppError> {
    let asset = store
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    let from = AssetStatus::parse(&asset.status).ok_or_else(|| {
        AppError::Validation(format!("status de asset desconocido: {}", asset.status))
    })?;
    let to = reject(from).map_err(|e| AppError::Validation(e.to_string()))?;
    let ts = now();
    store.update_status(asset_id, to.as_str(), None, Some(&ts), reason)?;
    store
        .get(asset_id)?
        .ok_or_else(|| AppError::Internal("asset desapareció tras reject".into()))
}

/// Edit metadata without leaving waiting_review.
pub fn edit_asset_metadata(
    store: &impl AssetStore,
    asset_id: &str,
    review_notes: Option<&str>,
    orientation: Option<&str>,
    style: Option<&str>,
    prompt: Option<&str>,
) -> Result<AssetDto, AppError> {
    let asset = store
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    if asset.status != AssetStatus::WaitingReview.as_str() {
        return Err(AppError::Validation(
            "solo se edita metadata en waiting_review".into(),
        ));
    }
    store.update_metadata(asset_id, review_notes, orientation, style, prompt)?;
    store
        .get(asset_id)?
        .ok_or_else(|| AppError::Internal("asset missing after metadata edit".into()))
}

pub fn mark_asset_duplicate(
    store: &impl AssetStore,
    asset_id: &str,
    of_asset_id: &str,
) -> Result<AssetDto, AppError> {
    if asset_id == of_asset_id {
        return Err(AppError::Validation(
            "duplicate_of no puede ser el mismo asset".into(),
        ));
    }
    let asset = store
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    let of = store
        .get(of_asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset destino {of_asset_id}")))?;
    if of.status != AssetStatus::Approved.as_str()
        && of.status != AssetStatus::WaitingReview.as_str()
    {
        return Err(AppError::Validation(
            "duplicate_of debe ser approved o waiting_review".into(),
        ));
    }
    let from = AssetStatus::parse(&asset.status).ok_or_else(|| {
        AppError::Validation(format!("status de asset desconocido: {}", asset.status))
    })?;
    let to = mark_duplicate(from).map_err(|e| AppError::Validation(e.to_string()))?;
    store.update_status(asset_id, to.as_str(), None, None, Some("duplicate"))?;
    store.set_duplicate_of(asset_id, of_asset_id)?;
    store
        .get(asset_id)?
        .ok_or_else(|| AppError::Internal("asset missing after mark duplicate".into()))
}

/// Supersede current waiting asset and generate a new stub into waiting_review.
pub fn regenerate_asset(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    asset_id: &str,
) -> Result<GenerateStubResult, AppError> {
    let asset = assets
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    let from = AssetStatus::parse(&asset.status).ok_or_else(|| {
        AppError::Validation(format!("status de asset desconocido: {}", asset.status))
    })?;
    let to = supersede(from).map_err(|e| AppError::Validation(e.to_string()))?;
    assets.update_status(asset_id, to.as_str(), None, None, Some("regenerated"))?;

    generate_stub_asset(
        catalog,
        assets,
        jobs,
        media,
        GenerateStubInput {
            concept_id: asset.concept_id,
            representation_id: asset.representation_id,
            prompt: asset.prompt,
            idempotency_key: Some(format!("regen:{asset_id}:{}", now())),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemAssets(Mutex<HashMap<String, AssetDto>>);

    impl AssetStore for MemAssets {
        fn insert(&self, asset: &AssetDto) -> Result<(), AppError> {
            self.0
                .lock()
                .unwrap()
                .insert(asset.id.clone(), asset.clone());
            Ok(())
        }

        fn get(&self, id: &str) -> Result<Option<AssetDto>, AppError> {
            Ok(self.0.lock().unwrap().get(id).cloned())
        }

        fn list_by_status(&self, status: &str) -> Result<Vec<AssetDto>, AppError> {
            Ok(self
                .0
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
            let mut g = self.0.lock().unwrap();
            let a = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            a.status = status.to_string();
            if let Some(t) = approved_at {
                a.approved_at = Some(t.to_string());
            }
            if let Some(t) = rejected_at {
                a.rejected_at = Some(t.to_string());
            }
            if let Some(r) = reject_reason {
                a.reject_reason = Some(r.to_string());
            }
            a.updated_at = now();
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
                .0
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
            let mut g = self.0.lock().unwrap();
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
            let mut g = self.0.lock().unwrap();
            let a = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
            a.duplicate_of_asset_id = Some(of_asset_id.into());
            Ok(())
        }
    }

    fn sample_waiting() -> AssetDto {
        AssetDto {
            id: "a1".into(),
            concept_id: "c1".into(),
            representation_id: "r1".into(),
            status: "waiting_review".into(),
            storage_path: "x.png".into(),
            content_hash: None,
            width: Some(1),
            height: Some(1),
            mime: Some("image/png".into()),
            format: Some("png".into()),
            orientation: None,
            style: None,
            provider: Some("stub".into()),
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: None,
            rejected_at: None,
            created_at: "t0".into(),
            updated_at: "t0".into(),
        }
    }

    #[test]
    fn approve_moves_to_library_status() {
        let store = MemAssets::default();
        store.insert(&sample_waiting()).unwrap();
        let a = approve_asset(&store, "a1").unwrap();
        assert_eq!(a.status, "approved");
        assert_eq!(list_library_assets(&store).unwrap().len(), 1);
        assert!(list_waiting_review(&store).unwrap().is_empty());
    }

    #[test]
    fn cannot_approve_twice() {
        let store = MemAssets::default();
        store.insert(&sample_waiting()).unwrap();
        approve_asset(&store, "a1").unwrap();
        assert!(approve_asset(&store, "a1").is_err());
    }

    #[test]
    fn edit_metadata_stays_waiting() {
        let store = MemAssets::default();
        store.insert(&sample_waiting()).unwrap();
        let a =
            edit_asset_metadata(&store, "a1", Some("ok"), Some("landscape"), None, None).unwrap();
        assert_eq!(a.status, "waiting_review");
        assert_eq!(a.review_notes.as_deref(), Some("ok"));
        assert_eq!(a.orientation.as_deref(), Some("landscape"));
    }

    #[test]
    fn mark_duplicate_links_target() {
        let store = MemAssets::default();
        let mut a = sample_waiting();
        a.id = "a1".into();
        let mut b = sample_waiting();
        b.id = "a2".into();
        b.status = "approved".into();
        store.insert(&a).unwrap();
        store.insert(&b).unwrap();
        let d = mark_asset_duplicate(&store, "a1", "a2").unwrap();
        assert_eq!(d.status, "duplicate");
        assert_eq!(d.duplicate_of_asset_id.as_deref(), Some("a2"));
    }
}
