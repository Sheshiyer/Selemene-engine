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
        {/* Tryambakam Noesis brand fonts — VARIABLE axes.
            `@1` on Fontshare's API = variable-weight version of the font.
            Lets us drive `font-variation-settings: 'wght' N` continuously
            (motion can interpolate weight as well as opacity now). */}
        <link
          rel="stylesheet"
          href="https://api.fontshare.com/v2/css?f[]=panchang@1&f[]=satoshi@1&display=swap"
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
