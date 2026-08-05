//! Presupuesto y gasto por conector (incl. gratuitos).
//! Costes en centavos para evitar floats. 0 unit cost = free.

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::integrations::config::IntegrationConfig;

/// Ledger de un conector (image o script AI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectorLedger {
    pub provider_id: String,
    /// Coste estimado por unidad (imagen o llamada needs) en centavos.
    pub unit_cost_cents: u64,
    /// Tope de presupuesto del periodo en centavos. **0 = sin tope**.
    pub budget_limit_cents: u64,
    /// Gastado en el periodo (centavos).
    pub spent_cents: u64,
    /// Cupo gratuito (unidades). **0** = sin cupo finito (ilimitado si is_free / unit=0).
    pub free_quota: u64,
    /// Unidades gratuitas ya usadas.
    pub free_used: u64,
    pub currency: String,
    /// `month` | `total`
    pub period: String,
    /// true si el conector se considera free (unit_cost 0 o free tier).
    pub is_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectorBudgetDto {
    pub provider_id: String,
    pub unit_cost_cents: u64,
    pub budget_limit_cents: u64,
    pub spent_cents: u64,
    /// None = unlimited budget.
    pub available_budget_cents: Option<u64>,
    pub free_quota: u64,
    pub free_used: u64,
    /// None = unlimited free / N/A.
    pub free_remaining: Option<u64>,
    pub currency: String,
    pub period: String,
    pub is_free: bool,
    pub can_afford_one: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorBudgetUpdate {
    pub provider_id: String,
    pub unit_cost_cents: Option<u64>,
    pub budget_limit_cents: Option<u64>,
    pub free_quota: Option<u64>,
    pub period: Option<String>,
    pub is_free: Option<bool>,
    /// Si true, resetea spent y free_used.
    pub reset_usage: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct UsageEvent {
    pub provider_id: String,
    pub units: u64,
    pub cost_cents: u64,
    pub used_free: u64,
}

impl ConnectorLedger {
    pub fn default_for(provider_id: &str) -> Self {
        match provider_id {
            "stub" | "heuristic" | "omniroute" => Self {
                provider_id: provider_id.into(),
                unit_cost_cents: 0,
                budget_limit_cents: 0,
                spent_cents: 0,
                // omniroute: free tiers behind gateway; track usage units even at 0¢
                free_quota: if provider_id == "omniroute" { 500 } else { 0 },
                free_used: 0,
                currency: "USD".into(),
                period: "month".into(),
                is_free: true,
            },
            "spacexai" | "spacexai-image" => Self {
                provider_id: provider_id.into(),
                unit_cost_cents: 2,        // placeholder estimate
                budget_limit_cents: 2_000, // $20 default cap
                spent_cents: 0,
                free_quota: 0,
                free_used: 0,
                currency: "USD".into(),
                period: "month".into(),
                is_free: false,
            },
            "openai-image" => Self {
                provider_id: provider_id.into(),
                unit_cost_cents: 4,
                budget_limit_cents: 2_000,
                spent_cents: 0,
                free_quota: 0,
                free_used: 0,
                currency: "USD".into(),
                period: "month".into(),
                is_free: false,
            },
            "stability" => Self {
                provider_id: provider_id.into(),
                unit_cost_cents: 3,
                budget_limit_cents: 1_500,
                spent_cents: 0,
                free_quota: 25, // ejemplo free tier
                free_used: 0,
                currency: "USD".into(),
                period: "month".into(),
                is_free: false,
            },
            _ => Self {
                provider_id: provider_id.into(),
                unit_cost_cents: 1,
                budget_limit_cents: 1_000,
                spent_cents: 0,
                free_quota: 0,
                free_used: 0,
                currency: "USD".into(),
                period: "month".into(),
                is_free: false,
            },
        }
    }

    pub fn available_budget_cents(&self) -> Option<u64> {
        if self.budget_limit_cents == 0 {
            None
        } else {
            Some(self.budget_limit_cents.saturating_sub(self.spent_cents))
        }
    }

    pub fn free_remaining(&self) -> Option<u64> {
        if self.is_free && self.free_quota == 0 {
            // Unlimited free (stub / heuristic)
            None
        } else if self.free_quota == 0 {
            Some(0)
        } else {
            Some(self.free_quota.saturating_sub(self.free_used))
        }
    }

    pub fn can_afford(&self, units: u64) -> bool {
        if units == 0 {
            return true;
        }
        // Unlimited free
        if self.is_free && self.unit_cost_cents == 0 && self.free_quota == 0 {
            return true;
        }
        let free_rem = if self.free_quota > 0 {
            self.free_quota.saturating_sub(self.free_used)
        } else if self.is_free && self.unit_cost_cents == 0 {
            return true;
        } else {
            0
        };
        let paid_units = units.saturating_sub(free_rem);
        if paid_units == 0 {
            return true;
        }
        let need = self.unit_cost_cents.saturating_mul(paid_units);
        if self.budget_limit_cents == 0 {
            return true; // no cap
        }
        self.spent_cents.saturating_add(need) <= self.budget_limit_cents
    }

    pub fn to_dto(&self) -> ConnectorBudgetDto {
        ConnectorBudgetDto {
            provider_id: self.provider_id.clone(),
            unit_cost_cents: self.unit_cost_cents,
            budget_limit_cents: self.budget_limit_cents,
            spent_cents: self.spent_cents,
            available_budget_cents: self.available_budget_cents(),
            free_quota: self.free_quota,
            free_used: self.free_used,
            free_remaining: self.free_remaining(),
            currency: self.currency.clone(),
            period: self.period.clone(),
            is_free: self.is_free || self.unit_cost_cents == 0,
            can_afford_one: self.can_afford(1),
        }
    }

    pub fn apply_usage(&mut self, units: u64) -> UsageEvent {
        let free_rem = if self.free_quota > 0 {
            self.free_quota.saturating_sub(self.free_used)
        } else {
            0
        };
        let used_free = units.min(free_rem);
        let paid = units.saturating_sub(used_free);
        let cost = self.unit_cost_cents.saturating_mul(paid);
        self.free_used = self.free_used.saturating_add(used_free);
        self.spent_cents = self.spent_cents.saturating_add(cost);
        UsageEvent {
            provider_id: self.provider_id.clone(),
            units,
            cost_cents: cost,
            used_free,
        }
    }
}

pub fn ensure_default_ledgers(cfg: &mut IntegrationConfig) {
    let ids = [
        "stub",
        "heuristic",
        "spacexai",
        "spacexai-image",
        "openai-image",
        "stability",
        "omniroute",
    ];
    for id in ids {
        if !cfg.connector_ledgers.iter().any(|l| l.provider_id == id) {
            cfg.connector_ledgers.push(ConnectorLedger::default_for(id));
        }
    }
}

pub fn ledger_mut<'a>(
    cfg: &'a mut IntegrationConfig,
    provider_id: &str,
) -> &'a mut ConnectorLedger {
    ensure_default_ledgers(cfg);
    if let Some(pos) = cfg
        .connector_ledgers
        .iter()
        .position(|l| l.provider_id == provider_id)
    {
        return &mut cfg.connector_ledgers[pos];
    }
    cfg.connector_ledgers
        .push(ConnectorLedger::default_for(provider_id));
    cfg.connector_ledgers.last_mut().unwrap()
}

