import { NavLink, Navigate, Route, Routes } from "react-router-dom";
import { FactoryPage } from "../flows/factory/FactoryPage";
import { ReviewPage } from "../flows/review/ReviewPage";
import { LibraryPage } from "../flows/library/LibraryPage";
import { CoveragePage } from "../flows/coverage/CoveragePage";
import { PlansPage } from "../flows/plans/PlansPage";
import { SettingsPage } from "../flows/settings/SettingsPage";

const stations = [
  { to: "/factory", label: "Factory" },
  { to: "/review", label: "Review" },
  { to: "/library", label: "Library" },
  { to: "/coverage", label: "Coverage" },
  { to: "/plans", label: "Plans" },
  { to: "/settings", label: "Settings" },
] as const;

export function App() {
  return (
    <div className="app-shell">
      <nav className="nav" aria-label="Estaciones de trabajo">
        <div className="brand">
          <h1>Visual Library</h1>
          <p>F1–F6 · local MVP</p>
        </div>
        {stations.map((station) => (
          <NavLink
            key={station.to}
            to={station.to}
            className={({ isActive }) => (isActive ? "active" : undefined)}
          >
            {station.label}
          </NavLink>
        ))}
      </nav>
      <main className="main">
        <Routes>
          <Route path="/" element={<Navigate to="/library" replace />} />
          <Route path="/factory/*" element={<FactoryPage />} />
          <Route path="/review" element={<ReviewPage />} />
          <Route path="/library" element={<LibraryPage />} />
          <Route path="/coverage" element={<CoveragePage />} />
          <Route path="/plans" element={<PlansPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/library" replace />} />
        </Routes>
      </main>
    </div>
  );
}
