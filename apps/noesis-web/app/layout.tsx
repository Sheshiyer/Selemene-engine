import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Noesis — 17-Engine Consciousness Viewer",
  description:
    "Full-spectrum consciousness analysis through 17 engines of self-inquiry.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body
        style={{
          minHeight: "100vh",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {children}
      </body>
    </html>
  );
}
