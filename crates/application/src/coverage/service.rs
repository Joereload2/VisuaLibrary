use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::ports::assets::AssetStore;
use crate::ports::catalog::CatalogStore;
use crate::ports::plans::PlanStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub concepts_total: usize,
    pub concepts_under_covered: usize,
    pub concepts_over_covered: usize,
    pub concepts_missing_representations: usize,
    pub waiting_review: usize,
    pub approved_assets: usize,
    pub draft_plans: usize,
    pub approved_plans: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageIssue {
    pub code: String,
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub cta_flow: String,
    pub related_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub summary: CoverageSummary,
    pub issues: Vec<CoverageIssue>,
}

/// Actionable coverage diagnostics (no charts-only).
pub fn get_coverage_report(
    catalog: &impl CatalogStore,
    assets: &impl AssetStore,
    plans: &impl PlanStore,
) -> Result<CoverageReport, AppError> {
    let concepts = catalog.list_concepts()?;
    let waiting = assets.list_by_status("waiting_review")?;
    let approved = assets.list_by_status("approved")?;
    let plan_list = plans.list_plans()?;

    let mut issues = Vec::new();
    let mut under = 0usize;
    let mut over = 0usize;
    let mut missing_reps = 0usize;

    for c in &concepts {
        if c.status == "deprecated" {
            continue;
        }
        let reps = catalog.list_representations(&c.id)?;
        let active_reps = reps.iter().filter(|r| r.status == "active").count();
        let approved_for_c = approved.iter().filter(|a| a.concept_id == c.id).count();

        // Defaults aligned with schema min_*: 1 rep, 1 approved.
        let min_reps = 1usize;
        let min_approved = 1usize;
        let max_approved: Option<usize> = None;

        if active_reps < min_reps {
            missing_reps += 1;
            issues.push(CoverageIssue {
                code: "representation_missing".into(),
                severity: "high".into(),
                title: format!("Concepto «{}» sin representaciones suficientes", c.key),
                detail: format!("{active_reps}/{min_reps} representaciones active"),
                cta_flow: "plans".into(),
                related_id: Some(c.id.clone()),
            });
        }

        if approved_for_c < min_approved {
            under += 1;
            issues.push(CoverageIssue {
                code: "concept_under_covered".into(),
                severity: "high".into(),
                title: format!("Concepto «{}» mal cubierto", c.key),
                detail: format!("{approved_for_c}/{min_approved} assets approved"),
                cta_flow: "plans".into(),
                related_id: Some(c.id.clone()),
            });
        }

        if let Some(max) = max_approved {
            if approved_for_c > max {
                over += 1;
                issues.push(CoverageIssue {
                    code: "concept_over_covered".into(),
                    severity: "low".into(),
                    title: format!("Concepto «{}» con demasiados assets", c.key),
                    detail: format!("{approved_for_c} approved (max {max})"),
                    cta_flow: "library".into(),
                    related_id: Some(c.id.clone()),
                });
            }
        }
    }

    if !waiting.is_empty() {
        issues.push(CoverageIssue {
            code: "review_backlog".into(),
            severity: "medium".into(),
            title: "Cola de Waiting Review".into(),
            detail: format!("{} assets esperan revisión humana", waiting.len()),
            cta_flow: "review".into(),
            related_id: None,
        });
    }

    let draft_plans = plan_list.iter().filter(|p| p.status == "draft").count();
    let approved_plans = plan_list.iter().filter(|p| p.status == "approved").count();

    if concepts.is_empty() {
        issues.push(CoverageIssue {
            code: "catalog_empty".into(),
            severity: "medium".into(),
            title: "Catálogo vacío".into(),
            detail: "No hay conceptos. Crea needs en Factory o items en Plans.".into(),
            cta_flow: "factory".into(),
            related_id: None,
        });
    }

    // Stable order: high first
    issues.sort_by(|a, b| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)));

    Ok(CoverageReport {
        summary: CoverageSummary {
            concepts_total: concepts.iter().filter(|c| c.status != "deprecated").count(),
            concepts_under_covered: under,
            concepts_over_covered: over,
            concepts_missing_representations: missing_reps,
            waiting_review: waiting.len(),
            approved_assets: approved.len(),
            draft_plans,
            approved_plans,
        },
        issues,
    })
}

