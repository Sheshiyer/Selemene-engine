"use client";

import { useEffect, useMemo, useState } from "react";

const styles = {
  panel: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "center",
    gap: "1rem",
    padding: "2rem 1.5rem",
    borderRadius: "var(--r-md)",
    border: "1px solid var(--line-strong)",
    background:
      "linear-gradient(135deg, #070B1D 0%, rgba(45,0,80,0.35) 55%, rgba(11,80,251,0.1) 100%)",
    boxShadow: "var(--inset-glow), var(--shadow-md)",
  },
  svgWrap: {
    width: 164,
    height: 164,
    borderRadius: "50%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    background:
      "radial-gradient(circle, rgba(16,181,167,0.12) 0%, rgba(11,80,251,0.04) 46%, transparent 70%)",
  },
  title: {
    fontFamily: "var(--font-display)",
    fontSize: "1rem",
    letterSpacing: "0.08em",
    color: "var(--signal)",
    textTransform: "uppercase" as const,
  },
  text: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.75rem",
    letterSpacing: "0.06em",
    color: "var(--muted)",
  },
  activeEngine: {
    color: "var(--c-emerald)",
  },
};

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
    <section style={styles.panel} aria-live="polite" aria-busy="true">
      <div style={styles.svgWrap} aria-hidden>
        <svg width="132" height="132" viewBox="0 0 132 132" fill="none">
          <circle
            className="loader-core"
            cx="66"
            cy="66"
            r="7"
            fill="rgba(16,181,167,0.35)"
          />
          <circle
            className="sigil-path"
            cx="66"
            cy="66"
            r="44"
            stroke="rgba(197,160,23,0.82)"
            strokeWidth="1.2"
          />
          <path
            className="sigil-path sigil-path-delay-1"
            d="M66 16 L78 52 L116 52 L85 74 L97 112 L66 89 L35 112 L47 74 L16 52 L54 52 Z"
            stroke="rgba(197,160,23,0.76)"
            strokeWidth="1.1"
          />
          <path
            className="sigil-path sigil-path-delay-2"
            d="M66 28 C80 44 87 57 87 66 C87 81 77 91 66 104 C55 91 45 81 45 66 C45 57 52 44 66 28 Z"
            stroke="rgba(16,181,167,0.7)"
            strokeWidth="1.1"
          />
          <path
            className="sigil-path sigil-path-delay-3"
            d="M32 66 H100 M66 32 V100 M42 42 L90 90 M90 42 L42 90"
            stroke="rgba(11,80,251,0.62)"
            strokeWidth="0.9"
          />
        </svg>
      </div>
      <h2 style={styles.title}>{modeLabel} calculation underway</h2>
      <p style={styles.text}>
        drawing <span style={styles.activeEngine}>{names[index]}</span>…
      </p>
    </section>
  );
}
