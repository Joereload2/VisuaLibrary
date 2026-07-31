use serde::{Deserialize, Serialize};
use visual_library_domain::{can_run_automatic, CoveragePlanStatus};

use crate::error::AppError;
use crate::factory::manual::{
    preview_manual_batch, submit_manual_batch, ManualBatchPreview, ManualNeed,
};
use crate::jobs::MediaWriter;
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::jobs::JobStore;
use crate::ports::plans::PlanStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticRunResult {
    pub plan_id: String,
    pub plan_status: String,
    pub batch: ManualBatchPreview,
    pub items_touched: usize,
}

fn constraints_field(json: &Option<String>, key: &str) -> String {
    json.as_ref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "any".into())
}

/// Run Automatic Factory: only approved plans; converts plan items → needs → FOUND/GENERATE.
pub fn run_automatic_from_plan(
    plans: &impl PlanStore,
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    jobs: &impl JobStore,
    media: &impl MediaWriter,
    plan_id: &str,
) -> Result<AutomaticRunResult, AppError> {
    let plan = plans
        .get_plan(plan_id)?
        .ok_or_else(|| AppError::NotFound(format!("plan {plan_id}")))?;
    let status = CoveragePlanStatus::parse(&plan.status).ok_or_else(|| {
        AppError::Validation(format!("status de plan desconocido: {}", plan.status))
    })?;
    can_run_automatic(status).map_err(|e| AppError::Validation(e.to_string()))?;

    let items = plans.list_items(plan_id)?;
    let pending: Vec<_> = items
        .into_iter()
        .filter(|i| i.status == "pending" || i.status == "scheduled")
        .collect();
    if pending.is_empty() {
        return Err(AppError::Validation(
            "el plan no tiene items pending/scheduled".into(),
        ));
    }

    let mut needs = Vec::new();
    for item in &pending {
        let ck = item
            .concept_key
            .as_deref()
            .ok_or_else(|| AppError::Validation("item sin concept_key".into()))?;
        let rk = item
            .representation_key
            .as_deref()
            .ok_or_else(|| AppError::Validation("item sin representation_key".into()))?;
        needs.push(ManualNeed {
            concept_key: ck.to_string(),
            concept_name: Some(ck.to_string()),
            representation_key: rk.to_string(),
            representation_name: Some(rk.to_string()),
            prompt: Some(format!("auto:{}:{}", ck, rk)),
            orientation: Some(constraints_field(&item.constraints_json, "orientation")),
            style: Some(constraints_field(&item.constraints_json, "style")),
            provider: Some("stub".into()),
        });
    }

    // Preview first to classify, then submit only generates.
    let _preview = preview_manual_batch(catalog, assets, &needs)?;
    let batch = submit_manual_batch(
        catalog,
        assets,
        jobs,
        media,
        &needs,
        Some(&format!("auto:{plan_id}")),
    )?;

    // Update item statuses: found → fulfilled; generate → scheduled.
    for (item, result) in pending.iter().zip(batch.results.iter()) {
        let st = match result.decision.as_str() {
            "found" => "fulfilled",
            "generate" => "scheduled",
            _ => "pending",
        };
        plans.update_item_status(&item.id, st)?;
    }

    Ok(AutomaticRunResult {
        plan_id: plan_id.to_string(),
        plan_status: plan.status,
        items_touched: pending.len(),
        batch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetDto;
    use crate::catalog::{ConceptDto, RepresentationDto, ThemeDto};
    use crate::jobs::JobDto;
    use crate::plans::{add_plan_item, approve_coverage_plan, create_plan};
    use crate::plans::{PlanDto, PlanItemDto};
    use crate::ports::assets::AssetStore;
    use crate::ports::catalog::CatalogStore;
    use crate::ports::jobs::JobStore;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Mem {
        plans: Mutex<HashMap<String, PlanDto>>,
        items: Mutex<Vec<PlanItemDto>>,
        concepts: Mutex<Vec<ConceptDto>>,
        reps: Mutex<Vec<RepresentationDto>>,
        assets: Mutex<Vec<AssetDto>>,
        jobs: Mutex<Vec<JobDto>>,
    }

    impl PlanStore for Mem {
        fn insert_plan(&self, plan: &PlanDto) -> Result<(), AppError> {
            self.plans
                .lock()
                .unwrap()
                .insert(plan.id.clone(), plan.clone());
            Ok(())
        }
        fn get_plan(&self, id: &str) -> Result<Option<PlanDto>, AppError> {
            Ok(self.plans.lock().unwrap().get(id).cloned())
        }
        fn list_plans(&self) -> Result<Vec<PlanDto>, AppError> {
            Ok(self.plans.lock().unwrap().values().cloned().collect())
        }
        fn update_plan_status(
            &self,
            id: &str,
            status: &str,
            approved_at: Option<&str>,
        ) -> Result<(), AppError> {
            let mut g = self.plans.lock().unwrap();
            let p = g.get_mut(id).unwrap();
            p.status = status.into();
            if let Some(a) = approved_at {
                p.approved_at = Some(a.into());
            }
            Ok(())
        }
        fn insert_item(&self, item: &PlanItemDto) -> Result<(), AppError> {
            self.items.lock().unwrap().push(item.clone());
            Ok(())
        }
        fn list_items(&self, plan_id: &str) -> Result<Vec<PlanItemDto>, AppError> {
            Ok(self
                .items
                .lock()
                .unwrap()
                .iter()
                .filter(|i| i.plan_id == plan_id)
                .cloned()
                .collect())
        }
        fn update_item_status(&self, id: &str, status: &str) -> Result<(), AppError> {
            let mut g = self.items.lock().unwrap();
            let i = g.iter_mut().find(|i| i.id == id).unwrap();
            i.status = status.into();
            Ok(())
        }
    }

    impl CatalogStore for Mem {
        fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
            Ok(vec![])
        }
        fn ensure_theme(&self, name: &str, _: Option<&str>) -> Result<ThemeDto, AppError> {
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
            _: Option<&str>,
        ) -> Result<ConceptDto, AppError> {
            let mut g = self.concepts.lock().unwrap();
            if let Some(c) = g.iter().find(|c| c.key == key) {
                return Ok(c.clone());
            }
            let c = ConceptDto {
                id: format!("c_{key}"),
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
            _: &str,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
            _: Option<&str>,
        ) -> Result<(), AppError> {
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
                .iter()
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
            _: &str,
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
            _: Option<&str>,
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
        fn write_asset_file(&self, relative_path: &str, _: &[u8]) -> Result<PathBuf, AppError> {
            Ok(PathBuf::from(relative_path))
        }
    }

    #[test]
    fn draft_plan_cannot_run_automatic() {
        let m = Mem::default();
        let p = create_plan(&m, "Grow", None, None).unwrap();
        add_plan_item(&m, &p.id, "a", "b", None, None, None, None).unwrap();
        let err = run_automatic_from_plan(&m, &m, &m, &m, &m, &p.id).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn approved_plan_runs_and_generates_waiting_review() {
        let m = Mem::default();
        let p = create_plan(&m, "Grow", None, None).unwrap();
        add_plan_item(
            &m,
            &p.id,
            "river",
            "wide",
            None,
            None,
            Some("landscape"),
            None,
        )
        .unwrap();
        approve_coverage_plan(&m, &p.id).unwrap();
        let run = run_automatic_from_plan(&m, &m, &m, &m, &m, &p.id).unwrap();
        assert_eq!(run.batch.generate_count, 1);
        assert_eq!(
            run.batch.results[0].generate.as_ref().unwrap().asset_status,
            "waiting_review"
        );
        let items = m.list_items(&p.id).unwrap();
        assert_eq!(items[0].status, "scheduled");
    }
}
