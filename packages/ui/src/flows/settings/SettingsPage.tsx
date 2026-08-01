import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  invokeGetAppPaths,
  invokeGetIntegrationConfig,
  invokeGetSettings,
  invokeListImageProviders,
  invokeListScriptAiProviders,
  invokeSetMediaRoot,
  invokeUpdateIntegrationConfig,
  type AppPathsDto,
  type ConnectorBudgetDto,
  type ImageProvider,
  type IntegrationConfigDto,
  type ScriptAiProviderInfo,
  type SettingsDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable" | "error";

export function SettingsPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [paths, setPaths] = useState<AppPathsDto | null>(null);
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [mediaRootInput, setMediaRootInput] = useState("");
  const [integrations, setIntegrations] = useState<IntegrationConfigDto | null>(null);
  const [scriptAi, setScriptAi] = useState<ScriptAiProviderInfo[]>([]);
  const [imageProviders, setImageProviders] = useState<ImageProvider[]>([]);
  const [xaiKey, setXaiKey] = useState("");
  const [openaiKey, setOpenaiKey] = useState("");
  const [stabilityKey, setStabilityKey] = useState("");
  const [omniKey, setOmniKey] = useState("");
  const [budgetEdits, setBudgetEdits] = useState<
    Record<string, { budget_limit_cents: string; unit_cost_cents: string; free_quota: string }>
  >({});
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [p, s, integ, scripts, images] = await Promise.all([
        invokeGetAppPaths(),
        invokeGetSettings(),
        invokeGetIntegrationConfig(),
        invokeListScriptAiProviders(),
        invokeListImageProviders(),
      ]);
      setPaths(p);
      setSettings(s);
      setMediaRootInput(s.media_root);
      setIntegrations(integ);
      setScriptAi(scripts);
      setImageProviders(images);
      const edits: Record<
        string,
        { budget_limit_cents: string; unit_cost_cents: string; free_quota: string }
      > = {};
      for (const b of integ.connector_budgets ?? []) {
        edits[b.provider_id] = {
          budget_limit_cents: String(b.budget_limit_cents),
          unit_cost_cents: String(b.unit_cost_cents),
          free_quota: String(b.free_quota),
        };
      }
      setBudgetEdits(edits);
      setLoad("ready");
    } catch {
      setLoad("unavailable");
      setMessage(
        "IPC no disponible (abre la app con pnpm dev / Tauri). En browser puro solo se ve el placeholder.",
      );
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function onSaveMedia(e: FormEvent) {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      const s = await invokeSetMediaRoot(mediaRootInput.trim());
      setSettings(s);
      setMediaRootInput(s.media_root);
      setMessage("media_root guardado.");
      const p = await invokeGetAppPaths();
      setPaths(p);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setSaving(false);
    }
  }

  async function onSaveIntegrations(e: FormEvent) {
    e.preventDefault();
    if (saving || !integrations) return;
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      const connector_budget_updates = Object.entries(budgetEdits).map(([provider_id, e]) => ({
        provider_id,
        budget_limit_cents: Number(e.budget_limit_cents) || 0,
        unit_cost_cents: Number(e.unit_cost_cents) || 0,
        free_quota: Number(e.free_quota) || 0,
        is_free: (Number(e.unit_cost_cents) || 0) === 0,
      }));
      const dto = await invokeUpdateIntegrationConfig({
        script_ai_provider: integrations.script_ai_provider,
        default_image_provider: integrations.default_image_provider,
        enabled_image_providers: integrations.enabled_image_providers,
        xai_api_key: xaiKey.trim() ? xaiKey.trim() : null,
        openai_api_key: openaiKey.trim() ? openaiKey.trim() : null,
        stability_api_key: stabilityKey.trim() ? stabilityKey.trim() : null,
        omniroute_base_url: integrations.omniroute_base_url,
        omniroute_api_key: omniKey.trim() ? omniKey.trim() : null,
        omniroute_image_model: integrations.omniroute_image_model,
        omniroute_chat_model: integrations.omniroute_chat_model,
        omniroute_prefer_free: integrations.omniroute_prefer_free,
        connector_budget_updates,
      });
      setIntegrations(dto);
      setXaiKey("");
      setOpenaiKey("");
      setStabilityKey("");
      setOmniKey("");
      const [scripts, images] = await Promise.all([
        invokeListScriptAiProviders(),
        invokeListImageProviders(),
      ]);
      setScriptAi(scripts);
      setImageProviders(images);
      setMessage(
        "Integraciones guardadas (local). Con keys puedes elegir provider; HTTP real se conecta en adapters.",
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setSaving(false);
    }
  }

  function toggleImageProvider(id: string) {
    if (!integrations) return;
    const set = new Set(integrations.enabled_image_providers);
    if (id === "stub") return; // always on
    if (set.has(id)) set.delete(id);
    else set.add(id);
    setIntegrations({
      ...integrations,
      enabled_image_providers: Array.from(set),
    });
  }

  return (
    <section className="station">
      <header>
        <h2>Settings</h2>
        <p>
          Paths locales + integraciones (IA guion / providers de imagen). Solo falta conectar HTTP
          de cada API y elegir en esta pantalla.
        </p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card">
          <p>Cargando configuración…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card">
          <h3>Modo UI-only</h3>
          <p>{message}</p>
        </div>
      ) : null}

      {load === "ready" && paths && settings && integrations ? (
        <>
          <div className="placeholder-card">
            <h3>Rutas de la aplicación</h3>
            <ul>
              <li>
                <strong>App data:</strong> <code>{paths.app_data_root}</code>
              </li>
              <li>
                <strong>SQLite:</strong> <code>{paths.db_path}</code>
              </li>
              <li>
                <strong>Exports:</strong> <code>{paths.exports_dir}</code>
              </li>
            </ul>

            <form onSubmit={onSaveMedia} style={{ marginTop: "1.25rem" }}>
              <label htmlFor="media_root" style={{ display: "block", marginBottom: "0.35rem" }}>
                Media root
              </label>
              <input
                id="media_root"
                value={mediaRootInput}
                onChange={(ev) => setMediaRootInput(ev.target.value)}
                style={inputStyle}
                autoComplete="off"
                spellCheck={false}
              />
              <button type="submit" disabled={saving} style={btnStyle}>
                Guardar media root
              </button>
            </form>
          </div>

          <div className="placeholder-card" style={{ marginTop: "1rem" }}>
            <h3>Presupuesto y gasto por conector</h3>
            <p style={{ fontSize: "0.9rem", color: "var(--text-muted)" }}>
              Aquí ves y editas límites (centavos: 100 = $1). Free = 0 ¢/unidad o cuota free.
              Si no ves filas, guarda integraciones una vez o reinicia la app.
            </p>
            {(integrations.connector_budgets ?? []).length === 0 ? (
              <p style={{ color: "#fbbf24" }}>
                Aún no hay ledgers. Pulsa «Guardar integraciones» abajo para crearlos.
              </p>
            ) : (
              <div style={{ overflowX: "auto", marginTop: 8 }}>
                <table style={{ width: "100%", fontSize: "0.85rem", borderCollapse: "collapse" }}>
                  <thead>
                    <tr style={{ textAlign: "left", color: "var(--text-muted)" }}>
                      <th style={{ padding: 6 }}>Conector</th>
                      <th style={{ padding: 6 }}>Free</th>
                      <th style={{ padding: 6 }}>¢/ud</th>
                      <th style={{ padding: 6 }}>Límite ¢</th>
                      <th style={{ padding: 6 }}>Gastado</th>
                      <th style={{ padding: 6 }}>Disp.</th>
                      <th style={{ padding: 6 }}>Free us./rest./cuota</th>
                      <th style={{ padding: 6 }} />
                    </tr>
                  </thead>
                  <tbody>
                    {(integrations.connector_budgets ?? []).map((b: ConnectorBudgetDto) => {
                      const ed = budgetEdits[b.provider_id] ?? {
                        budget_limit_cents: String(b.budget_limit_cents),
                        unit_cost_cents: String(b.unit_cost_cents),
                        free_quota: String(b.free_quota),
                      };
                      return (
                        <tr key={b.provider_id} style={{ borderTop: "1px solid var(--border)" }}>
                          <td style={{ padding: 6 }}>
                            <code>{b.provider_id}</code>
                          </td>
                          <td style={{ padding: 6 }}>{b.is_free ? "sí" : "no"}</td>
                          <td style={{ padding: 6 }}>
                            <input
                              value={ed.unit_cost_cents}
                              onChange={(e) =>
                                setBudgetEdits({
                                  ...budgetEdits,
                                  [b.provider_id]: { ...ed, unit_cost_cents: e.target.value },
                                })
                              }
                              style={miniInput}
                            />
                          </td>
                          <td style={{ padding: 6 }}>
                            <input
                              value={ed.budget_limit_cents}
                              onChange={(e) =>
                                setBudgetEdits({
                                  ...budgetEdits,
                                  [b.provider_id]: { ...ed, budget_limit_cents: e.target.value },
                                })
                              }
                              style={miniInput}
                            />
                          </td>
                          <td style={{ padding: 6 }}>{b.spent_cents}¢</td>
                          <td style={{ padding: 6 }}>
                            {b.available_budget_cents == null
                              ? "∞"
                              : `${b.available_budget_cents}¢`}
                          </td>
                          <td style={{ padding: 6 }}>
                            {b.free_used} / {b.free_remaining == null ? "∞" : b.free_remaining} /{" "}
                            <input
                              value={ed.free_quota}
                              onChange={(e) =>
                                setBudgetEdits({
                                  ...budgetEdits,
                                  [b.provider_id]: { ...ed, free_quota: e.target.value },
                                })
                              }
                              style={miniInput}
                            />
                          </td>
                          <td style={{ padding: 6 }}>
                            <button
                              type="button"
                              style={{
                                ...btnStyle,
                                padding: "0.25rem 0.55rem",
                                fontSize: "0.75rem",
                                marginTop: 0,
                              }}
                              onClick={() =>
                                void invokeUpdateIntegrationConfig({
                                  connector_budget_updates: [
                                    { provider_id: b.provider_id, reset_usage: true },
                                  ],
                                }).then((dto) => {
                                  setIntegrations(dto);
                                  setMessage(`Uso reseteado: ${b.provider_id}`);
                                })
                              }
                            >
                              Reset
                            </button>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            )}
            <p style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginTop: 8 }}>
              Los cambios de ¢ se guardan con «Guardar integraciones» más abajo.
            </p>
          </div>

          <div className="placeholder-card" style={{ marginTop: "1rem" }}>
            <h3>Integraciones (API keys y providers)</h3>
            <p style={{ color: "var(--text-muted)", fontSize: "0.9rem" }}>
              Las keys se guardan solo en tu máquina (settings SQLite). No se suben a la nube.
              Deja el campo vacío para no cambiar la key actual.
            </p>

            <form onSubmit={onSaveIntegrations}>
              <h4 style={{ marginTop: "1rem" }}>IA que propone needs desde guion</h4>
              <select
                value={integrations.script_ai_provider}
                onChange={(e) =>
                  setIntegrations({ ...integrations, script_ai_provider: e.target.value })
                }
                style={inputStyle}
              >
                {scriptAi.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} — {p.status}
                  </option>
                ))}
              </select>
              <ul style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>
                {scriptAi.map((p) => (
                  <li key={p.id}>
                    <code>{p.id}</code>: {p.status_detail}
                  </li>
                ))}
              </ul>

              <h4 style={{ marginTop: "1rem" }}>Provider de imagen por defecto</h4>
              <select
                value={integrations.default_image_provider}
                onChange={(e) =>
                  setIntegrations({ ...integrations, default_image_provider: e.target.value })
                }
                style={inputStyle}
              >
                {imageProviders.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} [{p.status ?? "?"}]
                  </option>
                ))}
              </select>

              <h4 style={{ marginTop: "1rem" }}>Providers de imagen habilitados</h4>
              {imageProviders.map((p) => (
                <label
                  key={p.id}
                  style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 6 }}
                >
                  <input
                    type="checkbox"
                    checked={
                      p.id === "stub" ||
                      integrations.enabled_image_providers.includes(p.id)
                    }
                    disabled={p.id === "stub"}
                    onChange={() => toggleImageProvider(p.id)}
                  />
                  <span>
                    <strong>{p.name}</strong> — {p.status ?? "?"}
                    {p.status_detail ? (
                      <span style={{ color: "var(--text-muted)" }}> ({p.status_detail})</span>
                    ) : null}
                  </span>
                </label>
              ))}

              <h4 style={{ marginTop: "1rem" }}>OmniRoute (gateway local)</h4>
              <p style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>
                Ideal para free tiers + Automatic. Arranca OmniRoute y apunta aquí. Manual y
                Automatic usan el mismo provider <code>omniroute</code>.
              </p>
              <label style={labelStyle}>Base URL</label>
              <input
                value={integrations.omniroute_base_url ?? "http://127.0.0.1:20128/v1"}
                onChange={(e) =>
                  setIntegrations({ ...integrations, omniroute_base_url: e.target.value })
                }
                style={inputStyle}
                spellCheck={false}
              />
              <label style={labelStyle}>Image model (p. ej. auto)</label>
              <input
                value={integrations.omniroute_image_model ?? "auto"}
                onChange={(e) =>
                  setIntegrations({ ...integrations, omniroute_image_model: e.target.value })
                }
                style={inputStyle}
                spellCheck={false}
              />
              <label style={labelStyle}>Chat model (needs desde guion)</label>
              <input
                value={integrations.omniroute_chat_model ?? "auto"}
                onChange={(e) =>
                  setIntegrations({ ...integrations, omniroute_chat_model: e.target.value })
                }
                style={inputStyle}
                spellCheck={false}
              />
              <label style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 8 }}>
                <input
                  type="checkbox"
                  checked={integrations.omniroute_prefer_free ?? true}
                  onChange={(e) =>
                    setIntegrations({ ...integrations, omniroute_prefer_free: e.target.checked })
                  }
                />
                Preferir free (rota free stack / omniroute antes que paid)
              </label>
              <label style={labelStyle}>
                OmniRoute API key{" "}
                {integrations.omniroute_api_key_set
                  ? `(set ${integrations.omniroute_api_key_hint})`
                  : "(opcional; muchos free no la piden)"}
              </label>
              <input
                type="password"
                value={omniKey}
                onChange={(e) => setOmniKey(e.target.value)}
                placeholder="opcional"
                style={inputStyle}
                autoComplete="off"
              />

              <h4 style={{ marginTop: "1rem" }}>API keys (otros)</h4>
              <label style={labelStyle}>
                xAI / SpaceXAI {integrations.xai_api_key_set ? `(set ${integrations.xai_api_key_hint})` : "(vacía)"}
              </label>
              <input
                type="password"
                value={xaiKey}
                onChange={(e) => setXaiKey(e.target.value)}
                placeholder="pegar nueva key (opcional)"
                style={inputStyle}
                autoComplete="off"
              />
              <label style={labelStyle}>
                OpenAI{" "}
                {integrations.openai_api_key_set
                  ? `(set ${integrations.openai_api_key_hint})`
                  : "(vacía)"}
              </label>
              <input
                type="password"
                value={openaiKey}
                onChange={(e) => setOpenaiKey(e.target.value)}
                placeholder="pegar nueva key (opcional)"
                style={inputStyle}
                autoComplete="off"
              />
              <label style={labelStyle}>
                Stability{" "}
                {integrations.stability_api_key_set
                  ? `(set ${integrations.stability_api_key_hint})`
                  : "(vacía)"}
              </label>
              <input
                type="password"
                value={stabilityKey}
                onChange={(e) => setStabilityKey(e.target.value)}
                placeholder="pegar nueva key (opcional)"
                style={inputStyle}
                autoComplete="off"
              />

              <button type="submit" disabled={saving} style={{ ...btnStyle, marginTop: 12 }}>
                Guardar integraciones
              </button>
            </form>
          </div>

          {message ? (
            <p className="health" style={{ color: "var(--accent)" }}>
              {message}
            </p>
          ) : null}
          {error ? (
            <p className="health" style={{ color: "#f87171" }}>
              {error}
            </p>
          ) : null}
        </>
      ) : null}
    </section>
  );
}

const inputStyle = {
  width: "100%",
  maxWidth: "40rem",
  padding: "0.5rem 0.65rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  display: "block" as const,
  marginBottom: 8,
};

const labelStyle = {
  display: "block" as const,
  marginTop: 8,
  marginBottom: 4,
  fontSize: "0.85rem",
  color: "var(--text-muted)",
};

const btnStyle = {
  padding: "0.45rem 0.9rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer" as const,
  marginTop: 8,
};

const miniInput = {
  width: "4.5rem",
  padding: "0.25rem",
  borderRadius: 6,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
};
