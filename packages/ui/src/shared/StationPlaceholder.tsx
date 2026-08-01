type Props = {
  title: string;
  purpose: string;
  notes?: string[];
};

/** Lightweight station shell for empty / offline views (not primary F1–F6 pages). */
export function StationPlaceholder({ title, purpose, notes = [] }: Props) {
  return (
    <section className="station">
      <header>
        <h2>{title}</h2>
        <p>{purpose}</p>
      </header>
      <div className="placeholder-card">
        <h3>Vista auxiliar</h3>
        <p>Las estaciones MVP viven en sus páginas de flujo (Factory, Review, Library, …).</p>
        {notes.length > 0 ? (
          <ul>
            {notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  );
}
