import { BIOFIELD_ENGINE_ID } from "@selemene/biofield-domain";

const viewerCards = [
  {
    label: "Runtime split",
    value: "Local browser hot path",
    copy: "Camera, segmentation, and PIP rendering stay in the browser for low-latency operation.",
  },
  {
    label: "Persisted engine id",
    value: BIOFIELD_ENGINE_ID,
    copy: "Completed captures will persist through the shared readings table using the frozen biofield engine id.",
  },
  {
    label: "Next batch",
    value: "Session + capture flow",
    copy: "The next implementation wave wires the app shell into Noesis routes and the private Python sidecar.",
  },
];

export default function ViewerPage() {
  return (
    <section className="biofield-grid">
      {viewerCards.map((card) => (
        <article className="biofield-panel" key={card.label}>
          <p className="biofield-kicker">{card.label}</p>
          <p className="biofield-metric">{card.value}</p>
          <p className="biofield-copy">{card.copy}</p>
        </article>
      ))}
    </section>
  );
}
