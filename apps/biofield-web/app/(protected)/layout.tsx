import Link from "next/link";

export default function ProtectedLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="biofield-shell">
      <div className="biofield-stack">
        <section className="biofield-panel biofield-shell-nav">
          <h1>Selemene Biofield</h1>
          <nav className="biofield-nav" aria-label="Biofield navigation">
            <Link className="biofield-link" href="/viewer">
              Viewer
            </Link>
            <Link className="biofield-link" href="/history">
              History
            </Link>
            <Link className="biofield-link" href="/login">
              Login shell
            </Link>
          </nav>
        </section>
        {children}
      </div>
    </main>
  );
}
