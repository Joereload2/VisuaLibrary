use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoveragePlanStatus {
    Draft,
    Approved,
    Archived,
    Superseded,
}

impl CoveragePlanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Approved => "approved",
            Self::Archived => "archived",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "approved" => Some(Self::Approved),
            "archived" => Some(Self::Archived),
            "superseded" => Some(Self::Superseded),
            _ => None,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("coverage plan is not approved (status={status})")]
    NotApproved { status: &'static str },
}

/// Automatic Factory may run only when the plan is approved (D-005 / domain gate).
pub fn can_run_automatic(status: CoveragePlanStatus) -> Result<(), PlanError> {
    match status {
        CoveragePlanStatus::Approved => Ok(()),
        other => Err(PlanError::NotApproved {
            status: other.as_str(),
        }),
    }
}

pub fn approve_plan(from: CoveragePlanStatus) -> Result<CoveragePlanStatus, PlanError> {
    match from {
        CoveragePlanStatus::Draft => Ok(CoveragePlanStatus::Approved),
        other => Err(PlanError::NotApproved {
            status: other.as_str(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_only_when_approved() {
        assert!(can_run_automatic(CoveragePlanStatus::Approved).is_ok());
        assert!(can_run_automatic(CoveragePlanStatus::Draft).is_err());
        assert!(can_run_automatic(CoveragePlanStatus::Archived).is_err());
    }

    #[test]
    fn approve_draft_only() {
        assert_eq!(
            approve_plan(CoveragePlanStatus::Draft).unwrap(),
            CoveragePlanStatus::Approved
        );
        assert!(approve_plan(CoveragePlanStatus::Approved).is_err());
    }
}