pub fn list_connector_budgets(cfg: &IntegrationConfig) -> Vec<ConnectorBudgetDto> {
    let mut cfg = cfg.clone();
    ensure_default_ledgers(&mut cfg);
    cfg.connector_ledgers.iter().map(|l| l.to_dto()).collect()
}

pub fn update_connector_budget(
    cfg: &mut IntegrationConfig,
    u: ConnectorBudgetUpdate,
) -> Result<ConnectorBudgetDto, AppError> {
    if u.provider_id.trim().is_empty() {
        return Err(AppError::Validation("provider_id vacío".into()));
    }
    let led = ledger_mut(cfg, &u.provider_id);
    if let Some(v) = u.unit_cost_cents {
        led.unit_cost_cents = v;
        led.is_free = v == 0;
    }
    if let Some(v) = u.budget_limit_cents {
        led.budget_limit_cents = v;
    }
    if let Some(v) = u.free_quota {
        led.free_quota = v;
    }
    if let Some(v) = u.period {
        led.period = v;
    }
    if let Some(v) = u.is_free {
        led.is_free = v;
        if v {
            led.unit_cost_cents = 0;
        }
    }
    if u.reset_usage == Some(true) {
        led.spent_cents = 0;
        led.free_used = 0;
    }
    Ok(led.to_dto())
}

/// Record usage after a successful generate / script call.
pub fn record_usage(
    cfg: &mut IntegrationConfig,
    provider_id: &str,
    units: u64,
) -> Result<UsageEvent, AppError> {
    let led = ledger_mut(cfg, provider_id);
    if !led.can_afford(units) {
        return Err(AppError::Validation(format!(
            "presupuesto insuficiente en conector `{provider_id}` \
             (spent={}¢ limit={}¢ free_used={}/{})",
            led.spent_cents, led.budget_limit_cents, led.free_used, led.free_quota
        )));
    }
    Ok(led.apply_usage(units))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_unlimited_can_afford() {
        let led = ConnectorLedger::default_for("stub");
        assert!(led.can_afford(100));
        assert!(led.is_free);
        assert_eq!(led.available_budget_cents(), None);
    }

    #[test]
    fn paid_respects_budget() {
        let mut led = ConnectorLedger::default_for("openai-image");
        led.budget_limit_cents = 10;
        led.unit_cost_cents = 4;
        assert!(led.can_afford(2)); // 8¢
        assert!(!led.can_afford(3)); // 12¢
        led.apply_usage(2);
        assert_eq!(led.spent_cents, 8);
        assert!(!led.can_afford(1));
    }

    #[test]
    fn free_quota_then_paid() {
        let mut led = ConnectorLedger::default_for("stability");
        led.free_quota = 2;
        led.free_used = 0;
        led.unit_cost_cents = 3;
        led.budget_limit_cents = 10;
        let ev = led.apply_usage(3);
        assert_eq!(ev.used_free, 2);
        assert_eq!(ev.cost_cents, 3);
        assert_eq!(led.free_used, 2);
        assert_eq!(led.spent_cents, 3);
    }
}
