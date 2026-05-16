"use client";

import { useEffect, useMemo, useState } from "react";

interface CalculationLoaderProps {
  engineLabels: string[];
  modeLabel: string;
}

export default function CalculationLoader({
  engineLabels,
  modeLabel,
}: CalculationLoaderProps) {
  const names = useMemo(
    () => (engineLabels.length > 0 ? engineLabels : ["witness synthesis"]),
    [engineLabels],
  );
  const [index, setIndex] = useState(0);

  useEffect(() => {
    const interval = window.setInterval(() => {
      setIndex((current) => (current + 1) % names.length);
    }, 520);
    return () => window.clearInterval(interval);
  }, [names.length]);

  return (
    <section
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "1.25rem",
        padding: "4rem 2rem",
        borderRadius: "10px",
        border: "1px solid rgba(255, 255, 255, 0.08)",
        background: "#000000",
        position: "relative",
        overflow: "hidden",
        textAlign: "center",
      }}
      aria-live="polite"
      aria-busy="true"
    >
      {/* Atmospheric gradient orb — the Deep Ocean breathing */}
      <div
        aria-hidden
        style={{
          position: "absolute",
          top: "50%", left: "50%",
          width: 280, height: 280,
          borderRadius: "50%",
          background:
            "linear-gradient(135deg, rgb(160, 224, 171) 0%, rgb(255, 172, 46) 50%, rgb(165, 45, 37) 100%)",
          animation: "deepOceanOrb 3.5s ease-in-out infinite",
          pointerEvents: "none",
        }}
      />

      {/* Scanning gradient line — sweeps the field */}
      <div
        aria-hidden
        style={{
          position: "absolute",
          top: 0, left: 0, right: 0,
          height: "1px",
          background:
            "linear-gradient(90deg, transparent 0%, rgb(160, 224, 171) 25%, rgb(255, 172, 46) 50%, rgb(165, 45, 37) 75%, transparent 100%)",
          animation: "deepOceanScan 2.8s ease-in-out infinite",
        }}
      />

      {/* Heading */}
      <h2
        style={{
          position: "relative",
          fontFamily: "system-ui, -apple-system, sans-serif",
          fontSize: "0.6875rem",
          fontWeight: 400,
          letterSpacing: "0.2em",
          color: "#ffffff",
          textTransform: "uppercase",
          margin: 0,
        }}
      >
        {modeLabel} — calculating
      </h2>

      {/* Cycling engine name */}
      <p
        style={{
          position: "relative",
          fontFamily: "system-ui, -apple-system, sans-serif",
          fontSize: "0.875rem",
          lineHeight: 1.5,
          color: "#6d6d6d",
          margin: 0,
        }}
      >
        drawing{" "}
        <span
          style={{ color: "#ffffff", transition: "color 0.15s ease" }}
          key={index}
        >
          {names[index]}
        </span>
        …
      </p>
    </section>
  );
}

