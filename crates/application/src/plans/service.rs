use visual_library_domain::{approve_plan, can_run_automatic, CoveragePlanStatus};

use crate::error::AppError;
use crate::plans::dto::{PlanDto, PlanItemDto, PlanWithItemsDto};
use crate::ports::plans::PlanStore;

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

pub fn create_plan(
    store: &impl PlanStore,
    name: &str,
    description: Option<&str>,
    theme_id: Option<&str>,
) -> Result<PlanDto, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("plan name requerido".into()));
    }
    let ts = now();
    let plan = PlanDto {
        id: new_id("plan"),
        theme_id: theme_id.map(|s| s.to_string()),
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        status: CoveragePlanStatus::Draft.as_str().into(),
        approved_at: None,
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.insert_plan(&plan)?;
    Ok(plan)
}

pub fn list_plans(store: &impl PlanStore) -> Result<Vec<PlanDto>, AppError> {
    store.list_plans()
}

pub fn get_plan_with_items(
    store: &impl PlanStore,
    plan_id: &str,
) -> Result<PlanWithItemsDto, AppError> {
    let plan = store
        .get_plan(plan_id)?
        .ok_or_else(|| AppError::NotFound(format!("plan {plan_id}")))?;
    let items = store.list_items(plan_id)?;
    Ok(PlanWithItemsDto { plan, items })
}

pub fn add_plan_item(
    store: &impl PlanStore,
    plan_id: &str,
    concept_key: &str,
    representation_key: &str,
    action: Option<&str>,
    priority: Option<i64>,
    orientation: Option<&str>,
    style: Option<&str>,
) -> Result<PlanItemDto, AppError> {
    let plan = store
        .get_plan(plan_id)?
        .ok_or_else(|| AppError::NotFound(format!("plan {plan_id}")))?;
    if plan.status != CoveragePlanStatus::Draft.as_str() {
        return Err(AppError::Validation(
            "solo se pueden añadir items a planes draft".into(),
        ));
    }
    let ck = concept_key.trim();
    let rk = representation_key.trim();
    if ck.is_empty() || rk.is_empty() {
        return Err(AppError::Validation(
            "concept_key y representation_key requeridos".into(),
        ));
    }
    let constraints = serde_json::json!({
        "orientation": orientation.unwrap_or("any"),
        "style": style.unwrap_or("any"),
    });
    let ts = now();
    let item = PlanItemDto {
        id: new_id("pi"),
        plan_id: plan_id.to_string(),
        concept_id: None,
        representation_id: None,
        concept_key: Some(ck.to_string()),
        representation_key: Some(rk.to_string()),
        action: action.unwrap_or("ensure_approved_asset").to_string(),
        priority: priority.unwrap_or(100),
        target_count: 1,
        constraints_json: Some(constraints.to_string()),
        status: "pending".into(),
        created_at: ts.clone(),
        updated_at: ts,
    };
    store.insert_item(&item)?;
    Ok(item)
}

/// Approve draft plan (does not generate). Enables Automatic Factory.
pub fn approve_coverage_plan(store: &impl PlanStore, plan_id: &str) -> Result<PlanDto, AppError> {
    let plan = store
        .get_plan(plan_id)?
        .ok_or_else(|| AppError::NotFound(format!("plan {plan_id}")))?;
    let from = CoveragePlanStatus::parse(&plan.status).ok_or_else(|| {
        AppError::Validation(format!("status de plan desconocido: {}", plan.status))
    })?;
    let to = approve_plan(from).map_err(|e| AppError::Validation(e.to_string()))?;
    // Domain gate for automatic path must succeed after approve.
    can_run_automatic(to).map_err(|e| AppError::Validation(e.to_string()))?;
    let ts = now();
    store.update_plan_status(plan_id, to.as_str(), Some(&ts))?;
    store
        .get_plan(plan_id)?
        .ok_or_else(|| AppError::Internal("plan missing after approve".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Mem {
        plans: Mutex<HashMap<String, PlanDto>>,
        items: Mutex<Vec<PlanItemDto>>,
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
            let p = g.get_mut(id).ok_or_else(|| AppError::NotFound(id.into()))?;
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

    #[test]
    fn approve_enables_automatic_gate() {
        let m = Mem::default();
        let p = create_plan(&m, "Growth", None, None).unwrap();
        assert_eq!(p.status, "draft");
        add_plan_item(&m, &p.id, "tree", "hero", None, None, None, None).unwrap();
        let approved = approve_coverage_plan(&m, &p.id).unwrap();
        assert_eq!(approved.status, "approved");
        let st = CoveragePlanStatus::parse(&approved.status).unwrap();
        assert!(can_run_automatic(st).is_ok());
    }

    #[test]
    fn cannot_run_automatic_on_draft() {
        let m = Mem::default();
        let p = create_plan(&m, "Growth", None, None).unwrap();
        let st = CoveragePlanStatus::parse(&p.status).unwrap();
        assert!(can_run_automatic(st).is_err());
    }
}
