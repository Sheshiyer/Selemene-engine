import type { Metadata } from "next";
import { Geist_Mono } from "next/font/google";
import "./globals.css";

// Brand fonts: Panchang (display) + Satoshi (body) via FontShare CDN
// SF Mono / Geist Mono (monospace) — Geist_Mono loaded as CSS var fallback
const FONTSHARE_HREF =
  "https://api.fontshare.com/v2/css?f[]=panchang@400,500,600,700,800&f[]=satoshi@300,400,500,700&display=swap";

const geistMono = Geist_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
  display: "swap",
});

export { reportWebVitals } from "@/lib/telemetry-vitals";

export const metadata: Metadata = {
  title: "Selemene Admin",
  description: "Admin portal for key management, users, and platform operations"
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={geistMono.variable}>
      <head>
        <link rel="preconnect" href="https://api.fontshare.com" />
        <link rel="stylesheet" href={FONTSHARE_HREF} />
      </head>
      <body>
        {children}
      </body>
    </html>
  );
}
