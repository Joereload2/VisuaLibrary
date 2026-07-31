import { type CSSProperties, useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  invokeCoverageReport,
  type CoverageIssue,
  type CoverageReport,
} from "../../shared/ipc/client";

export function CoveragePage() {
  const [report, setReport] = useState<CoverageReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [load, setLoad] = useState<"loading" | "ready" | "unavailable">("loading");

  const refresh = useCallback(async () => {
    try {
      const r = await invokeCoverageReport();
      setReport(r);
      setLoad("ready");
      setError(null);
    } catch {
      setLoad("unavailable");
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (load === "loading") {
    return (
      <section className="station">
        <header>
          <h2>Coverage</h2>
          <p>Cargando diagnóstico…</p>
        </header>
      </section>
    );
  }

  if (load === "unavailable" || !report) {
    return (
      <section className="station">
        <header>
          <h2>Coverage</h2>
          <p>IPC no disponible. Usa `pnpm dev`.</p>
        </header>
      </section>
    );
  }

  const s = report.summary;

  return (
    <section className="station">
      <header>
        <h2>Coverage</h2>
        <p>Problemas accionables de cobertura conceptual (no solo números).</p>
      </header>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(9rem, 1fr))",
          gap: "0.75rem",
          marginBottom: "1.25rem",
        }}
      >
        <Stat label="Conceptos" value={s.concepts_total} />
        <Stat label="Under-covered" value={s.concepts_under_covered} />
        <Stat label="Sin reps" value={s.concepts_missing_representations} />
        <Stat label="Waiting Review" value={s.waiting_review} />
        <Stat label="Approved" value={s.approved_assets} />
        <Stat label="Plans approved" value={s.approved_plans} />
      </div>

      <div className="placeholder-card">
        <h3>Issues ({report.issues.length})</h3>
        {report.issues.length === 0 ? (
          <p>No hay issues detectados con el catálogo actual.</p>
        ) : (
          <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
            {report.issues.map((issue, i) => (
              <IssueRow key={`${issue.code}-${i}`} issue={issue} />
            ))}
          </ul>
        )}
        {error ? (
          <p className="health" style={{ color: "#f87171" }}>
            {error}
          </p>
        ) : null}
        <button type="button" style={btnStyle} onClick={() => void refresh()}>
          Refrescar
        </button>
      </div>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div
      style={{
        border: "1px solid var(--border)",
        borderRadius: 10,
        padding: "0.75rem",
        background: "rgba(255,255,255,0.02)",
      }}
    >
      <div style={{ fontSize: "1.35rem", fontWeight: 650 }}>{value}</div>
      <div style={{ color: "var(--text-muted)", fontSize: "0.8rem" }}>{label}</div>
    </div>
  );
}

function IssueRow({ issue }: { issue: CoverageIssue }) {
  const href =
    issue.cta_flow === "review"
      ? "/review"
      : issue.cta_flow === "plans"
        ? "/plans"
        : issue.cta_flow === "factory"
          ? "/factory/manual"
          : issue.cta_flow === "library"
            ? "/library"
            : "/library";

  return (
    <li
      style={{
        borderBottom: "1px solid var(--border)",
        padding: "0.75rem 0",
        display: "grid",
        gridTemplateColumns: "minmax(0, 1fr) auto",
        gap: "0.75rem",
        alignItems: "start",
      }}
    >
      <div>
        <div style={{ fontWeight: 600 }}>
          <span style={severityBadge(issue.severity)}>{issue.severity}</span> {issue.title}
        </div>
        <div style={{ color: "var(--text-muted)", fontSize: "0.9rem", marginTop: 4 }}>
          {issue.detail}
        </div>
        <code style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>{issue.code}</code>
      </div>
      <Link to={href} style={ctaStyle}>
        Ir a {issue.cta_flow}
      </Link>
    </li>
  );
}

function severityBadge(sev: string): CSSProperties {
  const color =
    sev === "high" ? "#f87171" : sev === "medium" ? "#fbbf24" : "var(--text-muted)";
  return {
    display: "inline-block",
    marginRight: 8,
    fontSize: "0.7rem",
    textTransform: "uppercase",
    color,
    border: `1px solid ${color}`,
    borderRadius: 4,
    padding: "0 0.35rem",
  };
}

const ctaStyle: CSSProperties = {
  padding: "0.35rem 0.65rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  color: "var(--accent)",
  fontSize: "0.85rem",
  whiteSpace: "nowrap",
};

const btnStyle: CSSProperties = {
  marginTop: 12,
  padding: "0.45rem 0.9rem",
  borderRadius: 8,
  border: "1px solid var(--border)",
  background: "var(--accent-soft)",
  color: "var(--accent)",
  fontWeight: 600,
  cursor: "pointer",
};
