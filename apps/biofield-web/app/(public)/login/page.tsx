import Link from "next/link";

export default function LoginPage() {
  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <section className="biofield-hero">
          <p className="biofield-eyebrow">Wave 1.1 scaffold</p>
          <h1 className="biofield-title">Biofield Web</h1>
          <p className="biofield-copy">
            This shell is the dedicated user-facing surface for the native biofield viewer.
            Authentication wiring, session startup, and capture analysis land in the next batches.
          </p>
          <div className="biofield-actions">
            <Link className="biofield-button" href="/viewer">
              Open viewer shell
            </Link>
            <Link className="biofield-link" href="/history">
              View history shell
            </Link>
          </div>
        </section>
      </div>
    </main>
  );
}
