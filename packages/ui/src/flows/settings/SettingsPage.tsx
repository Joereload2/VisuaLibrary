import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  invokeGetAppPaths,
  invokeGetSettings,
  invokeSetMediaRoot,
  type AppPathsDto,
  type SettingsDto,
} from "../../shared/ipc/client";

type LoadState = "loading" | "ready" | "unavailable" | "error";

export function SettingsPage() {
  const [load, setLoad] = useState<LoadState>("loading");
  const [paths, setPaths] = useState<AppPathsDto | null>(null);
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [mediaRootInput, setMediaRootInput] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [p, s] = await Promise.all([invokeGetAppPaths(), invokeGetSettings()]);
      setPaths(p);
      setSettings(s);
      setMediaRootInput(s.media_root);
      setLoad("ready");
    } catch {
      setLoad("unavailable");
      setMessage(
        "IPC no disponible (abre la app con pnpm dev / Tauri). En browser puro solo se ve el placeholder.",
      );
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function onSave(e: FormEvent) {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      const s = await invokeSetMediaRoot(mediaRootInput.trim());
      setSettings(s);
      setMediaRootInput(s.media_root);
      setMessage("media_root guardado. Se conserva al reiniciar la app.");
      const p = await invokeGetAppPaths();
      setPaths(p);
    } catch (err) {
      const msg =
        typeof err === "object" && err && "message" in err
          ? String((err as { message: string }).message)
          : String(err);
      setError(msg);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="station">
      <header>
        <h2>Settings</h2>
        <p>Configuración local: paths de app data, base SQLite y media root.</p>
      </header>

      {load === "loading" ? (
        <div className="placeholder-card">
          <p>Cargando configuración…</p>
        </div>
      ) : null}

      {load === "unavailable" ? (
        <div className="placeholder-card">
          <h3>Modo UI-only</h3>
          <p>{message}</p>
        </div>
      ) : null}

      {load === "ready" && paths && settings ? (
        <div className="placeholder-card">
          <h3>Rutas de la aplicación</h3>
          <ul>
            <li>
              <strong>App data:</strong> <code>{paths.app_data_root}</code>
            </li>
            <li>
              <strong>SQLite:</strong> <code>{paths.db_path}</code>
            </li>
            <li>
              <strong>Exports:</strong> <code>{paths.exports_dir}</code>
            </li>
            <li>
              <strong>Tmp jobs:</strong> <code>{paths.tmp_dir}</code>
            </li>
          </ul>

          <form onSubmit={onSave} style={{ marginTop: "1.25rem" }}>
            <label htmlFor="media_root" style={{ display: "block", marginBottom: "0.35rem" }}>
              Media root
            </label>
            <input
              id="media_root"
              value={mediaRootInput}
              onChange={(ev) => setMediaRootInput(ev.target.value)}
              style={{
                width: "100%",
                maxWidth: "40rem",
                padding: "0.5rem 0.65rem",
                borderRadius: 8,
                border: "1px solid var(--border)",
                background: "var(--bg)",
                color: "var(--text)",
              }}
              autoComplete="off"
              spellCheck={false}
            />
            <div style={{ marginTop: "0.75rem", display: "flex", gap: "0.75rem", alignItems: "center" }}>
              <button
                type="submit"
                disabled={saving}
                style={{
                  padding: "0.45rem 0.9rem",
                  borderRadius: 8,
                  border: "1px solid var(--border)",
                  background: "var(--accent-soft)",
                  color: "var(--accent)",
                  fontWeight: 600,
                  cursor: saving ? "wait" : "pointer",
                }}
              >
                {saving ? "Guardando…" : "Guardar media root"}
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
      ) : null}

      {load === "error" && error ? (
        <div className="placeholder-card">
          <p>{error}</p>
        </div>
      ) : null}
    </section>
  );
}
