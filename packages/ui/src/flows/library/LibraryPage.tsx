import { type CSSProperties, FormEvent, useCallback, useEffect, useState } from "react";
import {
  invokeEnsureConcept,
  invokeEnsureRepresentation,
  invokeGenerateStub,
  invokeHealth,
  invokeListConcepts,
  invokeListLibraryAssets,
  invokeListRepresentations,
  type AssetDto,
  type ConceptDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable";

export function LibraryPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [concepts, setConcepts] = useState<ConceptDto[]>([]);
  const [assets, setAssets] = useState<AssetDto[]>([]);
  const [health, setHealth] = useState<string>("…");
  const [key, setKey] = useState("demo-subject");
  const [name, setName] = useState("Demo subject");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [list, approved, h] = await Promise.all([
        invokeListConcepts(),
        invokeListLibraryAssets(),
        invokeHealth(),
      ]);
      setConcepts(list);
      setAssets(approved);
      setHealth(h);
      setLoad("ready");
    } catch {
      setLoad("unavailable");
      setHealth("UI only (Tauri IPC no disponible en browser puro)");
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function onEnsure(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const c = await invokeEnsureConcept(key.trim(), name.trim() || key.trim());
      setMessage(`Concepto listo: ${c.key}`);
      await refresh();
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function onGenerateStub() {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const c = await invokeEnsureConcept(key.trim() || "demo-subject", name.trim() || "Demo");
      let reps = await invokeListRepresentations(c.id);
      let rep = reps[0];
      if (!rep) {
        rep = await invokeEnsureRepresentation(c.id, "default", "Default", "any");
        reps = [rep];
      }
      const gen = await invokeGenerateStub({
        conceptId: c.id,
        representationId: rep.id,
        prompt: "foundation stub",
        idempotencyKey: `stub-${c.id}-${rep.id}-${Date.now()}`,
      });
      setMessage(
        `Generate stub → job ${gen.job_status}, asset ${gen.asset_status}. Ve a Review (no está en Library).`,
      );
      await refresh();
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="station">
      <header>
        <h2>Library</h2>
        <p>
          Solo assets <strong>approved</strong>. Generar stub manda a Waiting Review (nunca
          directo a Library).
        </p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card">
          <p>Cargando…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card">
          <h3>Modo UI-only</h3>
          <p>{health}</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <>
          <div className="placeholder-card">
            <h3>Assets approved ({assets.length})</h3>
            {assets.length === 0 ? (
              <p>Empty: aún no hay recursos en Library. Aprueba algo en Review.</p>
            ) : (
              <ul>
                {assets.map((a) => (
                  <li key={a.id}>
                    <code>{a.id.slice(0, 16)}…</code> — {a.storage_path}{" "}
                    <span style={{ color: "var(--text-muted)" }}>({a.status})</span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="placeholder-card" style={{ marginTop: "1rem" }}>
            <h3>Conceptos ({concepts.length})</h3>
            {concepts.length === 0 ? (
              <p>Empty: crea un concepto y/o genera stub.</p>
            ) : (
              <ul>
                {concepts.map((c) => (
                  <li key={c.id}>
                    <code>{c.key}</code> — {c.name}
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="placeholder-card" style={{ marginTop: "1rem" }}>
            <h3>Ensure concept + generate stub</h3>
            <form onSubmit={onEnsure}>
              <label htmlFor="c_key">key</label>
              <input
                id="c_key"
                value={key}
                onChange={(ev) => setKey(ev.target.value)}
                style={inputStyle}
                required
              />
              <label htmlFor="c_name" style={{ display: "block", marginTop: 8 }}>
                name
              </label>
              <input
                id="c_name"
                value={name}
                onChange={(ev) => setName(ev.target.value)}
                style={inputStyle}
              />
              <div style={{ marginTop: "0.75rem", display: "flex", gap: 8, flexWrap: "wrap" }}>
                <button type="submit" disabled={busy} style={btnStyle}>
                  Ensure concept
                </button>
                <button
                  type="button"
                  disabled={busy}
                  style={btnStyle}
                  onClick={() => void onGenerateStub()}
                >
                  Generate stub → Waiting Review
                </button>
              </div>
            </form>
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
        </>
      ) : null}

      <p className="health">
        Backend: <code>{health}</code>
      </p>
    </section>
  );
}

const inputStyle: CSSProperties = {
  width: "100%",
  maxWidth: "28rem",
  padding: "0.5rem 0.65rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  display: "block",
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
