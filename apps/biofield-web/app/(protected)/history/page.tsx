const placeholderHistory = [
  {
    id: "reading-sample-001",
    title: "History shell ready",
    copy: "Persisted biofield readings will land here once capture upload and reading save routes are wired.",
  },
  {
    id: "reading-sample-002",
    title: "Reading detail route ready",
    copy: "The route structure is in place so later waves can plug real history and detail data into stable paths.",
  },
];

export default function HistoryPage() {
  return (
    <section className="biofield-panel">
      <p className="biofield-eyebrow">History scaffold</p>
      <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
        Readings
      </h2>
      <p className="biofield-copy">
        This route is intentionally thin in Wave 1.1. It exists so later API and persistence work lands into a stable user-facing shell.
      </p>
      <ul className="biofield-list">
        {placeholderHistory.map((entry) => (
          <li key={entry.id}>
            <p className="biofield-kicker">{entry.id}</p>
            <p className="biofield-metric">{entry.title}</p>
            <p className="biofield-copy">{entry.copy}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
