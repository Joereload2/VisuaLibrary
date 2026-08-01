use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::integrations::budget::{
    ensure_default_ledgers, list_connector_budgets, update_connector_budget, ConnectorBudgetDto,
    ConnectorBudgetUpdate, ConnectorLedger,
};
use crate::ports::settings::SettingsStore;
use crate::settings::keys;

/// Local integration config (desktop). Secrets stay on machine only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationConfig {
    /// `heuristic` | `spacexai` (others: register + connect).
    pub script_ai_provider: String,
    /// Preferred image provider id for Manual when need has no override.
    pub default_image_provider: String,
    /// Image provider ids the user enabled (subset of catalog).
    pub enabled_image_providers: Vec<String>,
    /// API keys (empty string = not set). Never log these.
    pub xai_api_key: String,
    pub openai_api_key: String,
    pub stability_api_key: String,
    /// OmniRoute gateway (https://github.com/diegosouzapw/OmniRoute)
    #[serde(default = "default_omniroute_base")]
    pub omniroute_base_url: String,
    #[serde(default)]
    pub omniroute_api_key: String,
    /// Image model id as seen by OmniRoute (e.g. auto, pollinations/..., free stack).
    #[serde(default = "default_omniroute_image_model")]
    pub omniroute_image_model: String,
    #[serde(default = "default_omniroute_chat_model")]
    pub omniroute_chat_model: String,
    /// Prefer free-tier routing when selecting providers / models.
    #[serde(default = "default_true")]
    pub omniroute_prefer_free: bool,
    /// Presupuesto y gasto por conector (incl. free).
    #[serde(default)]
    pub connector_ledgers: Vec<ConnectorLedger>,
}

fn default_omniroute_base() -> String {
    "http://127.0.0.1:20128/v1".into()
}
fn default_omniroute_image_model() -> String {
    "auto".into()
}
fn default_omniroute_chat_model() -> String {
    "auto".into()
}
fn default_true() -> bool {
    true
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        let mut s = Self {
            script_ai_provider: "heuristic".into(),
            default_image_provider: "stub".into(),
            enabled_image_providers: vec!["stub".into(), "omniroute".into()],
            xai_api_key: String::new(),
            openai_api_key: String::new(),
            stability_api_key: String::new(),
            omniroute_base_url: default_omniroute_base(),
            omniroute_api_key: String::new(),
            omniroute_image_model: default_omniroute_image_model(),
            omniroute_chat_model: default_omniroute_chat_model(),
            omniroute_prefer_free: true,
            connector_ledgers: vec![],
        };
        ensure_default_ledgers(&mut s);
        s
    }
}

/// Public DTO for UI: keys masked except when writing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationConfigDto {
    pub script_ai_provider: String,
    pub default_image_provider: String,
    pub enabled_image_providers: Vec<String>,
    pub xai_api_key_set: bool,
    pub openai_api_key_set: bool,
    pub stability_api_key_set: bool,
    pub xai_api_key_hint: String,
    pub openai_api_key_hint: String,
    pub stability_api_key_hint: String,
    pub omniroute_base_url: String,
    pub omniroute_api_key_set: bool,
    pub omniroute_api_key_hint: String,
    pub omniroute_image_model: String,
    pub omniroute_chat_model: String,
    pub omniroute_prefer_free: bool,
    pub connector_budgets: Vec<ConnectorBudgetDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfigUpdate {
    pub script_ai_provider: Option<String>,
    pub default_image_provider: Option<String>,
    pub enabled_image_providers: Option<Vec<String>>,
    pub xai_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub stability_api_key: Option<String>,
    pub omniroute_base_url: Option<String>,
    pub omniroute_api_key: Option<String>,
    pub omniroute_image_model: Option<String>,
    pub omniroute_chat_model: Option<String>,
    pub omniroute_prefer_free: Option<bool>,
    pub connector_budget_updates: Option<Vec<ConnectorBudgetUpdate>>,
}

fn hint(key: &str) -> String {
    let t = key.trim();
    if t.is_empty() {
        String::new()
    } else if t.len() <= 4 {
        "••••".into()
    } else {
        format!("••••{}", &t[t.len() - 4..])
    }
}

impl IntegrationConfig {
    pub fn to_dto(&self) -> IntegrationConfigDto {
        IntegrationConfigDto {
            script_ai_provider: self.script_ai_provider.clone(),
            default_image_provider: self.default_image_provider.clone(),
            enabled_image_providers: self.enabled_image_providers.clone(),
            xai_api_key_set: !self.xai_api_key.trim().is_empty(),
            openai_api_key_set: !self.openai_api_key.trim().is_empty(),
            stability_api_key_set: !self.stability_api_key.trim().is_empty(),
            xai_api_key_hint: hint(&self.xai_api_key),
            openai_api_key_hint: hint(&self.openai_api_key),
            stability_api_key_hint: hint(&self.stability_api_key),
            omniroute_base_url: self.omniroute_base_url.clone(),
            omniroute_api_key_set: !self.omniroute_api_key.trim().is_empty(),
            omniroute_api_key_hint: hint(&self.omniroute_api_key),
            omniroute_image_model: self.omniroute_image_model.clone(),
            omniroute_chat_model: self.omniroute_chat_model.clone(),
            omniroute_prefer_free: self.omniroute_prefer_free,
            connector_budgets: list_connector_budgets(self),
        }
    }

