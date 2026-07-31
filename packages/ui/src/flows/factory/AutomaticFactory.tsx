import { type CSSProperties, useCallback, useEffect, useState } from "react";
import {
  invokeListPlans,
  invokeRunAutomaticPlan,
  type PlanDto,
  type AutomaticRunResult,
} from "../../shared/ipc/client";

export function AutomaticFactory() {
  const [plans, setPlans] = useState<PlanDto[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [result, setResult] = useState<AutomaticRunResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [unavailable, setUnavailable] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const list = await invokeListPlans();
      const approved = list.filter((p) => p.status === "approved");
      setPlans(approved);
      setSelectedId((prev) =>
        prev && approved.some((p) => p.id === prev) ? prev : approved[0]?.id ?? "",
      );
      setUnavailable(false);
    } catch {
      setUnavailable(true);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function onRun() {
    if (!selectedId || busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    setResult(null);
    try {
      const r = await invokeRunAutomaticPlan(selectedId);
      setResult(r);
      setMessage(
        `Automatic run: FOUND ${r.batch.found_count} · GENERATE ${r.batch.generate_count}. Revisa Waiting Review.`,
      );
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  if (unavailable) {
    return (
      <section className="station">
        <header>
          <h2>Automatic Factory</h2>
          <p>IPC no disponible. Usa `pnpm dev`.</p>
        </header>
      </section>
    );
  }

  return (
    <section className="station">
      <header>
        <h2>Automatic Factory</h2>
        <p>
          Solo planes <strong>approved</strong>. No genera al azar. Plans decide qué; aquí se
          ejecuta.
        </p>
      </header>

      <div className="placeholder-card">
        {plans.length === 0 ? (
          <p>
            Empty: no hay planes approved. Crea items y aprueba en <strong>Plans</strong>.
          </p>
        ) : (
          <>
            <label htmlFor="plan_sel">Plan approved</label>
            <select
              id="plan_sel"
              value={selectedId}
              onChange={(e) => setSelectedId(e.target.value)}
              style={selectStyle}
            >
              {plans.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
            <div style={{ marginTop: 12 }}>
              <button type="button" disabled={busy || !selectedId} style={btnStyle} onClick={() => void onRun()}>
                Run Automatic Factory
              </button>
            </div>
          </>
        )}
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

      {result ? (
        <div className="placeholder-card" style={{ marginTop: "1rem" }}>
          <h3>Resultado</h3>
          <ul>
            {result.batch.results.map((r) => (
              <li key={r.index}>
                <strong>{r.decision}</strong> {r.concept_key}/{r.representation_key} — {r.message}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}

const selectStyle: CSSProperties = {
  display: "block",
  marginTop: 6,
  padding: "0.45rem 0.6rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  minWidth: "16rem",
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
