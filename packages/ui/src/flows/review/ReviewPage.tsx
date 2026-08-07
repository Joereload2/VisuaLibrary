import { type CSSProperties, useCallback, useEffect, useMemo, useState } from "react";
import {
  invokeApproveAsset,
  invokeAssetPreview,
  invokeEditMetadata,
  invokeListLibraryAssets,
  invokeListWaitingReview,
  invokeMarkDuplicate,
  invokeRegenerateAsset,
  invokeRejectAsset,
  type AssetDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable";
type DetailTab = "image" | "data";

type ReviewGroup = {
  key: string;
  label: string;
  assets: AssetDto[];
};

/** Group variants of the same concept+representation. */
function groupWaitingAssets(items: AssetDto[]): ReviewGroup[] {
  const order: string[] = [];
  const map = new Map<string, AssetDto[]>();
  const sorted = [...items].sort((a, b) => a.created_at.localeCompare(b.created_at));
  for (const a of sorted) {
    const key = `${a.concept_id}::${a.representation_id}`;
    if (!map.has(key)) {
      map.set(key, []);
      order.push(key);
    }
    map.get(key)!.push(a);
  }
  return order.map((key, gi) => {
    const assets = map.get(key)!;
    return {
      key,
      label:
        assets.length > 1
          ? `G${gi + 1} · ${assets.length} var`
          : `G${gi + 1}`,
      assets,
    };
  });
}

function lineLabel(a: AssetDto, indexInGroup: number, groupSize: number): string {
  const provider = a.provider ?? "—";
  if (groupSize > 1) {
    return `v${indexInGroup + 1} · ${provider}`;
  }
  return provider;
}

export function ReviewPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [items, setItems] = useState<AssetDto[]>([]);
  const [library, setLibrary] = useState<AssetDto[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [groupTab, setGroupTab] = useState(0);
  const [detailTab, setDetailTab] = useState<DetailTab>("image");
  const [notes, setNotes] = useState("");
  const [orientation, setOrientation] = useState("");
  const [duplicateOf, setDuplicateOf] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [thumbs, setThumbs] = useState<Record<string, string>>({});

  const groups = useMemo(() => groupWaitingAssets(items), [items]);

  const refresh = useCallback(async (preferId?: string | null) => {
    setError(null);
    try {
      const [list, lib] = await Promise.all([
        invokeListWaitingReview(),
        invokeListLibraryAssets(),
      ]);
      setItems(list);
      setLibrary(lib);
      setSelectedId((prev) => {
        if (preferId && list.some((a) => a.id === preferId)) return preferId;
        if (prev && list.some((a) => a.id === prev)) return prev;
        return list[0]?.id ?? null;
      });
      setLoad("ready");
    } catch {
      setLoad("unavailable");
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Sync group tab when selection or groups change
  useEffect(() => {
    if (!selectedId || groups.length === 0) {
      setGroupTab(0);
      return;
    }
    const gi = groups.findIndex((g) => g.assets.some((a) => a.id === selectedId));
    if (gi >= 0) setGroupTab(gi);
  }, [selectedId, groups]);

  // Thumbnails for current group only (visible at a glance)
  useEffect(() => {
    let cancelled = false;
    const g = groups[groupTab];
    if (!g) return;
    const missing = g.assets.filter((a) => !thumbs[a.id]);
    if (missing.length === 0) return;

    void (async () => {
      for (const a of missing) {
        if (cancelled) return;
        try {
          const p = await invokeAssetPreview(a.id);
          if (!cancelled) {
            setThumbs((prev) => (prev[a.id] ? prev : { ...prev, [a.id]: p.data_url }));
          }
        } catch {
          // leave empty
        }
      }
    })();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [groupTab, groups.map((g) => g.assets.map((a) => a.id).join(",")).join("|")]);

  const selected = items.find((a) => a.id === selectedId) ?? null;
  const selectedGroup = groups[groupTab] ?? null;

  useEffect(() => {
    if (selected) {
      setNotes(selected.review_notes ?? "");
      setOrientation(selected.orientation ?? "");
      setDuplicateOf(selected.duplicate_of_asset_id ?? library[0]?.id ?? "");
    }
  }, [selected, library]);

  useEffect(() => {
    if (!selected) {
      setPreviewUrl(null);
      setPreviewError(null);
      return;
    }
    let cancelled = false;
    const cached = thumbs[selected.id];
    setPreviewUrl(cached ?? null);
    setPreviewError(null);
    void invokeAssetPreview(selected.id)
      .then((p) => {
        if (!cancelled) {
          setPreviewUrl(p.data_url);
          setThumbs((prev) => ({ ...prev, [selected.id]: p.data_url }));
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setPreviewUrl(null);
          setPreviewError(String((err as { message?: string })?.message ?? err));
        }
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected?.id, selected?.storage_path, selected?.content_hash]);

  async function run(action: () => Promise<string | void>, okMsg: string) {
    if (busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const prefer = await action();
      setMessage(okMsg);
      await refresh(typeof prefer === "string" ? prefer : null);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  async function approveSelected(hotkey?: boolean) {
    if (!selected || busy) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const res = await invokeApproveAsset(selected.id);
      const tag = hotkey ? " (A)" : "";
      if (res.package_writeback) {
        const beats = res.package_writeback.written.map((w) => w.beat_id).join(", ");
        setMessage(
          `Aprobado — Library + package${beats ? ` [${beats}]` : ""}.${tag}\n${res.package_writeback.notes}`,
        );
      } else if (res.package_writeback_error) {
        setMessage(
          `Aprobado — en Library.${tag} Write-back package falló: ${res.package_writeback_error}`,
        );
      } else if (selected.package_path && selected.beat_id) {
        setMessage(
          `Aprobado — en Library.${tag} (sin write-back; revisa package_path/beat_id)`,
        );
      } else {
        setMessage(`Aprobado — en Library.${tag}`);
      }
      await refresh(null);
    } catch (err) {
      setError(String((err as { message?: string })?.message ?? err));
    } finally {
      setBusy(false);
    }
  }

  function selectGroup(gi: number) {
    const g = groups[gi];
    if (!g) return;
    setGroupTab(gi);
    const stillIn = selectedId && g.assets.some((a) => a.id === selectedId);
    if (!stillIn) setSelectedId(g.assets[0]?.id ?? null);
  }

  function selectSibling(delta: number) {
    if (!selectedGroup || !selected) return;
    const idx = selectedGroup.assets.findIndex((a) => a.id === selected.id);
    const next = selectedGroup.assets[idx + delta];
    if (next) setSelectedId(next.id);
  }

  const flatOrder = useMemo(
    () => groups.flatMap((g) => g.assets.map((a) => a.id)),
    [groups],
  );

  function selectInQueue(delta: number) {
    if (!selectedId || flatOrder.length === 0) return;
    const idx = flatOrder.indexOf(selectedId);
    const next = flatOrder[idx + delta];
    if (next) setSelectedId(next);
  }

  // A · R · G · ←→ · ↑↓ · D (toggle Imagen/Datos)
  useEffect(() => {
    if (load !== "ready") return;

    function isTypingTarget(el: EventTarget | null): boolean {
      if (!(el instanceof HTMLElement)) return false;
      const tag = el.tagName;
      return (
        tag === "INPUT" ||
        tag === "TEXTAREA" ||
        tag === "SELECT" ||
        el.isContentEditable
      );
    }

    function onKey(e: KeyboardEvent) {
      if (isTypingTarget(e.target)) return;
      if (busy) return;
      if (!selected) return;

      const key = e.key;
      const lower = key.toLowerCase();

      if (lower === "a" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        void approveSelected(true);
        return;
      }
      if (lower === "r" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        void run(
          () =>
            invokeRejectAsset(selected.id, "rejected_from_ui").then(() => undefined),
          "Rechazado. (R)",
        );
        return;
      }
      if (lower === "g" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        void run(
          () => invokeRegenerateAsset(selected.id).then((res) => res.asset_id),
          "Regenerado. (G)",
        );
        return;
      }
      if (lower === "d" && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        setDetailTab((t) => (t === "image" ? "data" : "image"));
        return;
      }
      if (key === "ArrowLeft") {
        e.preventDefault();
        selectSibling(-1);
        return;
      }
      if (key === "ArrowRight") {
        e.preventDefault();
        selectSibling(1);
        return;
      }
      if (key === "ArrowUp") {
        e.preventDefault();
        selectInQueue(-1);
        return;
      }
      if (key === "ArrowDown") {
        e.preventDefault();
        selectInQueue(1);
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [load, busy, selected, selectedGroup, flatOrder]);

  return (
    <section className="station">
      <header>
        <h2>Review</h2>
        <p>
          Imagen primero · sin scroll · pestañas.{" "}
          <kbd>A</kbd> Approve · <kbd>R</kbd> Reject · <kbd>G</kbd> Regen ·{" "}
          <kbd>←→</kbd> var · <kbd>↑↓</kbd> cola · <kbd>D</kbd> datos
        </p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card fill">
          <p>Cargando cola…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card fill">
          <p>IPC no disponible. Abre con `pnpm dev` (Tauri).</p>
        </div>
      ) : null}

      {load === "ready" ? (
        <div className="station-body">
          {/* Group tabs (no scrollable sidebar) */}
          <div className="tab-strip" role="tablist" aria-label="Grupos de la cola">
            <span style={{ fontSize: "0.78rem", color: "var(--text-muted)", marginRight: 4 }}>
              Cola {items.length}
            </span>
            {groups.length === 0 ? (
              <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                vacía — genera desde Factory
              </span>
            ) : (
              groups.map((g, gi) => (
                <button
                  key={g.key}
                  type="button"
                  role="tab"
                  aria-selected={gi === groupTab}
                  className={`tab-btn${gi === groupTab ? " active" : ""}`}
                  onClick={() => selectGroup(gi)}
                >
                  {g.label}
                </button>
              ))
            )}
          </div>

          {/* Variants of current group as chips */}
          {selectedGroup && selectedGroup.assets.length > 0 ? (
            <div className="chip-strip" style={{ marginBottom: 8 }}>
              {selectedGroup.assets.map((a, i) => {
                const isSel = a.id === selectedId;
                const thumb = thumbs[a.id];
                return (
                  <button
                    key={a.id}
                    type="button"
                    onClick={() => setSelectedId(a.id)}
                    style={{
                      ...chipBtn,
                      border: isSel ? "2px solid var(--accent)" : "1px solid var(--border)",
                      background: isSel ? "var(--accent-soft)" : "rgba(0,0,0,0.2)",
                    }}
                    title={lineLabel(a, i, selectedGroup.assets.length)}
                  >
                    <span style={miniThumb}>
                      {thumb ? (
                        <img src={thumb} alt="" style={miniThumbImg} />
                      ) : (
                        <span style={{ fontSize: "0.6rem", color: "var(--text-muted)" }}>…</span>
                      )}
                    </span>
                    <span style={{ fontSize: "0.75rem" }}>
                      {lineLabel(a, i, selectedGroup.assets.length)}
                    </span>
                  </button>
                );
              })}
            </div>
          ) : null}

          {/* Detail tabs: Imagen | Datos */}
          <div className="tab-strip">
            <button
              type="button"
              className={`tab-btn${detailTab === "image" ? " active" : ""}`}
              onClick={() => setDetailTab("image")}
            >
              Imagen
            </button>
            <button
              type="button"
              className={`tab-btn${detailTab === "data" ? " active" : ""}`}
              onClick={() => setDetailTab("data")}
              disabled={!selected}
            >
              Datos
            </button>
            {selected ? (
              <span style={{ fontSize: "0.78rem", color: "var(--text-muted)", marginLeft: 4 }}>
                {selected.provider ?? "—"} · {selected.status}
                {selected.width && selected.height
                  ? ` · ${selected.width}×${selected.height}`
                  : ""}
              </span>
            ) : null}
          </div>

          <div className="placeholder-card fill">
            {!selected ? (
              <p>Selecciona un grupo / variante arriba.</p>
            ) : detailTab === "image" ? (
              <>
                <div style={heroPreview}>
                  {previewUrl ? (
                    <img
                      key={selected.id}
                      src={previewUrl}
                      alt={`Preview ${selected.id}`}
                      style={heroImg}
                    />
                  ) : previewError ? (
                    <p style={{ color: "#f87171", fontSize: "0.85rem" }}>
                      Sin preview: {previewError}
                    </p>
                  ) : (
                    <p style={{ color: "var(--text-muted)" }}>Cargando imagen…</p>
                  )}
                </div>

                <div style={actionBar}>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => void approveSelected(false)}
                    style={approveBtn}
                  >
                    {selected.package_path && selected.beat_id
                      ? "Approve → Library + package"
                      : "Approve → Library"}
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
                          invokeRegenerateAsset(selected.id).then((r) => r.asset_id),
                        "Regenerado: misma ficha, nueva imagen.",
                      )
                    }
                    style={secondaryBtn}
                  >
                    Regenerate
                  </button>
                  {selectedGroup && selectedGroup.assets.length > 1 ? (
                    <span style={variantNav}>
                      <button
                        type="button"
                        style={ghostBtn}
                        onClick={() => selectSibling(-1)}
                        disabled={selectedGroup.assets[0]?.id === selected.id || busy}
                      >
                        ← var
                      </button>
                      <button
                        type="button"
                        style={ghostBtn}
                        onClick={() => selectSibling(1)}
                        disabled={
                          selectedGroup.assets[selectedGroup.assets.length - 1]?.id ===
                            selected.id || busy
                        }
                      >
                        var →
                      </button>
                    </span>
                  ) : null}
                </div>
              </>
            ) : (
              <div style={dataPanel}>
                <div style={dataGrid}>
                  <div>
                    <p style={metaLine}>
                      <strong>id:</strong> <code style={codeEllipsis}>{selected.id}</code>
                    </p>
                    <p style={metaLine}>
                      <strong>path:</strong>{" "}
                      <code style={codeEllipsis}>{selected.storage_path}</code>
                    </p>
                    <p style={metaLine}>
                      <strong>provider:</strong> {selected.provider ?? "—"}
                    </p>
                    {selected.package_path && selected.beat_id ? (
                      <p style={{ ...metaLine, color: "var(--accent)" }}>
                        <strong>package:</strong> beat <code>{selected.beat_id}</code>
                        {selected.package_id ? ` · ${selected.package_id}` : null}
                        <br />
                        <span style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>
                          Approve escribe a media/images/
                        </span>
                      </p>
                    ) : null}
                    {selected.prompt ? (
                      <p style={{ ...metaLine, fontSize: "0.8rem", color: "var(--text-muted)" }}>
                        <strong>prompt:</strong> {selected.prompt.slice(0, 200)}
                        {selected.prompt.length > 200 ? "…" : ""}
                      </p>
                    ) : null}
                  </div>
                  <div>
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
                          "Metadata actualizada.",
                        )
                      }
                      style={{ ...secondaryBtn, marginTop: 8 }}
                    >
                      Guardar metadata
                    </button>
                  </div>
                  <div>
                    <label style={labelStyle}>Mark duplicate of</label>
                    <select
                      value={duplicateOf}
                      onChange={(e) => setDuplicateOf(e.target.value)}
                      style={inputStyle}
                    >
                      <option value="">— elegir de Library —</option>
                      {library.map((a) => (
                        <option key={a.id} value={a.id}>
                          {a.id.slice(0, 16)}… {a.storage_path}
                        </option>
                      ))}
                    </select>
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
                </div>
              </div>
            )}
          </div>

          {message ? <p className="health msg-ok">{message}</p> : null}
          {error ? <p className="health msg-err">{error}</p> : null}
        </div>
      ) : null}
    </section>
  );
}

const chipBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 6,
  padding: "0.25rem 0.5rem 0.25rem 0.25rem",
  borderRadius: 10,
  color: "var(--text)",
  cursor: "pointer",
};

const miniThumb: CSSProperties = {
  width: 32,
  height: 32,
  borderRadius: 6,
  border: "1px solid var(--border)",
  background: "rgba(0,0,0,0.35)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  overflow: "hidden",
  flexShrink: 0,
};

const miniThumbImg: CSSProperties = {
  width: "100%",
  height: "100%",
  objectFit: "cover",
  imageRendering: "pixelated",
};

const heroPreview: CSSProperties = {
  flex: "1 1 auto",
  minHeight: 0,
  padding: "0.5rem",
  borderRadius: 10,
  border: "1px solid var(--border)",
  background: "rgba(0,0,0,0.35)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  marginBottom: 8,
  overflow: "hidden",
};

const heroImg: CSSProperties = {
  maxWidth: "100%",
  maxHeight: "100%",
  width: "auto",
  height: "auto",
  objectFit: "contain",
  borderRadius: 6,
};

const actionBar: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  gap: 8,
  alignItems: "center",
  flexShrink: 0,
};

const approveBtn: CSSProperties = {
  padding: "0.55rem 1rem",
  borderRadius: 10,
  border: "1px solid var(--accent)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 750,
  fontSize: "0.95rem",
  cursor: "pointer",
};

const secondaryBtn: CSSProperties = {
  padding: "0.4rem 0.75rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "transparent",
  color: "var(--text-muted)",
  fontWeight: 600,
  cursor: "pointer",
};

const ghostBtn: CSSProperties = {
  ...secondaryBtn,
  padding: "0.3rem 0.55rem",
  fontSize: "0.78rem",
};

const variantNav: CSSProperties = {
  display: "inline-flex",
  gap: 6,
  marginLeft: "auto",
};

const dataPanel: CSSProperties = {
  flex: 1,
  minHeight: 0,
  overflow: "hidden",
  display: "flex",
  flexDirection: "column",
};

const dataGrid: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
  gap: "0.75rem 1.25rem",
  minHeight: 0,
};

const metaLine: CSSProperties = {
  margin: "0.25rem 0",
  fontSize: "0.85rem",
};

const codeEllipsis: CSSProperties = {
  fontSize: "0.78rem",
  wordBreak: "break-all",
};

const inputStyle: CSSProperties = {
  width: "100%",
  display: "block",
  padding: "0.35rem 0.5rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
};

const labelStyle: CSSProperties = {
  display: "block",
  marginTop: 6,
  marginBottom: 3,
  fontSize: "0.8rem",
  color: "var(--text-muted)",
};