    pub fn apply_update(&mut self, u: IntegrationConfigUpdate) -> Result<(), AppError> {
        if let Some(v) = u.script_ai_provider {
            self.script_ai_provider = v;
        }
        if let Some(v) = u.default_image_provider {
            self.default_image_provider = v;
        }
        if let Some(v) = u.enabled_image_providers {
            self.enabled_image_providers = v;
        }
        if let Some(v) = u.xai_api_key {
            self.xai_api_key = v;
        }
        if let Some(v) = u.openai_api_key {
            self.openai_api_key = v;
        }
        if let Some(v) = u.stability_api_key {
            self.stability_api_key = v;
        }
        if let Some(v) = u.omniroute_base_url {
            self.omniroute_base_url = v;
        }
        if let Some(v) = u.omniroute_api_key {
            self.omniroute_api_key = v;
        }
        if let Some(v) = u.omniroute_image_model {
            self.omniroute_image_model = v;
        }
        if let Some(v) = u.omniroute_chat_model {
            self.omniroute_chat_model = v;
        }
        if let Some(v) = u.omniroute_prefer_free {
            self.omniroute_prefer_free = v;
        }
        if !self.enabled_image_providers.iter().any(|p| p == "stub") {
            self.enabled_image_providers.push("stub".into());
        }
        if let Some(updates) = u.connector_budget_updates {
            for bu in updates {
                update_connector_budget(self, bu)?;
            }
        }
        ensure_default_ledgers(self);
        Ok(())
    }
}

pub fn load_integration_config(store: &impl SettingsStore) -> Result<IntegrationConfig, AppError> {
    let mut cfg = match store.get_json(keys::INTEGRATIONS)? {
        None => IntegrationConfig::default(),
        Some(raw) => serde_json::from_str(&raw).map_err(|e| {
            AppError::Storage(format!("settings.integrations JSON inválido: {e}"))
        })?,
    };
    ensure_default_ledgers(&mut cfg);
    Ok(cfg)
}

pub fn save_integration_config(
    store: &impl SettingsStore,
    cfg: &IntegrationConfig,
) -> Result<IntegrationConfigDto, AppError> {
    let json = serde_json::to_string(cfg)
        .map_err(|e| AppError::Internal(format!("serialize integrations: {e}")))?;
    store.set_json(keys::INTEGRATIONS, &json)?;
    Ok(cfg.to_dto())
}

pub fn get_integration_config_dto(
    store: &impl SettingsStore,
) -> Result<IntegrationConfigDto, AppError> {
    Ok(load_integration_config(store)?.to_dto())
}

pub fn update_integration_config(
    store: &impl SettingsStore,
    update: IntegrationConfigUpdate,
) -> Result<IntegrationConfigDto, AppError> {
    let mut cfg = load_integration_config(store)?;
    cfg.apply_update(update)?;
    let allowed_script = ["heuristic", "spacexai", "omniroute"];
    if !allowed_script.contains(&cfg.script_ai_provider.as_str()) {
        return Err(AppError::Validation(format!(
            "script_ai_provider desconocido: {}",
            cfg.script_ai_provider
        )));
    }
    save_integration_config(store, &cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct Mem(Mutex<HashMap<String, String>>);
    impl SettingsStore for Mem {
        fn get_json(&self, key: &str) -> Result<Option<String>, AppError> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }
        fn set_json(&self, key: &str, value: &str) -> Result<(), AppError> {
            self.0.lock().unwrap().insert(key.into(), value.into());
            Ok(())
        }
    }

    #[test]
    fn roundtrip_and_mask() {
        let store = Mem(Mutex::new(HashMap::new()));
        let dto = update_integration_config(
            &store,
            IntegrationConfigUpdate {
                script_ai_provider: Some("spacexai".into()),
                default_image_provider: Some("stub".into()),
                enabled_image_providers: Some(vec!["stub".into(), "spacexai-image".into()]),
                xai_api_key: Some("sk-test-abcdef12".into()),
                openai_api_key: None,
                stability_api_key: None,
                omniroute_base_url: Some("http://127.0.0.1:20128/v1".into()),
                omniroute_api_key: None,
                omniroute_image_model: Some("auto".into()),
                omniroute_chat_model: None,
                omniroute_prefer_free: Some(true),
                connector_budget_updates: None,
            },
        )
        .unwrap();
        assert!(dto.xai_api_key_set);
        assert!(!dto.connector_budgets.is_empty());
        assert!(dto.omniroute_prefer_free);
        let stub = dto
            .connector_budgets
            .iter()
            .find(|b| b.provider_id == "stub")
            .unwrap();
        assert!(stub.is_free);
        assert!(stub.can_afford_one);
        assert!(dto
            .connector_budgets
            .iter()
            .any(|b| b.provider_id == "omniroute"));
    }
}
