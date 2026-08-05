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
  const [issueTab, setIssueTab] = useState(0);

  const refresh = useCallback(async () => {
    try {
      const r = await invokeCoverageReport();
      setReport(r);
      setLoad("ready");
      setError(null);
      setIssueTab(0);
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
  const issue = report.issues[issueTab];

  return (
    <section className="station">
      <header>
        <h2>Coverage</h2>
        <p>Diagnóstico accionable. Sin scroll: un issue por pestaña.</p>
      </header>

      <div className="station-body">
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(6, minmax(0, 1fr))",
            gap: "0.45rem",
            flexShrink: 0,
            marginBottom: 8,
          }}
        >
          <Stat label="Conceptos" value={s.concepts_total} />
          <Stat label="Under" value={s.concepts_under_covered} />
          <Stat label="Sin reps" value={s.concepts_missing_representations} />
          <Stat label="Waiting" value={s.waiting_review} />
          <Stat label="Approved" value={s.approved_assets} />
          <Stat label="Plans" value={s.approved_plans} />
        </div>

        <div className="placeholder-card fill">
          <div className="tab-strip">
            <span style={{ fontSize: "0.78rem", color: "var(--text-muted)" }}>
              Issues ({report.issues.length})
            </span>
            {report.issues.length === 0 ? (
              <span style={{ fontSize: "0.85rem", color: "var(--text-muted)" }}>ninguno</span>
            ) : (
              report.issues.map((iss, i) => (
                <button
                  key={`${iss.code}-${i}`}
                  type="button"
                  className={`tab-btn${issueTab === i ? " active" : ""}`}
                  onClick={() => setIssueTab(i)}
                >
                  #{i + 1}
                </button>
              ))
            )}
            <button type="button" className="tab-btn" onClick={() => void refresh()}>
              Refrescar
            </button>
          </div>

          {report.issues.length === 0 ? (
            <p>No hay issues con el catálogo actual.</p>
          ) : issue ? (
            <IssuePanel issue={issue} />
          ) : null}

          {error ? <p className="health msg-err">{error}</p> : null}
        </div>
      </div>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: number }) {
  return (
    <div
      style={{
        border: "1px solid var(--border)",
        borderRadius: 8,
        padding: "0.4rem 0.5rem",
        background: "rgba(255,255,255,0.02)",
        minWidth: 0,
      }}
    >
      <div style={{ fontSize: "1.1rem", fontWeight: 650 }}>{value}</div>
      <div
        style={{
          color: "var(--text-muted)",
          fontSize: "0.68rem",
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {label}
      </div>
    </div>
  );
}

function IssuePanel({ issue }: { issue: CoverageIssue }) {
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
    <div style={{ minHeight: 0, overflow: "hidden" }}>
      <div style={{ fontWeight: 600, marginBottom: 6 }}>
        <span style={severityBadge(issue.severity)}>{issue.severity}</span> {issue.title}
      </div>
      <p style={{ color: "var(--text-muted)", fontSize: "0.88rem", margin: "0 0 8px" }}>
        {issue.detail}
      </p>
      <code style={{ fontSize: "0.72rem", color: "var(--text-muted)" }}>{issue.code}</code>
      <div style={{ marginTop: 12 }}>
        <Link to={href} style={ctaStyle}>
          Ir a {issue.cta_flow}
        </Link>
      </div>
    </div>
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
