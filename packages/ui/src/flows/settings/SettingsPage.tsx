import { type CSSProperties, type FormEvent, useCallback, useEffect, useState } from "react";
import {
  invokeGetAppPaths,
  invokeGetIntegrationConfig,
  invokeGetSettings,
  invokeListImageProviders,
  invokeListOmnirouteModels,
  invokeListScriptAiProviders,
  invokeProbeOmniroute,
  invokeSetMediaRoot,
  invokeUpdateIntegrationConfig,
  type AppPathsDto,
  type ConnectorBudgetDto,
  type ImageProvider,
  type IntegrationConfigDto,
  type OmniRouteModelCatalog,
  type OmniRouteProbeResult,
  type ScriptAiProviderInfo,
  type SettingsDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable" | "error";
type SettingsTab = "paths" | "budgets" | "providers" | "keys" | "prompt";

const DEFAULT_NEEDS_PROMPT_HINT =
  "System prompt para OmniRoute/Claude (script → needs). Vacío al guardar = restaurar default.";

/** Ensure current saved value appears in the select even if not in remote list. */
function selectOptions(list: string[], current?: string | null): string[] {
  const cur = (current ?? "").trim();
  const out = [...list];
  if (cur && !out.includes(cur)) out.unshift(cur);
  return out;
}

function formatProbeLog(probe: OmniRouteProbeResult): string {
  const lines = [
    `base: ${probe.base_url}`,
    `summary: ${probe.summary}`,
    "",
    `models: ${probe.models_ok ? "ok" : "fail"}`,
    probe.models_detail,
    "",
    `images: ${probe.images_ok ? "ok" : "fail"}`,
    probe.images_detail,
    "",
    `chat: ${probe.chat_ok ? "ok" : "fail"}`,
    probe.chat_detail,
  ];
  return lines.join("\n");
}

function chip(ok: boolean): CSSProperties {
  return {
    display: "inline-block",
    padding: "0.15rem 0.45rem",
    borderRadius: 6,
    fontSize: "0.72rem",
    fontWeight: 650,
    border: `1px solid ${ok ? "var(--accent)" : "#f87171"}`,
    color: ok ? "var(--accent)" : "#f87171",
    background: ok ? "var(--accent-soft)" : "rgba(248,113,113,0.1)",
  };
}

export function SettingsPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [tab, setTab] = useState<SettingsTab>("paths");
  const [budgetTab, setBudgetTab] = useState(0);
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
  const [probing, setProbing] = useState(false);
  const [probe, setProbe] = useState<OmniRouteProbeResult | null>(null);
  const [omniCatalog, setOmniCatalog] = useState<OmniRouteModelCatalog | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);

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

  const loadOmniModels = useCallback(async () => {
    setLoadingModels(true);
    try {
      const cat = await invokeListOmnirouteModels();
      setOmniCatalog(cat);
    } catch {
      setOmniCatalog(null);
    } finally {
      setLoadingModels(false);
    }
  }, []);

  useEffect(() => {
    if (load === "ready" && tab === "keys") {
      void loadOmniModels();
    }
  }, [load, tab, loadOmniModels]);

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
        needs_system_prompt: integrations.needs_system_prompt ?? null,
        allow_stub_fallback_on_image_error:
          integrations.allow_stub_fallback_on_image_error ?? false,
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

  async function onProbeOmniroute(tryImage: boolean, tryChat: boolean) {
    if (probing) return;
    setProbing(true);
    setError(null);
    setMessage(null);
    try {
      // Persist current URL/models first so probe uses what you see on screen.
      if (integrations) {
        await invokeUpdateIntegrationConfig({
          omniroute_base_url: integrations.omniroute_base_url,
          omniroute_image_model: integrations.omniroute_image_model,
          omniroute_chat_model: integrations.omniroute_chat_model,
          omniroute_prefer_free: integrations.omniroute_prefer_free,
          omniroute_api_key: omniKey.trim() ? omniKey.trim() : null,
        });
      }
      const r = await invokeProbeOmniroute({ tryImage, tryChat });
      setProbe(r);
      setMessage(r.summary);
      // Short toast only — full dump lives in the scrollable log panel (no overlap).
      if (!r.overall_ok) {
        setError("Gateway no alcanzable. Detalle en el panel de log abajo.");
      } else if (tryImage && !r.images_ok) {
        setError("Imagen falló (ver log). Suele faltar provider de imagen en OmniRoute.");
      } else {
        setError(null);
      }
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setProbing(false);
    }
  }

  const budgets = integrations?.connector_budgets ?? [];
  const activeBudget: ConnectorBudgetDto | undefined = budgets[budgetTab];

  return (
    <section className="station">
      <header>
        <h2>Settings</h2>
        <p>Paths + integraciones locales. Sin scroll: pestañas.</p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card fill">
          <p>Cargando configuración…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card fill">
          <h3>Modo UI-only</h3>
          <p>{message}</p>
        </div>
      ) : null}

      {load === "ready" && paths && settings && integrations ? (
        <div className="station-body">
          <div className="tab-strip">
            {(
              [
                ["paths", "Rutas"],
                ["budgets", "Presupuesto"],
                ["providers", "Providers"],
                ["keys", "Keys / Omni"],
                ["prompt", "Prompt needs"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                type="button"
                className={`tab-btn${tab === id ? " active" : ""}`}
                onClick={() => setTab(id)}
              >
                {label}
              </button>
            ))}
          </div>

          <div className="placeholder-card fill">
            {tab === "paths" ? (
              <>
                <h3>Rutas</h3>
                <p style={{ marginBottom: 8 }}>
                  <strong>App data:</strong> <code style={codeSmall}>{paths.app_data_root}</code>
                </p>
                <p style={{ marginBottom: 8 }}>
                  <strong>SQLite:</strong> <code style={codeSmall}>{paths.db_path}</code>
                </p>
                <p style={{ marginBottom: 8 }}>
                  <strong>Exports:</strong> <code style={codeSmall}>{paths.exports_dir}</code>
                </p>
                <form onSubmit={onSaveMedia} style={{ marginTop: 12 }}>
                  <label htmlFor="media_root" style={labelStyle}>
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
              </>
            ) : null}

            {tab === "budgets" ? (
              <>
                <h3>Presupuesto por conector</h3>
                <p style={{ marginBottom: 8, fontSize: "0.82rem" }}>
                  Centavos (100 = $1). Cambios se guardan en «Providers» o «Keys».
                </p>
                {budgets.length === 0 ? (
                  <p style={{ color: "#fbbf24" }}>
                    Sin ledgers. Guarda providers una vez para crearlos.
                  </p>
                ) : (
                  <>
                    <div className="tab-strip">
                      {budgets.map((b, i) => (
                        <button
                          key={b.provider_id}
                          type="button"
                          className={`tab-btn${budgetTab === i ? " active" : ""}`}
                          onClick={() => setBudgetTab(i)}
                        >
                          {b.provider_id}
                        </button>
                      ))}
                    </div>
                    {activeBudget ? (
                      <BudgetEditor
                        b={activeBudget}
                        ed={
                          budgetEdits[activeBudget.provider_id] ?? {
                            budget_limit_cents: String(activeBudget.budget_limit_cents),
                            unit_cost_cents: String(activeBudget.unit_cost_cents),
                            free_quota: String(activeBudget.free_quota),
                          }
                        }
                        onChange={(patch) =>
                          setBudgetEdits({
                            ...budgetEdits,
                            [activeBudget.provider_id]: {
                              ...(budgetEdits[activeBudget.provider_id] ?? {
                                budget_limit_cents: String(activeBudget.budget_limit_cents),
                                unit_cost_cents: String(activeBudget.unit_cost_cents),
                                free_quota: String(activeBudget.free_quota),
                              }),
                              ...patch,
                            },
                          })
                        }
                        onReset={() =>
                          void invokeUpdateIntegrationConfig({
                            connector_budget_updates: [
                              { provider_id: activeBudget.provider_id, reset_usage: true },
                            ],
                          }).then((dto) => {
                            setIntegrations(dto);
                            setMessage(`Uso reseteado: ${activeBudget.provider_id}`);
                          })
                        }
                      />
                    ) : null}
                  </>
                )}
              </>
            ) : null}

            {tab === "providers" ? (
              <form
                onSubmit={onSaveIntegrations}
                style={{ display: "flex", flexDirection: "column", minHeight: 0, flex: 1 }}
              >
                <h3>Providers</h3>
                <label style={labelStyle}>IA de guion → needs</label>
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
                <p style={{ fontSize: "0.78rem", color: "var(--text-muted)", margin: "0 0 6px" }}>
                  Para Claude: elige <strong>omniroute</strong>, arranca el gateway, y en Keys pon
                  el chat model (ej. id Claude que exponga OmniRoute). Fallback = heurística.
                </p>
                {scriptAi.map((p) =>
                  p.id === integrations.script_ai_provider ? (
                    <p
                      key={p.id}
                      style={{ fontSize: "0.78rem", color: "var(--text-muted)", margin: "0 0 8px" }}
                    >
                      {p.status_detail}
                    </p>
                  ) : null,
                )}
                <label style={labelStyle}>Imagen por defecto</label>
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
                <label style={labelStyle}>Habilitados</label>
                <div style={providerGrid}>
                  {imageProviders.map((p) => (
                    <label key={p.id} style={checkRow}>
                      <input
                        type="checkbox"
                        checked={
                          p.id === "stub" || integrations.enabled_image_providers.includes(p.id)
                        }
                        disabled={p.id === "stub"}
                        onChange={() => toggleImageProvider(p.id)}
                      />
                      <span>
                        {p.name} · {p.status ?? "?"}
                      </span>
                    </label>
                  ))}
                </div>
                <label style={{ ...checkRow, marginTop: 12 }}>
                  <input
                    type="checkbox"
                    checked={integrations.allow_stub_fallback_on_image_error === true}
                    onChange={(e) =>
                      setIntegrations({
                        ...integrations,
                        allow_stub_fallback_on_image_error: e.target.checked,
                      })
                    }
                  />
                  Permitir fallback silencioso a stub si falla la imagen remota (no recomendado)
                </label>
                <button type="submit" disabled={saving} style={{ ...btnStyle, marginTop: "auto" }}>
                  Guardar integraciones
                </button>
              </form>
            ) : null}

            {tab === "prompt" ? (
              <form
                onSubmit={onSaveIntegrations}
                style={{ display: "flex", flexDirection: "column", minHeight: 0, flex: 1 }}
              >
                <h3>Prompt needs (system → Claude/OmniRoute)</h3>
                <p style={{ fontSize: "0.8rem", color: "var(--text-muted)", margin: "0 0 6px" }}>
                  {DEFAULT_NEEDS_PROMPT_HINT} Factory envía el guion (+ instrucciones extra) como
                  mensaje user.
                </p>
                <textarea
                  value={integrations.needs_system_prompt ?? ""}
                  onChange={(e) =>
                    setIntegrations({ ...integrations, needs_system_prompt: e.target.value })
                  }
                  style={promptTextarea}
                  spellCheck={false}
                />
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginTop: 8 }}>
                  <button type="submit" disabled={saving} style={btnStyle}>
                    Guardar prompt + integraciones
                  </button>
                  <button
                    type="button"
                    style={btnStyle}
                    onClick={() =>
                      setIntegrations({
                        ...integrations,
                        needs_system_prompt: "",
                      })
                    }
                    title="Vacío se restaura al default al guardar"
                  >
                    Vaciar → default al guardar
                  </button>
                </div>
              </form>
            ) : null}

            {tab === "keys" ? (
              <form
                onSubmit={onSaveIntegrations}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                  flex: 1,
                  overflow: "hidden",
                  gap: 0,
                }}
              >
                <h3 style={{ flexShrink: 0 }}>Keys y OmniRoute</h3>

                {/* Form fields: scroll only if the viewport is very short */}
                <div style={keysFieldsScroll}>
                  <div style={twoCol}>
                    <div>
                      <label style={labelStyle}>OmniRoute base URL</label>
                      <input
                        value={integrations.omniroute_base_url ?? "http://127.0.0.1:20128/v1"}
                        onChange={(e) =>
                          setIntegrations({ ...integrations, omniroute_base_url: e.target.value })
                        }
                        style={inputStyle}
                        spellCheck={false}
                      />
                      <label style={labelStyle}>Image model</label>
                      <select
                        value={integrations.omniroute_image_model ?? "pollinations/flux"}
                        onChange={(e) =>
                          setIntegrations({
                            ...integrations,
                            omniroute_image_model: e.target.value,
                          })
                        }
                        style={inputStyle}
                      >
                        {selectOptions(
                          omniCatalog?.image_models ?? ["pollinations/flux"],
                          integrations.omniroute_image_model,
                        ).map((id) => (
                          <option key={id} value={id}>
                            {id}
                          </option>
                        ))}
                      </select>
                      <label style={labelStyle}>Chat model (needs)</label>
                      <select
                        value={integrations.omniroute_chat_model ?? "auto/best-free"}
                        onChange={(e) =>
                          setIntegrations({
                            ...integrations,
                            omniroute_chat_model: e.target.value,
                          })
                        }
                        style={inputStyle}
                      >
                        {selectOptions(
                          omniCatalog?.chat_models ?? ["auto/best-free", "auto/chat"],
                          integrations.omniroute_chat_model,
                        ).map((id) => (
                          <option key={id} value={id}>
                            {id}
                          </option>
                        ))}
                      </select>
                      <p style={{ fontSize: "0.72rem", color: "var(--text-muted)", margin: "4px 0" }}>
                        {loadingModels
                          ? "Cargando modelos…"
                          : omniCatalog
                            ? omniCatalog.ok
                              ? `Menú: ${omniCatalog.detail}`
                              : `Lista local · ${omniCatalog.detail}`
                            : "Pulsa «Cargar modelos»"}
                      </p>
                      <label style={checkRow}>
                        <input
                          type="checkbox"
                          checked={integrations.omniroute_prefer_free ?? true}
                          onChange={(e) =>
                            setIntegrations({
                              ...integrations,
                              omniroute_prefer_free: e.target.checked,
                            })
                          }
                        />
                        Preferir free
                      </label>
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginTop: 8 }}>
                        <button
                          type="button"
                          disabled={loadingModels || saving}
                          style={btnStyle}
                          onClick={() => void loadOmniModels()}
                        >
                          {loadingModels ? "Cargando…" : "Cargar modelos"}
                        </button>
                        <button
                          type="button"
                          disabled={probing || saving}
                          style={btnStyle}
                          onClick={() => void onProbeOmniroute(false, false)}
                        >
                          {probing ? "Probando…" : "Probar models"}
                        </button>
                        <button
                          type="button"
                          disabled={probing || saving}
                          style={btnStyle}
                          onClick={() => void onProbeOmniroute(true, true)}
                        >
                          Probar e2e
                        </button>
                      </div>
                    </div>
                    <div>
                      <label style={labelStyle}>
                        OmniRoute key{" "}
                        {integrations.omniroute_api_key_set
                          ? `(set ${integrations.omniroute_api_key_hint})`
                          : "(vacía = pegar nueva)"}
                      </label>
                      <input
                        type="password"
                        value={omniKey}
                        onChange={(e) => setOmniKey(e.target.value)}
                        style={inputStyle}
                        autoComplete="off"
                        placeholder={
                          integrations.omniroute_api_key_set ? "dejar vacío = no cambiar" : ""
                        }
                      />
                      <label style={labelStyle}>
                        xAI{" "}
                        {integrations.xai_api_key_set
                          ? `(set ${integrations.xai_api_key_hint})`
                          : ""}
                      </label>
                      <input
                        type="password"
                        value={xaiKey}
                        onChange={(e) => setXaiKey(e.target.value)}
                        style={inputStyle}
                        autoComplete="off"
                      />
                      <label style={labelStyle}>
                        OpenAI{" "}
                        {integrations.openai_api_key_set
                          ? `(set ${integrations.openai_api_key_hint})`
                          : ""}
                      </label>
                      <input
                        type="password"
                        value={openaiKey}
                        onChange={(e) => setOpenaiKey(e.target.value)}
                        style={inputStyle}
                        autoComplete="off"
                      />
                      <label style={labelStyle}>
                        Stability{" "}
                        {integrations.stability_api_key_set
                          ? `(set ${integrations.stability_api_key_hint})`
                          : ""}
                      </label>
                      <input
                        type="password"
                        value={stabilityKey}
                        onChange={(e) => setStabilityKey(e.target.value)}
                        style={inputStyle}
                        autoComplete="off"
                      />
                    </div>
                  </div>
                </div>

                {/* Log panel: only this region scrolls — full text readable, no overlap */}
                {probe ? (
                  <div style={probePanel}>
                    <div style={probeStatusRow}>
                      <span style={chip(probe.models_ok)}>models {probe.models_ok ? "✓" : "✗"}</span>
                      <span style={chip(probe.images_ok)}>images {probe.images_ok ? "✓" : "✗"}</span>
                      <span style={chip(probe.chat_ok)}>chat {probe.chat_ok ? "✓" : "✗"}</span>
                      <span style={{ fontSize: "0.72rem", color: "var(--text-muted)" }}>
                        {probe.base_url}
                      </span>
                    </div>
                    <pre style={probeLogScroll} tabIndex={0}>
                      {formatProbeLog(probe)}
                    </pre>
                  </div>
                ) : null}

                <div style={footerBar}>
                  <button type="submit" disabled={saving} style={{ ...btnStyle, marginTop: 0 }}>
                    Guardar integraciones
                  </button>
                </div>
              </form>
            ) : null}
          </div>

          {message ? (
            <p className="health msg-ok" style={toastLine} title={message}>
              {message}
            </p>
          ) : null}
          {error ? (
            <p className="health msg-err" style={toastLine} title={error}>
              {error}
            </p>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}

function BudgetEditor({
  b,
  ed,
  onChange,
  onReset,
}: {
  b: ConnectorBudgetDto;
  ed: { budget_limit_cents: string; unit_cost_cents: string; free_quota: string };
  onChange: (p: Partial<typeof ed>) => void;
  onReset: () => void;
}) {
  return (
    <div style={budgetGrid}>
      <p style={{ margin: 0, gridColumn: "1 / -1" }}>
        <code>{b.provider_id}</code> · free: {b.is_free ? "sí" : "no"} · gastado {b.spent_cents}¢ ·
        disp. {b.available_budget_cents == null ? "∞" : `${b.available_budget_cents}¢`} · free{" "}
        {b.free_used}/{b.free_remaining == null ? "∞" : b.free_remaining}
      </p>
      <label style={labelStyle}>
        ¢/ud
        <input
          value={ed.unit_cost_cents}
          onChange={(e) => onChange({ unit_cost_cents: e.target.value })}
          style={inputStyle}
        />
      </label>
      <label style={labelStyle}>
        Límite ¢
        <input
          value={ed.budget_limit_cents}
          onChange={(e) => onChange({ budget_limit_cents: e.target.value })}
          style={inputStyle}
        />
      </label>
      <label style={labelStyle}>
        Free cuota
        <input
          value={ed.free_quota}
          onChange={(e) => onChange({ free_quota: e.target.value })}
          style={inputStyle}
        />
      </label>
      <div style={{ alignSelf: "end" }}>
        <button type="button" style={btnStyle} onClick={onReset}>
          Reset uso
        </button>
      </div>
    </div>
  );
}

const inputStyle = {
  width: "100%",
  maxWidth: "100%",
  padding: "0.4rem 0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  display: "block" as const,
  marginBottom: 4,
};

const labelStyle = {
  display: "block" as const,
  marginTop: 6,
  marginBottom: 3,
  fontSize: "0.8rem",
  color: "var(--text-muted)",
};

const btnStyle = {
  padding: "0.4rem 0.8rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer" as const,
  marginTop: 8,
};

const codeSmall = { fontSize: "0.78rem", wordBreak: "break-all" as const };

const twoCol = {
  display: "grid" as const,
  gridTemplateColumns: "1fr 1fr",
  gap: "0.75rem 1.25rem",
  minHeight: 0,
};

const keysFieldsScroll = {
  flex: "1 1 auto" as const,
  minHeight: 0,
  overflowY: "auto" as const,
  overflowX: "hidden" as const,
  paddingRight: 4,
};

const probePanel = {
  flexShrink: 0 as const,
  marginTop: 8,
  border: "1px solid var(--border)",
  borderRadius: 10,
  background: "rgba(0,0,0,0.25)",
  display: "flex" as const,
  flexDirection: "column" as const,
  maxHeight: 140,
  minHeight: 72,
  overflow: "hidden" as const,
};

const probeStatusRow = {
  display: "flex" as const,
  flexWrap: "wrap" as const,
  gap: 6,
  alignItems: "center" as const,
  padding: "0.4rem 0.55rem",
  borderBottom: "1px solid var(--border)",
  flexShrink: 0 as const,
};

const probeLogScroll = {
  margin: 0,
  padding: "0.45rem 0.55rem",
  flex: "1 1 auto" as const,
  minHeight: 0,
  overflowY: "auto" as const,
  overflowX: "auto" as const,
  fontSize: "0.72rem",
  lineHeight: 1.4,
  fontFamily: "ui-monospace, Consolas, monospace",
  color: "var(--text-muted)",
  whiteSpace: "pre-wrap" as const,
  wordBreak: "break-word" as const,
};

const footerBar = {
  flexShrink: 0 as const,
  marginTop: 8,
  paddingTop: 8,
  borderTop: "1px solid var(--border)",
  display: "flex" as const,
  alignItems: "center" as const,
  gap: 8,
};

const toastLine = {
  flexShrink: 0 as const,
  margin: "0.35rem 0 0",
  overflow: "hidden" as const,
  textOverflow: "ellipsis" as const,
  whiteSpace: "nowrap" as const,
};

const providerGrid = {
  display: "grid" as const,
  gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
  gap: 6,
  marginBottom: 8,
};

const checkRow = {
  display: "flex" as const,
  gap: 8,
  alignItems: "center" as const,
  fontSize: "0.85rem",
  marginTop: 6,
};

const budgetGrid = {
  display: "grid" as const,
  gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))",
  gap: 10,
  marginTop: 8,
};

const promptTextarea = {
  flex: "1 1 auto" as const,
  minHeight: 0,
  width: "100%",
  fontFamily: "ui-monospace, Consolas, monospace",
  fontSize: "0.78rem",
  padding: "0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  resize: "none" as const,
};
