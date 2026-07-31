import { type CSSProperties, useState } from "react";
import { NavLink, Navigate, Route, Routes } from "react-router-dom";
import {
  invokePreviewManualBatch,
  invokeSubmitManualBatch,
  type ManualBatchPreview,
  type ManualNeed,
} from "../../shared/ipc/client";
import { AutomaticFactory } from "./AutomaticFactory";

const DEFAULT_JSON = `[
  {
    "concept_key": "oak-tree",
    "concept_name": "Oak Tree",
    "representation_key": "hero",
    "representation_name": "Hero",
    "prompt": "majestic oak tree, landscape",
    "orientation": "landscape",
    "style": "any",
    "provider": "stub"
  }
]`;

function ManualFactory() {
  const [json, setJson] = useState(DEFAULT_JSON);
  const [preview, setPreview] = useState<ManualBatchPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  function parseNeeds(): ManualNeed[] {
    const data = JSON.parse(json) as ManualNeed[];
    if (!Array.isArray(data) || data.length === 0) {
      throw new Error("Se espera un array JSON de necesidades no vacío");
    }
    return data.map((n) => ({
      concept_key: n.concept_key,
      concept_name: n.concept_name ?? null,
      representation_key: n.representation_key,
      representation_name: n.representation_name ?? null,
      prompt: n.prompt ?? null,
      orientation: n.orientation ?? "any",
      style: n.style ?? "any",
      provider: n.provider ?? "stub",
    }));
  }

  async function onPreview() {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const needs = parseNeeds();
      const p = await invokePreviewManualBatch(needs);
      setPreview(p);
      setMessage(
        `Preview: FOUND ${p.found_count} · GENERATE ${p.generate_count} · SKIPPED ${p.skipped_count}`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
      setPreview(null);
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
      const needs = parseNeeds();
      const p = await invokeSubmitManualBatch(needs);
      setPreview(p);
      setMessage(
        `Submit: FOUND ${p.found_count} · GENERATE ${p.generate_count} (solo faltantes → Waiting Review). Ve a Review.`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="station">
      <header>
        <h2>Manual Factory</h2>
        <p>
          Lista estructurada de necesidades. Preview FOUND/GENERATE; Submit solo genera
          faltantes → Waiting Review.
        </p>
      </header>

      <div className="placeholder-card">
        <h3>Needs (JSON)</h3>
        <textarea
          value={json}
          onChange={(e) => setJson(e.target.value)}
          rows={14}
          style={textareaStyle}
          spellCheck={false}
        />
        <div style={{ display: "flex", gap: 8, marginTop: 12, flexWrap: "wrap" }}>
          <button type="button" disabled={busy} style={btnStyle} onClick={() => void onPreview()}>
            Preview FOUND/GENERATE
          </button>
          <button type="button" disabled={busy} style={btnStyle} onClick={() => void onSubmit()}>
            Submit generate faltantes
          </button>
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
      </div>

      {preview ? (
        <div className="placeholder-card" style={{ marginTop: "1rem" }}>
          <h3>Resultados</h3>
          <ul>
            {preview.results.map((r) => (
              <li key={r.index}>
                <strong>{r.decision.toUpperCase()}</strong> · {r.concept_key}/{r.representation_key}
                {r.found_asset_id ? (
                  <>
                    {" "}
                    · found <code>{r.found_asset_id.slice(0, 12)}…</code>
                  </>
                ) : null}
                {r.generate ? (
                  <>
                    {" "}
                    · asset <code>{r.generate.asset_status}</code>
                  </>
                ) : null}
                <div style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>{r.message}</div>
              </li>
            ))}
          </ul>
        </div>
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

const btnStyle: CSSProperties = {
  padding: "0.45rem 0.9rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer",
};

const tabStyle: CSSProperties = {
  padding: "0.35rem 0.75rem",
  borderRadius: 8,
  color: "var(--text-muted)",
  border: "1px solid var(--border)",
};
