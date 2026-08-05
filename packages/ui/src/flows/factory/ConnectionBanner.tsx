import { type CSSProperties, useCallback, useEffect, useState } from "react";
import {
  invokeGetIntegrationConfig,
  invokeListImageProviders,
  invokeProbeOmniroute,
  invokeUpdateIntegrationConfig,
  type ImageProvider,
  type IntegrationConfigDto,
  type OmniRouteProbeResult,
} from "../../shared/ipc/client";

type Level = "ok" | "warn" | "err" | "loading" | "idle";

/**
 * Auto-probes OmniRoute and applies safe defaults when the gateway is up.
 * No user setup required beyond having OmniRoute running.
 */
export function ConnectionBanner() {
  const [level, setLevel] = useState<Level>("loading");
  const [text, setText] = useState("Comprobando conexión…");
  const [detail, setDetail] = useState<string | null>(null);
  const [probe, setProbe] = useState<OmniRouteProbeResult | null>(null);
  const [busy, setBusy] = useState(false);

  const run = useCallback(async (deep: boolean) => {
    setBusy(true);
    setLevel("loading");
    setText(deep ? "Probando OmniRoute (imagen + chat)…" : "Comprobando OmniRoute…");
    setDetail(null);
    try {
      const [cfg, providers] = await Promise.all([
        invokeGetIntegrationConfig(),
        invokeListImageProviders().catch(() => [] as ImageProvider[]),
      ]);
      const omni = providers.find((p) => p.id === "omniroute");

      // Quick models probe first; deep = image+chat
      const r = await invokeProbeOmniroute({
        tryImage: deep,
        tryChat: deep,
      });
      setProbe(r);

      if (!r.overall_ok) {
        setLevel("warn");
        setText(
          "OmniRoute no está disponible — puedes seguir con stub/heurística. Arranca el gateway para imágenes reales y needs con Claude.",
        );
        setDetail(r.models_detail);
        return;
      }

      // Gateway up → auto-wire defaults so the user does not open Settings.
      await autoWireWhenReady(cfg, r, deep);

      if (deep) {
        if (r.images_ok && r.chat_ok) {
          setLevel("ok");
          setText("OmniRoute listo: imagen + chat. Factory usará omniroute por defecto.");
        } else if (r.images_ok) {
          setLevel("warn");
          setText("OmniRoute: imagen OK, chat falló — needs usarán heurística hasta arreglar chat model.");
          setDetail(r.chat_detail);
        } else if (r.chat_ok) {
          setLevel("warn");
          setText("OmniRoute: chat OK, imagen falló — elige stub o revisa image model.");
          setDetail(r.images_detail);
        } else {
          setLevel("warn");
          setText("OmniRoute alcanzable (models), pero imagen y chat fallaron.");
          setDetail(`${r.images_detail} · ${r.chat_detail}`);
        }
      } else {
        setLevel("ok");
        setText(
          omni?.status === "ready" || r.models_ok
            ? "OmniRoute alcanzable. Pulsa «Probar imagen» antes del primer Submit real."
            : "Conexión parcial.",
        );
        setDetail(r.models_detail);
      }
    } catch (e) {
      setLevel("err");
      setText("No se pudo comprobar la conexión (IPC).");
      setDetail(String((e as { message?: string })?.message ?? e));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void run(false);
  }, [run]);

  return (
    <div
      style={{
        ...bannerBase,
        borderColor:
          level === "ok"
            ? "var(--accent)"
            : level === "err"
              ? "#f87171"
              : level === "warn"
                ? "#fbbf24"
                : "var(--border)",
        background:
          level === "ok"
            ? "var(--accent-soft)"
            : level === "err"
              ? "rgba(248,113,113,0.08)"
              : level === "warn"
                ? "rgba(251,191,36,0.08)"
                : "rgba(255,255,255,0.03)",
      }}
      role="status"
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontWeight: 650, fontSize: "0.88rem" }}>{text}</div>
        {detail ? (
          <div style={{ fontSize: "0.75rem", color: "var(--text-muted)", marginTop: 4 }}>
            {detail}
          </div>
        ) : null}
        {probe ? (
          <div style={{ fontSize: "0.72rem", color: "var(--text-muted)", marginTop: 4 }}>
            models {probe.models_ok ? "✓" : "✗"}
            {probe.images_detail !== "omitido" && probe.images_detail !== "skipped"
              ? ` · images ${probe.images_ok ? "✓" : "✗"}`
              : ""}
            {probe.chat_detail !== "omitido" && probe.chat_detail !== "skipped"
              ? ` · chat ${probe.chat_ok ? "✓" : "✗"}`
              : ""}
          </div>
        ) : null}
      </div>
      <div style={{ display: "flex", gap: 6, flexShrink: 0, flexWrap: "wrap" }}>
        <button
          type="button"
          disabled={busy}
          style={btn}
          onClick={() => void run(false)}
        >
          Recomprobar
        </button>
        <button
          type="button"
          disabled={busy}
          style={btn}
          onClick={() => void run(true)}
        >
          Probar imagen
        </button>
      </div>
    </div>
  );
}

/** When gateway is up, set omniroute as script AI + default image without user navigation. */
async function autoWireWhenReady(
  cfg: IntegrationConfigDto,
  r: OmniRouteProbeResult,
  deep: boolean,
) {
  if (!r.overall_ok) return;
  const enabled = new Set(cfg.enabled_image_providers ?? []);
  enabled.add("stub");
  enabled.add("omniroute");

  const patch: Parameters<typeof invokeUpdateIntegrationConfig>[0] = {
    enabled_image_providers: Array.from(enabled),
    // Prefer OmniRoute for needs when gateway answers models.
    script_ai_provider: "omniroute",
    // Never force silent stub fallback.
    allow_stub_fallback_on_image_error: false,
  };

  // Only switch default image when image path actually works (deep) or user still on stub.
  if (deep && r.images_ok) {
    patch.default_image_provider = "omniroute";
  } else if (!deep && (cfg.default_image_provider === "stub" || !cfg.default_image_provider)) {
    // Soft preference: still stub until image proven, but omniroute enabled for needs.
  }

  try {
    await invokeUpdateIntegrationConfig(patch);
  } catch {
    // Non-fatal: banner still shows connectivity.
  }
}

const bannerBase: CSSProperties = {
  display: "flex",
  gap: 12,
  alignItems: "flex-start",
  flexWrap: "wrap",
  padding: "0.55rem 0.75rem",
  borderRadius: 10,
  border: "1px solid var(--border)",
  marginBottom: 10,
  flexShrink: 0,
};

const btn: CSSProperties = {
  padding: "0.3rem 0.65rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "transparent",
  color: "var(--text-muted)",
  fontWeight: 600,
  fontSize: "0.78rem",
  cursor: "pointer",
};
