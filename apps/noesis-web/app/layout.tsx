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
      <head>
        {/* Tryambakam Noesis brand fonts — Panchang (display) + Satoshi (body) */}
        <link
          rel="stylesheet"
          href="https://api.fontshare.com/v2/css?f[]=panchang@700,800&f[]=satoshi@400,500,600,700&display=swap"
        />
        {/* SF Mono is a system font — no CDN needed */}
      </head>
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
