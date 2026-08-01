import { type CSSProperties, useEffect, useState } from "react";
import { NavLink, Navigate, Route, Routes } from "react-router-dom";
import {
  invokeListImageProviders,
  invokePreviewManualBatch,
  invokeProposeNeedsFromScript,
  invokeSubmitManualBatch,
  type ImageProvider,
  type ManualBatchPreview,
  type ManualNeed,
} from "../../shared/ipc/client";
import { AutomaticFactory } from "./AutomaticFactory";

const SAMPLE_SCRIPT = `Hoy hablamos de fotosíntesis en las plantas.

Las hojas capturan la luz solar gracias a la clorofila.

El dióxido de carbono entra por los estomas y se transforma en glucosa.

Como resultado se libera oxígeno, esencial para la vida en el planeta.

En la práctica de laboratorio observamos el color verde como señal de clorofila activa.`;

type Step = "script" | "needs" | "run";

function ManualFactory() {
  const [step, setStep] = useState<Step>("script");
  const [script, setScript] = useState(SAMPLE_SCRIPT);
  const [scriptInstructions, setScriptInstructions] = useState("");
  const [needs, setNeeds] = useState<ManualNeed[]>([]);
  const [providers, setProviders] = useState<ImageProvider[]>([]);
  const [proposeNotes, setProposeNotes] = useState<string | null>(null);
  const [preview, setPreview] = useState<ManualBatchPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void invokeListImageProviders()
      .then(setProviders)
      .catch(() => setProviders([]));
  }, []);

  async function onPropose() {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const r = await invokeProposeNeedsFromScript(script);
      setNeeds(r.needs);
      setScriptInstructions(r.script_instructions);
      setProposeNotes(`${r.method}: ${r.notes}`);
      setPreview(null);
      setStep("needs");
      setMessage(
        `Propuesta: ${r.needs.length} needs (requerimientos BD). Default 3 variantes. Edita e incluye.`,
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
      setStep("run");
      const foundAsk = p.results.filter((r) => r.decision === "found").length;
      setMessage(
        `Preview: FOUND ${p.found_count} · GENERATE ${p.generate_count} · SKIPPED ${p.skipped_count} · variantes planificadas ${p.variant_images ?? 0}` +
          (foundAsk
            ? ` · ${foundAsk} FOUND: decide abajo si enriquecer con variantes.`
            : ""),
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
    setMessage(null);
    try {
      const list = includedNeeds();
      if (list.length === 0) throw new Error("Marca al menos una necesidad (included).");
      const p = await invokeSubmitManualBatch(list);
      setPreview(p);
      setStep("run");
      setMessage(
        `Submit: ${p.variant_images ?? 0} imagen(es) → Waiting Review. FOUND ${p.found_count}. Ve a Review (puedes aprobar 1–3 por need).`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  /** When preview shows FOUND, toggle enrich on the matching need by concept/rep index. */
  function setEnrichForResult(resultIndex: number, value: boolean) {
    const r = preview?.results[resultIndex];
    if (!r) return;
    // Map back via result.index into full needs list used at preview time (included only).
    const included = includedNeeds();
    const need = included[r.index];
    if (!need) return;
    const globalIdx = needs.findIndex(
      (n) =>
        n.concept_key === need.concept_key &&
        n.representation_key === need.representation_key &&
        n.script_excerpt === need.script_excerpt,
    );
    if (globalIdx >= 0) {
      updateNeed(globalIdx, { also_generate_if_found: value });
    }
  }

  return (
    <section className="station">
      <header>
        <h2>Manual Factory</h2>
        <p>
          Guion + instrucciones IA → needs (datos BD) → 1–3 variantes (literal/metáfora + estilo)
          → FOUND Library o GENERATE → Review.
        </p>
      </header>

      <div style={{ display: "flex", gap: 8, marginBottom: "1rem", flexWrap: "wrap" }}>
        {(["script", "needs", "run"] as const).map((s) => (
          <button
            key={s}
            type="button"
            style={step === s ? stepActive : stepIdle}
            onClick={() => setStep(s)}
          >
            {s === "script" ? "1. Guion" : s === "needs" ? "2. Needs (BD)" : "3. Preview / Submit"}
          </button>
        ))}
      </div>

      {step === "script" ? (
        <div className="placeholder-card">
          <h3>Guion (texto de la lección)</h3>
          <textarea
            value={script}
            onChange={(e) => setScript(e.target.value)}
            rows={10}
            style={textareaStyle}
          />
          <h3 style={{ marginTop: "1rem" }}>Instrucciones de la IA sobre el guion</h3>
          <p style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>
            Se rellenan al proponer needs. Puedes editarlas: orientan qué reconocer, no son la need.
          </p>
          <textarea
            value={scriptInstructions}
            onChange={(e) => setScriptInstructions(e.target.value)}
            rows={6}
            placeholder="Pulsa «Proponer needs» para generar instrucciones…"
            style={textareaStyle}
          />
          <div style={{ marginTop: 12 }}>
            <button type="button" disabled={busy} style={btnStyle} onClick={() => void onPropose()}>
              Proponer needs desde guion
            </button>
          </div>
          {proposeNotes ? (
            <p className="health" style={{ color: "var(--text-muted)" }}>
              {proposeNotes}
            </p>
          ) : null}
        </div>
      ) : null}

      {step === "needs" ? (
        <div className="placeholder-card">
          <h3>Needs = requerimientos de BD ({needs.length})</h3>
          <p style={{ color: "var(--text-muted)", fontSize: "0.9rem" }}>
            Cada need es un requerimiento (concepto/representación/metadata). Del mismo prompt base
            salen <strong>1–3 variantes</strong> (matices literal/metafórico + estilo). Default: 3.
          </p>
          {scriptInstructions ? (
            <details style={{ marginBottom: 12 }}>
              <summary style={{ cursor: "pointer", color: "var(--accent)" }}>
                Instrucciones del guion (referencia)
              </summary>
              <pre style={preStyle}>{scriptInstructions}</pre>
            </details>
          ) : null}
          {needs.length === 0 ? (
            <p>Aún no hay needs. Vuelve al paso 1.</p>
          ) : (
            <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
              {needs.map((n, i) => (
                <li key={i} style={needCardStyle}>
                  <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <input
                      type="checkbox"
                      checked={n.included !== false}
                      onChange={(e) => updateNeed(i, { included: e.target.checked })}
                    />
                    <strong>
                      #{i + 1} {n.concept_key} / {n.representation_key}
                    </strong>
                  </label>
                  <label style={labelStyle}>concept_key (BD)</label>
                  <input
                    value={n.concept_key}
                    onChange={(e) => updateNeed(i, { concept_key: e.target.value })}
                    style={inputStyle}
                  />
                  <label style={labelStyle}>concept_name</label>
                  <input
                    value={n.concept_name ?? ""}
                    onChange={(e) => updateNeed(i, { concept_name: e.target.value })}
                    style={inputStyle}
                  />
                  <label style={labelStyle}>Instrucciones IA del tramo</label>
                  <textarea
                    value={n.ai_instructions ?? ""}
                    onChange={(e) => updateNeed(i, { ai_instructions: e.target.value })}
                    rows={3}
                    style={textareaStyle}
                  />
                  <label style={labelStyle}>Prompt base (editable; las variantes añaden matiz)</label>
                  <textarea
                    value={n.prompt ?? ""}
                    onChange={(e) => updateNeed(i, { prompt: e.target.value })}
                    rows={4}
                    style={textareaStyle}
                  />
                  <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginTop: 8 }}>
                    <label style={{ fontSize: "0.85rem" }}>
                      Variantes{" "}
                      <select
                        value={n.variant_count ?? 3}
                        onChange={(e) =>
                          updateNeed(i, { variant_count: Number(e.target.value) as 1 | 2 | 3 })
                        }
                        style={{ ...inputStyle, width: "auto", display: "inline-block" }}
                      >
                        <option value={1}>1</option>
                        <option value={2}>2</option>
                        <option value={3}>3</option>
                      </select>
                    </label>
                    <label style={{ fontSize: "0.85rem" }}>
                      Provider{" "}
                      <select
                        value={n.provider ?? "stub"}
                        onChange={(e) => updateNeed(i, { provider: e.target.value })}
                        style={{ ...inputStyle, width: "auto", display: "inline-block" }}
                      >
                        {(providers.length
                          ? providers
                          : [{ id: "stub", name: "stub", available: true } as ImageProvider]
                        ).map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name}
                            {p.available ? "" : " (fallback)"}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                  {n.script_excerpt ? (
                    <p style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginTop: 8 }}>
                      Excerpt: {n.script_excerpt.slice(0, 140)}
                      {n.script_excerpt.length > 140 ? "…" : ""}
                    </p>
                  ) : null}
                </li>
              ))}
            </ul>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 12, flexWrap: "wrap" }}>
            <button type="button" disabled={busy} style={btnStyle} onClick={() => void onPreview()}>
              Preview FOUND / GENERATE
            </button>
            <button type="button" disabled={busy} style={btnStyle} onClick={() => void onSubmit()}>
              Submit (generar variantes)
            </button>
          </div>
        </div>
      ) : null}

      {step === "run" ? (
        <div className="placeholder-card">
          <h3>Resultados</h3>
          {!preview ? (
            <p>Ejecuta Preview o Submit.</p>
          ) : (
            <ul style={{ listStyle: "none", padding: 0 }}>
              {preview.results.map((r, ri) => (
                <li key={ri} style={needCardStyle}>
                  <strong>{r.decision.toUpperCase()}</strong> · {r.concept_key}/
                  {r.representation_key}
                  {r.variants_planned ? (
                    <>
                      {" "}
                      · plan {r.variants_planned} var.
                    </>
                  ) : null}
                  {r.selected_provider ? (
                    <>
                      {" "}
                      · <code>{r.selected_provider}</code>
                    </>
                  ) : null}
                  {r.found_asset_id ? (
                    <>
                      {" "}
                      · found <code>{r.found_asset_id.slice(0, 14)}…</code>
                    </>
                  ) : null}
                  {r.generates && r.generates.length > 0 ? (
                    <div style={{ fontSize: "0.85rem", marginTop: 6 }}>
                      Generadas: {r.generates.length}
                      {r.matiz_labels?.length
                        ? ` (${r.matiz_labels.join(", ")})`
                        : ""}
                    </div>
                  ) : null}
                  <div style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>{r.message}</div>
                  {r.decision === "found" ? (
                    <label
                      style={{
                        display: "flex",
                        gap: 8,
                        alignItems: "center",
                        marginTop: 8,
                        fontSize: "0.9rem",
                      }}
                    >
                      <input
                        type="checkbox"
                        onChange={(e) => {
                          setEnrichForResult(ri, e.target.checked);
                          // refresh decision path: user must Preview again after toggle
                        }}
                      />
                      Ya hay uno en Library. ¿También generar variantes para enriquecer el canal?
                      <span style={{ color: "var(--text-muted)" }}>(luego pulsa Preview de nuevo)</span>
                    </label>
                  ) : null}
                </li>
              ))}
            </ul>
          )}
          <div style={{ display: "flex", gap: 8, marginTop: 12, flexWrap: "wrap" }}>
            <button type="button" style={btnStyle} onClick={() => setStep("needs")}>
              Volver a needs
            </button>
            <button type="button" disabled={busy} style={btnStyle} onClick={() => void onPreview()}>
              Re-preview
            </button>
            <button type="button" disabled={busy} style={btnStyle} onClick={() => void onSubmit()}>
              Submit
            </button>
          </div>
        </div>
      ) : null}

      {message ? (
        <p className="health" style={{ color: "var(--accent)", marginTop: 12 }}>
          {message}
        </p>
      ) : null}
      {error ? (
        <p className="health" style={{ color: "#f87171", marginTop: 12 }}>
          {error}
        </p>
      ) : null}
    </section>
  );
}

