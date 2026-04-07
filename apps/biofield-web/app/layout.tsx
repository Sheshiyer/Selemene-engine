import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Selemene Biofield",
  description: "Native biofield viewer for session-based capture, reading history, and future baseline work.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
