import { type CSSProperties, useCallback, useEffect, useState } from "react";
import {
  invokeListPlans,
  invokeRunAutomaticPlan,
  type PlanDto,
  type AutomaticRunResult,
} from "../../shared/ipc/client";
import { ConnectionBanner } from "./ConnectionBanner";

export function AutomaticFactory() {
  const [plans, setPlans] = useState<PlanDto[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [result, setResult] = useState<AutomaticRunResult | null>(null);
  const [resultTab, setResultTab] = useState(0);
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
      setResultTab(0);
      const pending = r.batch.pending_review_count ?? 0;
      setMessage(
        `FOUND ${r.batch.found_count} · GENERATE ${r.batch.generate_count}` +
          (pending ? ` · PENDING ${pending}` : "") +
          `. Revisa Review.`,
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

  const active = result?.batch.results[resultTab];

  return (
    <section className="station">
      <header>
        <h2>Automatic Factory</h2>
        <p>Solo planes approved. Plans decide qué; aquí se ejecuta. Sin scroll.</p>
      </header>

      <ConnectionBanner />

      <div className="station-body">
        <div className="placeholder-card fill">
          {plans.length === 0 ? (
            <p>
              Empty: no hay planes approved. Crea y aprueba en <strong>Plans</strong>.
            </p>
          ) : (
            <>
              <div className="tab-strip" style={{ marginBottom: 8 }}>
                {plans.map((p) => (
                  <button
                    key={p.id}
                    type="button"
                    className={`tab-btn${selectedId === p.id ? " active" : ""}`}
                    onClick={() => setSelectedId(p.id)}
                  >
                    {p.name}
                  </button>
                ))}
              </div>
              <div style={footerBar}>
                <button
                  type="button"
                  disabled={busy || !selectedId}
                  style={btnStyle}
                  onClick={() => void onRun()}
                >
                  Run Automatic
                </button>
              </div>
            </>
          )}

          {result && result.batch.results.length > 0 ? (
            <>
              <div className="tab-strip" style={{ marginTop: 12 }}>
                {result.batch.results.map((r, i) => (
                  <button
                    key={i}
                    type="button"
                    className={`tab-btn${resultTab === i ? " active" : ""}`}
                    onClick={() => setResultTab(i)}
                  >
                    #{i + 1} {r.decision}
                  </button>
                ))}
              </div>
              {active ? (
                <p style={{ margin: "0.5rem 0 0", fontSize: "0.88rem", color: "var(--text-muted)" }}>
                  <strong style={{ color: "var(--text)" }}>{active.decision}</strong>{" "}
                  {active.concept_key}/{active.representation_key} — {active.message}
                </p>
              ) : null}
            </>
          ) : null}

          {message ? <p className="health msg-ok">{message}</p> : null}
          {error ? <p className="health msg-err">{error}</p> : null}
        </div>
      </div>
    </section>
  );
}

const footerBar: CSSProperties = {
  display: "flex",
  gap: 8,
  flexShrink: 0,
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
