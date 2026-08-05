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
type LibTab = "assets" | "concepts" | "tools";

export function LibraryPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [tab, setTab] = useState<LibTab>("assets");
  const [concepts, setConcepts] = useState<ConceptDto[]>([]);
  const [assets, setAssets] = useState<AssetDto[]>([]);
  const [assetTab, setAssetTab] = useState(0);
  const [conceptTab, setConceptTab] = useState(0);
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

  const activeAsset = assets[assetTab];
  const activeConcept = concepts[conceptTab];

  return (
    <section className="station">
      <header>
        <h2>Library</h2>
        <p>
          Solo <strong>approved</strong>. Stub → Waiting Review (nunca directo a Library). Sin
          scroll.
        </p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card fill">
          <p>Cargando…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card fill">
          <h3>Modo UI-only</h3>
          <p>{health}</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <div className="station-body">
          <div className="tab-strip">
            <button
              type="button"
              className={`tab-btn${tab === "assets" ? " active" : ""}`}
              onClick={() => setTab("assets")}
            >
              Assets ({assets.length})
            </button>
            <button
              type="button"
              className={`tab-btn${tab === "concepts" ? " active" : ""}`}
              onClick={() => setTab("concepts")}
            >
              Conceptos ({concepts.length})
            </button>
            <button
              type="button"
              className={`tab-btn${tab === "tools" ? " active" : ""}`}
              onClick={() => setTab("tools")}
            >
              Tools
            </button>
          </div>

          <div className="placeholder-card fill">
            {tab === "assets" ? (
              assets.length === 0 ? (
                <p>Empty: aprueba algo en Review.</p>
              ) : (
                <>
                  <div className="tab-strip">
                    {assets.map((a, i) => (
                      <button
                        key={a.id}
                        type="button"
                        className={`tab-btn${assetTab === i ? " active" : ""}`}
                        onClick={() => setAssetTab(i)}
                      >
                        #{i + 1}
                      </button>
                    ))}
                  </div>
                  {activeAsset ? (
                    <p style={{ margin: 0, fontSize: "0.9rem" }}>
                      <code>{activeAsset.id}</code>
                      <br />
                      {activeAsset.storage_path} · {activeAsset.status}
                      {activeAsset.provider ? ` · ${activeAsset.provider}` : ""}
                    </p>
                  ) : null}
                </>
              )
            ) : null}

            {tab === "concepts" ? (
              concepts.length === 0 ? (
                <p>Empty: crea un concepto en Tools.</p>
              ) : (
                <>
                  <div className="tab-strip">
                    {concepts.map((c, i) => (
                      <button
                        key={c.id}
                        type="button"
                        className={`tab-btn${conceptTab === i ? " active" : ""}`}
                        onClick={() => setConceptTab(i)}
                      >
                        {c.key}
                      </button>
                    ))}
                  </div>
                  {activeConcept ? (
                    <p style={{ margin: 0 }}>
                      <code>{activeConcept.key}</code> — {activeConcept.name}
                    </p>
                  ) : null}
                </>
              )
            ) : null}

            {tab === "tools" ? (
              <form onSubmit={onEnsure}>
                <h3>Ensure concept + generate stub</h3>
                <label htmlFor="c_key" style={{ fontSize: "0.8rem", color: "var(--text-muted)" }}>
                  key
                </label>
                <input
                  id="c_key"
                  value={key}
                  onChange={(ev) => setKey(ev.target.value)}
                  style={inputStyle}
                  required
                />
                <label
                  htmlFor="c_name"
                  style={{ display: "block", marginTop: 8, fontSize: "0.8rem", color: "var(--text-muted)" }}
                >
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
                    Generate stub → Review
                  </button>
                </div>
              </form>
            ) : null}
          </div>

          {message ? <p className="health msg-ok">{message}</p> : null}
          {error ? <p className="health msg-err">{error}</p> : null}
          <p className="health">
            Backend: <code>{health}</code>
          </p>
        </div>
      ) : null}
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
