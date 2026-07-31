import { type CSSProperties, FormEvent, useCallback, useEffect, useState } from "react";
import {
  invokeAddPlanItem,
  invokeApprovePlan,
  invokeCreatePlan,
  invokeGetPlan,
  invokeListPlans,
  type PlanDto,
  type PlanWithItemsDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable";

export function PlansPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [plans, setPlans] = useState<PlanDto[]>([]);
  const [selected, setSelected] = useState<PlanWithItemsDto | null>(null);
  const [name, setName] = useState("Coverage growth");
  const [conceptKey, setConceptKey] = useState("river");
  const [repKey, setRepKey] = useState("wide");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const list = await invokeListPlans();
      setPlans(list);
      setLoad("ready");
      if (selected) {
        const detail = await invokeGetPlan(selected.plan.id);
        setSelected(detail);
      }
    } catch {
      setLoad("unavailable");
    }
  }, [selected]);

  useEffect(() => {
    void (async () => {
      try {
        const list = await invokeListPlans();
        setPlans(list);
        setLoad("ready");
      } catch {
        setLoad("unavailable");
      }
    })();
  }, []);

  async function selectPlan(id: string) {
    setError(null);
    try {
      const detail = await invokeGetPlan(id);
      setSelected(detail);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    }
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      const p = await invokeCreatePlan(name.trim());
      setMessage(`Plan draft creado: ${p.name}`);
      await refresh();
      await selectPlan(p.id);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onAddItem(e: FormEvent) {
    e.preventDefault();
    if (!selected || busy) return;
    setBusy(true);
    setError(null);
    try {
      await invokeAddPlanItem({
        planId: selected.plan.id,
        conceptKey: conceptKey.trim(),
        representationKey: repKey.trim(),
        orientation: "any",
        style: "any",
      });
      setMessage("Item añadido (solo draft).");
      await selectPlan(selected.plan.id);
      const list = await invokeListPlans();
      setPlans(list);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onApprove() {
    if (!selected || busy) return;
    setBusy(true);
    setError(null);
    try {
      const p = await invokeApprovePlan(selected.plan.id);
      setMessage(
        `Plan approved — habilita Automatic Factory (no genera). Status: ${p.status}`,
      );
      await selectPlan(p.id);
      const list = await invokeListPlans();
      setPlans(list);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="station">
      <header>
        <h2>Plans</h2>
        <p>
          Decide <strong>qué</strong> generar. Approve no llama providers. Automatic Factory ejecuta
          planes approved.
        </p>
      </header>

      {load === "unavailable" ? (
        <div className="placeholder-card">
          <p>IPC no disponible. Usa `pnpm dev`.</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "minmax(0, 32%) minmax(0, 68%)",
            gap: "1rem",
          }}
        >
          <div className="placeholder-card">
            <h3>Planes ({plans.length})</h3>
            {plans.length === 0 ? (
              <p>Empty: crea un plan draft.</p>
            ) : (
              <ul style={{ listStyle: "none", padding: 0 }}>
                {plans.map((p) => (
                  <li key={p.id} style={{ marginBottom: 6 }}>
                    <button
                      type="button"
                      onClick={() => void selectPlan(p.id)}
                      style={{
                        width: "100%",
                        textAlign: "left",
                        padding: "0.5rem",
                        borderRadius: 8,
                        border:
                          selected?.plan.id === p.id
                            ? "2px solid var(--accent)"
                            : "1px solid var(--border)",
                        background:
                          selected?.plan.id === p.id
                            ? "var(--accent-soft)"
                            : "transparent",
                        color: "var(--text)",
                        cursor: "pointer",
                      }}
                    >
                      {p.name}{" "}
                      <span style={{ color: "var(--text-muted)" }}>({p.status})</span>
                    </button>
                  </li>
                ))}
              </ul>
            )}

            <form onSubmit={onCreate} style={{ marginTop: 12 }}>
              <label htmlFor="plan_name">Nuevo plan</label>
              <input
                id="plan_name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                style={inputStyle}
                required
              />
              <button type="submit" disabled={busy} style={btnStyle}>
                Create draft
              </button>
            </form>
          </div>

          <div className="placeholder-card">
            {!selected ? (
              <p>Selecciona un plan.</p>
            ) : (
              <>
                <h3>
                  {selected.plan.name} — <code>{selected.plan.status}</code>
                </h3>
                <p style={{ color: "var(--text-muted)" }}>
                  Items: {selected.items.length}. Approve no genera assets.
                </p>
                <ul>
                  {selected.items.map((i) => (
                    <li key={i.id}>
                      {i.concept_key}/{i.representation_key} — {i.status}
                    </li>
                  ))}
                </ul>

                {selected.plan.status === "draft" ? (
                  <>
                    <form onSubmit={onAddItem} style={{ marginTop: 12 }}>
                      <label>concept_key</label>
                      <input
                        value={conceptKey}
                        onChange={(e) => setConceptKey(e.target.value)}
                        style={inputStyle}
                        required
                      />
                      <label style={{ display: "block", marginTop: 8 }}>
                        representation_key
                      </label>
                      <input
                        value={repKey}
                        onChange={(e) => setRepKey(e.target.value)}
                        style={inputStyle}
                        required
                      />
                      <button type="submit" disabled={busy} style={btnStyle}>
                        Add item
                      </button>
                    </form>
                    <button
                      type="button"
                      disabled={busy || selected.items.length === 0}
                      style={{ ...btnStyle, marginTop: 12 }}
                      onClick={() => void onApprove()}
                    >
                      Approve plan
                    </button>
                  </>
                ) : (
                  <p>
                    Plan approved. Ejecuta en{" "}
                    <strong>Factory → Automatic</strong>.
                  </p>
                )}
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
        </div>
      ) : null}
    </section>
  );
}

const inputStyle: CSSProperties = {
  width: "100%",
  maxWidth: "20rem",
  display: "block",
  padding: "0.45rem 0.6rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  marginBottom: 8,
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
