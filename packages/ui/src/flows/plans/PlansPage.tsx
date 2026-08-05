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
type DetailTab = "items" | "edit";

export function PlansPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [plans, setPlans] = useState<PlanDto[]>([]);
  const [selected, setSelected] = useState<PlanWithItemsDto | null>(null);
  const [detailTab, setDetailTab] = useState<DetailTab>("items");
  const [itemTab, setItemTab] = useState(0);
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
      setItemTab(0);
      setDetailTab("items");
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
      setMessage(`Plan draft: ${p.name}`);
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
      });
      setMessage("Item añadido.");
      await selectPlan(selected.plan.id);
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
      setMessage(`Approved — habilita Automatic (no genera). ${p.status}`);
      await selectPlan(p.id);
      const list = await invokeListPlans();
      setPlans(list);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  const activeItem = selected?.items[itemTab];

  return (
    <section className="station">
      <header>
        <h2>Plans</h2>
        <p>
          Decide <strong>qué</strong> generar. Approve ≠ providers. Sin scroll: pestañas.
        </p>
      </header>

      {load === "unavailable" ? (
        <div className="placeholder-card fill">
          <p>IPC no disponible. Usa `pnpm dev`.</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <div className="station-body">
          <div className="tab-strip">
            <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>Planes</span>
            {plans.length === 0 ? (
              <span style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>vacío</span>
            ) : (
              plans.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  className={`tab-btn${selected?.plan.id === p.id ? " active" : ""}`}
                  onClick={() => void selectPlan(p.id)}
                >
                  {p.name} · {p.status}
                </button>
              ))
            )}
          </div>

          <div className="placeholder-card fill">
            {!selected ? (
              <form onSubmit={onCreate}>
                <h3>Nuevo plan draft</h3>
                <label style={labelStyle}>Nombre</label>
                <input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  style={inputStyle}
                  required
                />
                <button type="submit" disabled={busy} style={btnStyle}>
                  Create draft
                </button>
              </form>
            ) : (
              <>
                <div className="tab-strip">
                  <button
                    type="button"
                    className={`tab-btn${detailTab === "items" ? " active" : ""}`}
                    onClick={() => setDetailTab("items")}
                  >
                    Items ({selected.items.length})
                  </button>
                  <button
                    type="button"
                    className={`tab-btn${detailTab === "edit" ? " active" : ""}`}
                    onClick={() => setDetailTab("edit")}
                  >
                    {selected.plan.status === "draft" ? "Editar / Approve" : "Info"}
                  </button>
                  <span style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                    {selected.plan.name} · <code>{selected.plan.status}</code>
                  </span>
                </div>

                {detailTab === "items" ? (
                  selected.items.length === 0 ? (
                    <p>Sin items. Ve a Editar para añadir.</p>
                  ) : (
                    <>
                      <div className="tab-strip">
                        {selected.items.map((it, i) => (
                          <button
                            key={it.id}
                            type="button"
                            className={`tab-btn${itemTab === i ? " active" : ""}`}
                            onClick={() => setItemTab(i)}
                          >
                            #{i + 1}
                          </button>
                        ))}
                      </div>
                      {activeItem ? (
                        <p style={{ margin: 0, fontSize: "0.9rem" }}>
                          {activeItem.concept_key}/{activeItem.representation_key} —{" "}
                          {activeItem.status}
                        </p>
                      ) : null}
                    </>
                  )
                ) : selected.plan.status === "draft" ? (
                  <>
                    <form onSubmit={onAddItem}>
                      <label style={labelStyle}>concept_key</label>
                      <input
                        value={conceptKey}
                        onChange={(e) => setConceptKey(e.target.value)}
                        style={inputStyle}
                        required
                      />
                      <label style={labelStyle}>representation_key</label>
                      <input
                        value={repKey}
                        onChange={(e) => setRepKey(e.target.value)}
                        style={inputStyle}
                        required
                      />
                      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginTop: 8 }}>
                        <button type="submit" disabled={busy} style={btnStyle}>
                          Add item
                        </button>
                        <button
                          type="button"
                          disabled={busy || selected.items.length === 0}
                          style={btnStyle}
                          onClick={() => void onApprove()}
                        >
                          Approve plan
                        </button>
                      </div>
                    </form>
                    <form onSubmit={onCreate} style={{ marginTop: 16 }}>
                      <label style={labelStyle}>Otro plan nuevo</label>
                      <input
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        style={inputStyle}
                      />
                      <button type="submit" disabled={busy} style={btnStyle}>
                        Create draft
                      </button>
                    </form>
                  </>
                ) : (
                  <p>
                    Plan approved. Ejecuta en <strong>Factory → Automatic</strong>.
                  </p>
                )}
              </>
            )}
          </div>

          {message ? <p className="health msg-ok">{message}</p> : null}
          {error ? <p className="health msg-err">{error}</p> : null}
        </div>
      ) : null}
    </section>
  );
}

const inputStyle: CSSProperties = {
  width: "100%",
  maxWidth: "24rem",
  display: "block",
  padding: "0.4rem 0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  marginBottom: 6,
};

const labelStyle: CSSProperties = {
  display: "block",
  fontSize: "0.78rem",
  color: "var(--text-muted)",
  marginBottom: 3,
  marginTop: 6,
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