export function FactoryPage() {
  return (
    <div>
      <div style={{ display: "flex", gap: "0.75rem", marginBottom: "1rem" }}>
        <NavLink to="/factory/manual" style={tabStyle}>
          Manual
        </NavLink>
        <NavLink to="/factory/automatic" style={tabStyle}>
          Automatic
        </NavLink>
      </div>
      <Routes>
        <Route index element={<Navigate to="manual" replace />} />
        <Route path="manual" element={<ManualFactory />} />
        <Route path="automatic" element={<AutomaticFactory />} />
      </Routes>
    </div>
  );
}

const textareaStyle: CSSProperties = {
  width: "100%",
  fontFamily: "ui-monospace, Consolas, monospace",
  fontSize: "0.85rem",
  padding: "0.75rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  boxSizing: "border-box",
};

const preStyle: CSSProperties = {
  ...textareaStyle,
  whiteSpace: "pre-wrap",
  marginTop: 8,
};

const inputStyle: CSSProperties = {
  width: "100%",
  maxWidth: "36rem",
  display: "block",
  padding: "0.4rem 0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  boxSizing: "border-box",
};

const labelStyle: CSSProperties = {
  display: "block",
  marginTop: 8,
  marginBottom: 4,
  fontSize: "0.8rem",
  color: "var(--text-muted)",
};

const needCardStyle: CSSProperties = {
  border: "1px solid var(--border)",
  borderRadius: 10,
  padding: "0.85rem",
  marginBottom: 12,
  background: "rgba(255,255,255,0.02)",
};

const btnStyle: CSSProperties = {
  padding: "0.45rem 0.9rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer",
};

const stepActive: CSSProperties = {
  ...btnStyle,
  borderColor: "var(--accent)",
};

const stepIdle: CSSProperties = {
  ...btnStyle,
  background: "transparent",
  color: "var(--text-muted)",
};

const tabStyle: CSSProperties = {
  padding: "0.35rem 0.75rem",
  borderRadius: 8,
  color: "var(--text-muted)",
  border: "1px solid var(--border)",
};
