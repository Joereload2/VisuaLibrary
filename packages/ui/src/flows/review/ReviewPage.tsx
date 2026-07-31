import { type CSSProperties, useCallback, useEffect, useState } from "react";
import {
  invokeApproveAsset,
  invokeEditMetadata,
  invokeListLibraryAssets,
  invokeListWaitingReview,
  invokeMarkDuplicate,
  invokeRegenerateAsset,
  invokeRejectAsset,
  type AssetDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable";

export function ReviewPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [items, setItems] = useState<AssetDto[]>([]);
  const [library, setLibrary] = useState<AssetDto[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [notes, setNotes] = useState("");
  const [orientation, setOrientation] = useState("");
  const [duplicateOf, setDuplicateOf] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [list, lib] = await Promise.all([
        invokeListWaitingReview(),
        invokeListLibraryAssets(),
      ]);
      setItems(list);
      setLibrary(lib);
      setSelectedId((prev) =>
        prev && list.some((a) => a.id === prev) ? prev : list[0]?.id ?? null,
      );
      setLoad("ready");
    } catch {
      setLoad("unavailable");
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const selected = items.find((a) => a.id === selectedId) ?? null;

  useEffect(() => {
    if (selected) {
      setNotes(selected.review_notes ?? "");
      setOrientation(selected.orientation ?? "");
      setDuplicateOf(selected.duplicate_of_asset_id ?? library[0]?.id ?? "");
    }
  }, [selected, library]);

  async function run(action: () => Promise<void>, okMsg: string) {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      await action();
      setMessage(okMsg);
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
        <h2>Review</h2>
        <p>
          Cola Waiting Review. Acciones: Approve · Reject · Edit metadata · Regenerate · Mark
          duplicate.
        </p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card">
          <p>Cargando cola…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card">
          <p>IPC no disponible. Abre con `pnpm dev` (Tauri).</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "minmax(0, 30%) minmax(0, 70%)",
            gap: "1rem",
          }}
        >
          <div className="placeholder-card">
            <h3>Cola ({items.length})</h3>
            {items.length === 0 ? (
              <p>Empty: nada en Waiting Review.</p>
            ) : (
              <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
                {items.map((a) => {
                  const isSel = a.id === selectedId;
                  return (
                    <li key={a.id} style={{ marginBottom: 6 }}>
                      <button
                        type="button"
                        onClick={() => setSelectedId(a.id)}
                        style={{
                          width: "100%",
                          textAlign: "left",
                          padding: "0.5rem 0.65rem",
                          borderRadius: 8,
                          border: isSel
                            ? "2px solid var(--accent)"
                            : "1px solid var(--border)",
                          background: isSel ? "var(--accent-soft)" : "transparent",
                          color: "var(--text)",
                          cursor: "pointer",
                        }}
                      >
                        <code style={{ fontSize: "0.8rem" }}>{a.id.slice(0, 18)}…</code>
                        <div style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>
                          {a.storage_path}
                        </div>
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>

          <div className="placeholder-card">
            <h3>Detalle</h3>
            {!selected ? (
              <p>Selecciona un asset de la cola.</p>
            ) : (
              <>
                <p>
                  <strong>status:</strong> {selected.status}
                </p>
                <p>
                  <strong>path:</strong> <code>{selected.storage_path}</code>
                </p>
                <p>
                  <strong>provider:</strong> {selected.provider ?? "—"}
                </p>

                <label style={labelStyle}>review notes</label>
                <input
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  style={inputStyle}
                />
                <label style={labelStyle}>orientation</label>
                <input
                  value={orientation}
                  onChange={(e) => setOrientation(e.target.value)}
                  style={inputStyle}
                  placeholder="landscape | portrait | any"
                />

                <div style={{ display: "flex", gap: "0.5rem", marginTop: "1rem", flexWrap: "wrap" }}>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void run(
                        () => invokeApproveAsset(selected.id).then(() => undefined),
                        "Aprobado — en Library.",
                      )
                    }
                    style={primaryBtn}
                  >
                    Approve
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void run(
                        () =>
                          invokeRejectAsset(selected.id, "rejected_from_ui").then(
                            () => undefined,
                          ),
                        "Rechazado.",
                      )
                    }
                    style={secondaryBtn}
                  >
                    Reject
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void run(
                        () =>
                          invokeEditMetadata({
                            assetId: selected.id,
                            reviewNotes: notes || undefined,
                            orientation: orientation || undefined,
                          }).then(() => undefined),
                        "Metadata actualizada (sigue en Waiting Review).",
                      )
                    }
                    style={secondaryBtn}
                  >
                    Edit metadata
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void run(
                        () =>
                          invokeRegenerateAsset(selected.id).then(() => undefined),
                        "Regenerado: actual superseded, nuevo en cola.",
                      )
                    }
                    style={secondaryBtn}
                  >
                    Regenerate
                  </button>
                </div>

                <div style={{ marginTop: "1rem" }}>
                  <label style={labelStyle}>Mark duplicate of (asset id)</label>
                  <input
                    value={duplicateOf}
                    onChange={(e) => setDuplicateOf(e.target.value)}
                    style={inputStyle}
                    placeholder="id de approved o waiting"
                  />
                  {library.length > 0 ? (
                    <select
                      value={duplicateOf}
                      onChange={(e) => setDuplicateOf(e.target.value)}
                      style={{ ...inputStyle, marginTop: 6 }}
                    >
                      <option value="">— elegir de Library —</option>
                      {library.map((a) => (
                        <option key={a.id} value={a.id}>
                          {a.id.slice(0, 20)}… {a.storage_path}
                        </option>
                      ))}
                    </select>
                  ) : null}
                  <button
                    type="button"
                    disabled={busy || !duplicateOf.trim()}
                    style={{ ...secondaryBtn, marginTop: 8 }}
                    onClick={() =>
                      void run(
                        () =>
                          invokeMarkDuplicate(selected.id, duplicateOf.trim()).then(
                            () => undefined,
                          ),
                        "Marcado duplicate.",
                      )
                    }
                  >
                    Mark duplicate
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
        </div>
      ) : null}
    </section>
  );
}

const primaryBtn: CSSProperties = {
  padding: "0.5rem 0.85rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 650,
  cursor: "pointer",
};

const secondaryBtn: CSSProperties = {
  ...primaryBtn,
  background: "transparent",
  color: "var(--text-muted)",
};

const inputStyle: CSSProperties = {
  width: "100%",
  maxWidth: "28rem",
  display: "block",
  padding: "0.4rem 0.55rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
};

const labelStyle: CSSProperties = {
  display: "block",
  marginTop: 10,
  marginBottom: 4,
  fontSize: "0.85rem",
  color: "var(--text-muted)",
};
