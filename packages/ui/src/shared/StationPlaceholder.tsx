type Props = {
  title: string;
  purpose: string;
  notes?: string[];
};

export function StationPlaceholder({ title, purpose, notes = [] }: Props) {
  return (
    <section className="station">
      <header>
        <h2>{title}</h2>
        <p>{purpose}</p>
      </header>
      <div className="placeholder-card">
        <h3>Placeholder de estación</h3>
        <p>
          Sin lógica de catálogo en Fase 1. Esta vista solo fija la navegación
          por flujos.
        </p>
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