fn severity_rank(s: &str) -> u8 {
    match s {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetDto;
    use crate::catalog::{ConceptDto, RepresentationDto, ThemeDto};
    use crate::plans::{PlanDto, PlanItemDto};
    use crate::ports::assets::AssetStore;
    use crate::ports::catalog::CatalogStore;
    use crate::ports::plans::PlanStore;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Mem {
        concepts: Mutex<Vec<ConceptDto>>,
        reps: Mutex<Vec<RepresentationDto>>,
        assets: Mutex<Vec<AssetDto>>,
        plans: Mutex<Vec<PlanDto>>,
    }

    impl CatalogStore for Mem {
        fn list_themes(&self) -> Result<Vec<ThemeDto>, AppError> {
            Ok(vec![])
        }
        fn ensure_theme(&self, _: &str, _: Option<&str>) -> Result<ThemeDto, AppError> {
            unimplemented!()
        }
        fn list_concepts(&self) -> Result<Vec<ConceptDto>, AppError> {
            Ok(self.concepts.lock().unwrap().clone())
        }
        fn ensure_concept(
            &self,
            _: &str,
            _: &str,
            _: Option<&str>,
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
            _: &str,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<RepresentationDto, AppError> {
            unimplemented!()
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
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<Option<AssetDto>, AppError> {
            Ok(None)
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

    impl PlanStore for Mem {
        fn insert_plan(&self, plan: &PlanDto) -> Result<(), AppError> {
            self.plans.lock().unwrap().push(plan.clone());
            Ok(())
        }
        fn get_plan(&self, id: &str) -> Result<Option<PlanDto>, AppError> {
            Ok(self
                .plans
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }
        fn list_plans(&self) -> Result<Vec<PlanDto>, AppError> {
            Ok(self.plans.lock().unwrap().clone())
        }
        fn update_plan_status(&self, _: &str, _: &str, _: Option<&str>) -> Result<(), AppError> {
            Ok(())
        }
        fn insert_item(&self, _: &PlanItemDto) -> Result<(), AppError> {
            Ok(())
        }
        fn list_items(&self, _: &str) -> Result<Vec<PlanItemDto>, AppError> {
            Ok(vec![])
        }
        fn update_item_status(&self, _: &str, _: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[test]
    fn empty_catalog_reports_issue() {
        let m = Mem::default();
        let r = get_coverage_report(&m, &m, &m).unwrap();
        assert!(r.issues.iter().any(|i| i.code == "catalog_empty"));
    }

    #[test]
    fn under_covered_and_backlog() {
        let m = Mem::default();
        m.concepts.lock().unwrap().push(ConceptDto {
            id: "c1".into(),
            key: "oak".into(),
            name: "Oak".into(),
            description: None,
            status: "active".into(),
        });
        m.assets.lock().unwrap().push(AssetDto {
            id: "w1".into(),
            concept_id: "c1".into(),
            representation_id: "r1".into(),
            status: "waiting_review".into(),
            storage_path: "x.png".into(),
            content_hash: None,
            width: None,
            height: None,
            mime: None,
            format: None,
            orientation: None,
            style: None,
            provider: None,
            prompt: None,
            generation_request_id: None,
            review_notes: None,
            reject_reason: None,
            duplicate_of_asset_id: None,
            approved_at: None,
            rejected_at: None,
            created_at: "t".into(),
            updated_at: "t".into(),
            package_id: None,
            package_path: None,
            beat_id: None,
            package_concept_key: None,
        });
        let r = get_coverage_report(&m, &m, &m).unwrap();
        assert!(r.summary.concepts_under_covered >= 1);
        assert_eq!(r.summary.waiting_review, 1);
        assert!(r.issues.iter().any(|i| i.code == "review_backlog"));
        assert!(r.issues.iter().any(|i| i.code == "concept_under_covered"));
    }
}
