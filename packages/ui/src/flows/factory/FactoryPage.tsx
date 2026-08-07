import { type CSSProperties, useEffect, useState } from "react";
import { NavLink, Navigate, Route, Routes } from "react-router-dom";
import {
  invokeListImageProviders,
  invokeListPackages,
  invokeProposeNeedsFromPackage,
  invokePreviewManualBatch,
  invokeProposeNeedsFromScript,
  invokeSubmitManualBatch,
  invokeWritePackageImages,
  type ImageProvider,
  type ManualBatchPreview,
  type ManualNeed,
  type PackageSummary,
} from "../../shared/ipc/client";
import { AutomaticFactory } from "./AutomaticFactory";
import { ConnectionBanner } from "./ConnectionBanner";

const SAMPLE_SCRIPT = `Hoy hablamos de fotosíntesis en las plantas.

Las hojas capturan la luz solar gracias a la clorofila.

El dióxido de carbono entra por los estomas y se transforma en glucosa.

Como resultado se libera oxígeno, esencial para la vida en el planeta.

En la práctica de laboratorio observamos el color verde como señal de clorofila activa.`;

type Step = "script" | "needs" | "run";
type ScriptTab = "guion" | "instrucciones";

function ManualFactory() {
  const [step, setStep] = useState<Step>("script");
  const [scriptTab, setScriptTab] = useState<ScriptTab>("guion");
  const [needTab, setNeedTab] = useState(0);
  const [resultTab, setResultTab] = useState(0);
  const [script, setScript] = useState(SAMPLE_SCRIPT);
  const [scriptInstructions, setScriptInstructions] = useState("");
  const [needs, setNeeds] = useState<ManualNeed[]>([]);
  const [providers, setProviders] = useState<ImageProvider[]>([]);
  const [proposeNotes, setProposeNotes] = useState<string | null>(null);
  const [preview, setPreview] = useState<ManualBatchPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [packages, setPackages] = useState<PackageSummary[]>([]);
  const [selectedPackagePath, setSelectedPackagePath] = useState("");
  const [activePackagePath, setActivePackagePath] = useState<string | null>(null);

  useEffect(() => {
    void invokeListImageProviders()
      .then(setProviders)
      .catch(() => setProviders([]));
    void invokeListPackages()
      .then((list) => {
        setPackages(list);
        if (list[0] && !selectedPackagePath) setSelectedPackagePath(list[0].path);
      })
      .catch(() => setPackages([]));
  }, []);

  useEffect(() => {
    if (needTab >= needs.length && needs.length > 0) {
      setNeedTab(needs.length - 1);
    }
  }, [needs.length, needTab]);

  useEffect(() => {
    const n = preview?.results.length ?? 0;
    if (resultTab >= n && n > 0) setResultTab(n - 1);
  }, [preview?.results.length, resultTab]);

  async function onPropose() {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      // Instrucciones del tab se envían al chat (OmniRoute/Claude) si script_ai = omniroute.
      const r = await invokeProposeNeedsFromScript(
        script,
        undefined,
        scriptInstructions || null,
      );
      setNeeds(r.needs);
      setScriptInstructions(r.script_instructions);
      setProposeNotes(`${r.method}: ${r.notes}`);
      setPreview(null);
      setNeedTab(0);
      setStep("needs");
      const viaOmni = r.method.startsWith("omniroute_chat");
      const viaFallback = r.method.includes("fallback");
      if (viaOmni) {
        setMessage(`Needs vía OmniRoute/Claude: ${r.needs.length}. Revisa pestañas y edita.`);
        setError(null);
      } else if (viaFallback) {
        setMessage(`Needs: ${r.needs.length} (heurística de respaldo).`);
        setError(
          `OmniRoute/chat no se usó: ${r.method}. ${r.notes.slice(0, 220)}${r.notes.length > 220 ? "…" : ""}`,
        );
      } else {
        setMessage(`Needs (heurística): ${r.needs.length}. Revisa y edita.`);
      }
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onOpenPackage() {
    if (busy || !selectedPackagePath) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const r = await invokeProposeNeedsFromPackage(selectedPackagePath);
      setNeeds(r.needs);
      setScriptInstructions(r.script_instructions);
      setProposeNotes(`${r.method}: ${r.notes}`);
      setScript(
        r.needs
          .map((n) => n.script_excerpt || "")
          .filter(Boolean)
          .join("\n\n") || script,
      );
      setActivePackagePath(selectedPackagePath);
      setPreview(null);
      setNeedTab(0);
      setStep("needs");
      setMessage(
        `Package abierto: ${r.needs.length} needs (beats). Revisa, genera y usa «Escribir al package».`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onWriteToPackage() {
    if (busy) return;
    const pkgPath =
      activePackagePath ||
      needs.find((n) => n.package_path)?.package_path ||
      selectedPackagePath;
    if (!pkgPath) {
      setError("No hay package activo. Usa «Abrir package» primero.");
      return;
    }
    if (!preview?.results?.length) {
      setError("Primero Preview o Submit para tener assets generados.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const included = includedNeeds();
      const items = preview.results
        .map((res) => {
          const need = included[res.index];
          const beatId = need?.beat_id || `b${String(res.index + 1).padStart(2, "0")}`;
          const gen =
            res.generate ||
            (res.generates && res.generates[0]) ||
            null;
          const found = res.found_asset_id;
          const storage = gen?.storage_path;
          if (!storage && !found) return null;
          return {
            beat_id: beatId,
            source_path: storage || "",
            asset_id: gen?.asset_id || found || null,
            concept_key: need?.concept_key || res.concept_key || null,
          };
        })
        .filter((x): x is NonNullable<typeof x> => Boolean(x && x.source_path));
      if (!items.length) {
        throw new Error(
          "No hay storage_path de imágenes generadas. Ejecuta Submit (no solo FOUND) o genera variantes.",
        );
      }
      const result = await invokeWritePackageImages(pkgPath, items);
      setMessage(
        `Escrito al package: ${result.image_count} imagen(es). ${result.notes}\n${result.package_path}`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  function updateNeed(index: number, patch: Partial<ManualNeed>) {
    setNeeds((prev) => prev.map((n, i) => (i === index ? { ...n, ...patch } : n)));
  }

  function includedNeeds(): ManualNeed[] {
    return needs.filter((n) => n.included !== false);
  }

  async function onPreview() {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const list = includedNeeds();
      if (list.length === 0) throw new Error("Marca al menos una necesidad (included).");
      const p = await invokePreviewManualBatch(list);
      setPreview(p);
      setResultTab(0);
      setStep("run");
      const foundAsk = p.results.filter(
        (r) => r.decision === "found" || r.decision === "found_enrich",
      ).length;
      const pending = p.pending_review_count ?? 0;
      setMessage(
        `Preview: FOUND ${p.found_count} · GENERATE ${p.generate_count} · SKIPPED ${p.skipped_count}` +
          (pending ? ` · PENDING ${pending}` : "") +
          ` · var ${p.variant_images ?? 0}` +
          (foundAsk ? ` · ${foundAsk} FOUND: enriquecer en pestaña` : ""),
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onSubmit() {
    if (busy) return;
    setBusy(true);
    setError(null);
    const list = includedNeeds();
    const planned = list.reduce((n, x) => n + (Number(x.variant_count) || 3), 0);
    setMessage(
      `Generando… ${list.length} need(s), ~${planned} imagen(es). Con OmniRoute puede tardar 30–120 s; no cierres la app.`,
    );
    try {
      if (list.length === 0) throw new Error("Marca al menos una necesidad (included).");
      const p = await invokeSubmitManualBatch(list);
      setPreview(p);
      setResultTab(0);
      setStep("run");
      const pending = p.pending_review_count ?? 0;
      const gens = p.results.reduce((n, r) => n + (r.generates?.length ?? 0), 0);
      if (gens === 0 && (p.generate_count ?? 0) === 0) {
        setMessage(
          `Submit sin nuevas imágenes. FOUND ${p.found_count}` +
            (pending ? ` · PENDING_REVIEW ${pending} (ya hay cola; ve a Review o cambia need)` : "") +
            ` · SKIPPED ${p.skipped_count}.`,
        );
      } else {
        setMessage(
          `Listo: ${gens || p.variant_images || 0} imagen(es) → ve a Review.` +
            (pending ? ` (${pending} needs ya estaban en cola)` : ""),
        );
      }
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
      setMessage(null);
    } finally {
      setBusy(false);
    }
  }

  function globalNeedIndexForResult(resultIndex: number): number {
    const r = preview?.results[resultIndex];
    if (!r) return -1;
    const included = includedNeeds();
    const need = included[r.index];
    if (!need) return -1;
    return needs.findIndex(
      (n) =>
        n.concept_key === need.concept_key &&
        n.representation_key === need.representation_key &&
        n.script_excerpt === need.script_excerpt,
    );
  }

  function setEnrichForResult(resultIndex: number, value: boolean) {
    const globalIdx = globalNeedIndexForResult(resultIndex);
    if (globalIdx >= 0) {
      updateNeed(globalIdx, { also_generate_if_found: value });
    }
  }

  function enrichCheckedForResult(resultIndex: number): boolean {
    const globalIdx = globalNeedIndexForResult(resultIndex);
    if (globalIdx < 0) return false;
    return needs[globalIdx]?.also_generate_if_found === true;
  }

  function providerOptionLabel(p: ImageProvider): string {
    const runnable =
      p.id === "stub" ||
      p.status === "always" ||
      p.status === "ready" ||
      p.available === true;
    if (runnable && p.can_afford_one !== false) {
      return p.is_free || p.id === "stub" ? `${p.name} · free` : p.name;
    }
    if (p.status === "missing_key") return `${p.name} · sin key`;
    if (p.status === "budget_exhausted") return `${p.name} · sin presupuesto`;
    if (p.status === "disabled") return `${p.name} · off`;
    if (p.status === "not_connected") return `${p.name} · no conectado`;
    return `${p.name} · no listo`;
  }

  const activeNeed = needs[needTab];
  const activeResult = preview?.results[resultTab];

  return (
    <section className="station">
      <header>
        <h2>Manual Factory</h2>
        <p>
          Guion → needs → variantes → Review. También: abrir package FacelessStudio (YouToMagic)
          y escribir imágenes approved a media/images.
        </p>
      </header>

      <ConnectionBanner />

      <div
        style={{
          flexShrink: 0,
          display: "flex",
          flexWrap: "wrap",
          gap: 8,
          alignItems: "center",
          marginBottom: 10,
          padding: "0.5rem 0.65rem",
          borderRadius: 10,
          border: "1px solid var(--border)",
          background: "var(--panel-2, var(--panel))",
        }}
      >
        <label style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>Package</label>
        <select
          value={selectedPackagePath}
          onChange={(e) => setSelectedPackagePath(e.target.value)}
          disabled={busy}
          style={{ ...inputStyle, minWidth: 220, maxWidth: 420, flex: 1 }}
        >
          <option value="">— FacelessStudio packages —</option>
          {packages.map((p) => (
            <option key={p.path} value={p.path}>
              {p.smoke ? "SMOKE · " : ""}
              {p.title || p.package_id} ({p.beats} beats · {p.script_status})
            </option>
          ))}
        </select>
        <button type="button" disabled={busy || !selectedPackagePath} style={btnStyle} onClick={() => void onOpenPackage()}>
          Abrir package
        </button>
        <button
          type="button"
          disabled={busy}
          style={btnStyle}
          onClick={() => {
            void invokeListPackages()
              .then(setPackages)
              .catch(() => setPackages([]));
          }}
        >
          Actualizar lista
        </button>
        {activePackagePath ? (
          <span style={{ fontSize: "0.75rem", color: "var(--accent)" }}>
            Activo: {activePackagePath.split(/[/\\]/).slice(-2).join("/")}
          </span>
        ) : null}
      </div>

      {busy ? (
        <div
          style={{
            flexShrink: 0,
            marginBottom: 8,
            padding: "0.5rem 0.75rem",
            borderRadius: 10,
            border: "1px solid var(--accent)",
            background: "var(--accent-soft)",
            color: "var(--accent)",
            fontWeight: 650,
            fontSize: "0.88rem",
          }}
          role="status"
        >
          Trabajando… espera (OmniRoute puede tardar 1–2 min en cada imagen). No pulses otra vez.
        </div>
      ) : null}

      <div className="tab-strip">
        {(["script", "needs", "run"] as const).map((s) => (
          <button
            key={s}
            type="button"
            className={`tab-btn${step === s ? " active" : ""}`}
            onClick={() => setStep(s)}
            disabled={busy}
          >
            {s === "script" ? "1. Guion" : s === "needs" ? "2. Needs" : "3. Run"}
          </button>
        ))}
      </div>

      {(message || error) && (
        <div style={{ flexShrink: 0, marginBottom: 8 }}>
          {message ? (
            <p className="health msg-ok" style={{ margin: 0, whiteSpace: "pre-wrap" }}>
              {message}
            </p>
          ) : null}
          {error ? (
            <p
              className="health msg-err"
              style={{
                margin: message ? "6px 0 0" : 0,
                whiteSpace: "pre-wrap",
                maxHeight: 120,
                overflowY: "auto",
                wordBreak: "break-word",
              }}
            >
              {error}
            </p>
          ) : null}
        </div>
      )}

      <div className="station-body">
        {step === "script" ? (
          <div className="placeholder-card fill">
            <div className="tab-strip">
              <button
                type="button"
                className={`tab-btn${scriptTab === "guion" ? " active" : ""}`}
                onClick={() => setScriptTab("guion")}
              >
                Guion
              </button>
              <button
                type="button"
                className={`tab-btn${scriptTab === "instrucciones" ? " active" : ""}`}
                onClick={() => setScriptTab("instrucciones")}
              >
                Instrucciones (extra)
              </button>
            </div>
            {scriptTab === "guion" ? (
              <textarea
                value={script}
                onChange={(e) => setScript(e.target.value)}
                style={fillTextarea}
                placeholder="Texto de la lección…"
              />
            ) : (
              <textarea
                value={scriptInstructions}
                onChange={(e) => setScriptInstructions(e.target.value)}
                style={fillTextarea}
                placeholder="Opcional antes de proponer: se envían al chat OmniRoute/Claude. Tras proponer, aquí quedan las script_instructions del modelo."
              />
            )}
            <div style={footerBar}>
              <button type="button" disabled={busy} style={btnStyle} onClick={() => void onPropose()}>
                Proponer needs
              </button>
              {proposeNotes ? (
                <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>
                  {proposeNotes.slice(0, 160)}
                  {proposeNotes.length > 160 ? "…" : ""}
                </span>
              ) : null}
            </div>
          </div>
        ) : null}

        {step === "needs" ? (
          <div className="placeholder-card fill">
            {needs.length === 0 ? (
              <p>Aún no hay needs. Vuelve a Guion → Proponer.</p>
            ) : (
              <>
                <div className="tab-strip">
                  {needs.map((n, i) => (
                    <button
                      key={i}
                      type="button"
                      className={`tab-btn${needTab === i ? " active" : ""}`}
                      onClick={() => setNeedTab(i)}
                    >
                      #{i + 1}
                      {n.included === false ? " ·off" : ""}
                    </button>
                  ))}
                </div>
                {activeNeed ? (
                  <div style={needForm}>
                    <label style={rowLabel}>
                      <input
                        type="checkbox"
                        checked={activeNeed.included !== false}
                        onChange={(e) => updateNeed(needTab, { included: e.target.checked })}
                      />
                      Incluir · {activeNeed.concept_key} / {activeNeed.representation_key}
                    </label>
                    <div style={twoCol}>
                      <div>
                        <label style={labelStyle}>concept_key</label>
                        <input
                          value={activeNeed.concept_key}
                          onChange={(e) => updateNeed(needTab, { concept_key: e.target.value })}
                          style={inputStyle}
                        />
                        <label style={labelStyle}>concept_name</label>
                        <input
                          value={activeNeed.concept_name ?? ""}
                          onChange={(e) => updateNeed(needTab, { concept_name: e.target.value })}
                          style={inputStyle}
                        />
                        <div style={{ display: "flex", gap: 12, marginTop: 8, flexWrap: "wrap" }}>
                          <label style={{ fontSize: "0.82rem" }}>
                            Variantes{" "}
                            <select
                              value={activeNeed.variant_count ?? 3}
                              onChange={(e) =>
                                updateNeed(needTab, {
                                  variant_count: Number(e.target.value) as 1 | 2 | 3,
                                })
                              }
                              style={{ ...inputStyle, width: "auto", display: "inline-block" }}
                            >
                              <option value={1}>1</option>
                              <option value={2}>2</option>
                              <option value={3}>3</option>
                            </select>
                          </label>
                          <label style={{ fontSize: "0.82rem" }}>
                            Provider{" "}
                            <select
                              value={activeNeed.provider ?? "stub"}
                              onChange={(e) => updateNeed(needTab, { provider: e.target.value })}
                              style={{ ...inputStyle, width: "auto", display: "inline-block" }}
                            >
                              {(providers.length
                                ? providers
                                : [
                                    {
                                      id: "stub",
                                      name: "Stub",
                                      status: "always",
                                      available: true,
                                      cost_score: 0,
                                      quality_score: 20,
                                      availability_score: 100,
                                      kind: "local_stub",
                                    } as ImageProvider,
                                  ]
                              ).map((p) => (
                                <option key={p.id} value={p.id}>
                                  {providerOptionLabel(p)}
                                </option>
                              ))}
                            </select>
                          </label>
                        </div>
                      </div>
                      <div style={{ display: "flex", flexDirection: "column", minHeight: 0, flex: 1 }}>
                        <label style={labelStyle}>Instrucciones IA del tramo</label>
                        <textarea
                          value={activeNeed.ai_instructions ?? ""}
                          onChange={(e) =>
                            updateNeed(needTab, { ai_instructions: e.target.value })
                          }
                          style={halfTextarea}
                        />
                        <label style={labelStyle}>Prompt base</label>
                        <textarea
                          value={activeNeed.prompt ?? ""}
                          onChange={(e) => updateNeed(needTab, { prompt: e.target.value })}
                          style={halfTextarea}
                        />
                      </div>
                    </div>
                  </div>
                ) : null}
                <div style={footerBar}>
                  <button
                    type="button"
                    disabled={busy}
                    style={btnStyle}
                    onClick={() => void onPreview()}
                  >
                    Preview
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    style={btnStyle}
                    onClick={() => void onSubmit()}
                  >
                    {busy ? "Generando…" : "Submit → Review"}
                  </button>
                  <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>
                    {includedNeeds().length}/{needs.length} incluidas
                    {includedNeeds().length > 0
                      ? ` · ~${includedNeeds().reduce((n, x) => n + (Number(x.variant_count) || 3), 0)} imgs`
                      : ""}
                  </span>
                </div>
              </>
            )}
          </div>
        ) : null}

        {step === "run" ? (
          <div className="placeholder-card fill">
            {!preview ? (
              <p>Ejecuta Preview o Submit desde Needs.</p>
            ) : (
              <>
                <div className="tab-strip">
                  {preview.results.map((r, ri) => (
                    <button
                      key={ri}
                      type="button"
                      className={`tab-btn${resultTab === ri ? " active" : ""}`}
                      onClick={() => setResultTab(ri)}
                    >
                      #{ri + 1} {r.decision}
                    </button>
                  ))}
                </div>
                {activeResult ? (
                  <div style={resultBody}>
                    <p style={{ margin: 0, fontSize: "0.95rem", color: "var(--text)" }}>
                      <strong>{activeResult.decision.toUpperCase()}</strong> ·{" "}
                      {activeResult.concept_key}/{activeResult.representation_key}
                      {activeResult.variants_planned
                        ? ` · plan ${activeResult.variants_planned}`
                        : ""}
                      {activeResult.selected_provider
                        ? ` · ${activeResult.selected_provider}`
                        : ""}
                    </p>
                    <p style={{ margin: "0.5rem 0 0", color: "var(--text-muted)", fontSize: "0.85rem" }}>
                      {activeResult.message}
                    </p>
                    {activeResult.generates && activeResult.generates.length > 0 ? (
                      <p style={{ margin: "0.5rem 0 0", fontSize: "0.85rem" }}>
                        Generadas: {activeResult.generates.length}
                        {activeResult.matiz_labels?.length
                          ? ` (${activeResult.matiz_labels.join(", ")})`
                          : ""}
                      </p>
                    ) : null}
                    {activeResult.decision === "found" ||
                    activeResult.decision === "found_enrich" ? (
                      <label style={rowLabel}>
                        <input
                          type="checkbox"
                          checked={enrichCheckedForResult(resultTab)}
                          onChange={(e) => setEnrichForResult(resultTab, e.target.checked)}
                        />
                        También generar variantes (enriquecer Library)
                      </label>
                    ) : null}
                  </div>
                ) : null}
                <div style={footerBar}>
                  <button type="button" style={btnStyle} onClick={() => setStep("needs")}>
                    Needs
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    style={btnStyle}
                    onClick={() => void onPreview()}
                  >
                    Re-preview
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    style={btnStyle}
                    onClick={() => void onSubmit()}
                  >
                    Submit
                  </button>
                  <button
                    type="button"
                    disabled={busy || !(activePackagePath || selectedPackagePath)}
                    style={{ ...btnStyle, borderColor: "var(--accent)" }}
                    onClick={() => void onWriteToPackage()}
                    title="Copia storage_path de resultados a package media/images/{beat_id}"
                  >
                    Escribir al package
                  </button>
                </div>
              </>
            )}
          </div>
        ) : null}

      </div>
    </section>
  );
}

export function FactoryPage() {
  return (
    <div className="station" style={{ gap: 0 }}>
      <div className="tab-strip" style={{ marginBottom: 8 }}>
        <NavLink
          to="/factory/manual"
          className={({ isActive }) => `tab-btn${isActive ? " active" : ""}`}
          style={{ textDecoration: "none" }}
        >
          Manual
        </NavLink>
        <NavLink
          to="/factory/automatic"
          className={({ isActive }) => `tab-btn${isActive ? " active" : ""}`}
          style={{ textDecoration: "none" }}
        >
          Automatic
        </NavLink>
      </div>
      <div className="station-body">
        <Routes>
          <Route index element={<Navigate to="manual" replace />} />
          <Route path="manual" element={<ManualFactory />} />
          <Route path="automatic" element={<AutomaticFactory />} />
        </Routes>
      </div>
    </div>
  );
}

const fillTextarea: CSSProperties = {
  flex: "1 1 auto",
  minHeight: 0,
  width: "100%",
  fontFamily: "ui-monospace, Consolas, monospace",
  fontSize: "0.82rem",
  padding: "0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  resize: "none",
};

const halfTextarea: CSSProperties = {
  ...fillTextarea,
  flex: "1 1 0",
  minHeight: 48,
};

const needForm: CSSProperties = {
  flex: "1 1 auto",
  minHeight: 0,
  display: "flex",
  flexDirection: "column",
  overflow: "hidden",
  gap: 6,
};

const twoCol: CSSProperties = {
  flex: 1,
  minHeight: 0,
  display: "grid",
  gridTemplateColumns: "minmax(180px, 1fr) minmax(200px, 1.4fr)",
  gap: 12,
  overflow: "hidden",
};

const resultBody: CSSProperties = {
  flex: 1,
  minHeight: 0,
  overflow: "hidden",
};

const footerBar: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: 8,
  alignItems: "center",
  flexShrink: 0,
  marginTop: 8,
};

const inputStyle: CSSProperties = {
  width: "100%",
  display: "block",
  padding: "0.35rem 0.5rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  boxSizing: "border-box",
};

const labelStyle: CSSProperties = {
  display: "block",
  marginTop: 6,
  marginBottom: 3,
  fontSize: "0.78rem",
  color: "var(--text-muted)",
};

const rowLabel: CSSProperties = {
  display: "flex",
  gap: 8,
  alignItems: "center",
  fontSize: "0.88rem",
  flexShrink: 0,
  marginTop: 8,
};

const btnStyle: CSSProperties = {
  padding: "0.4rem 0.8rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer",
};
