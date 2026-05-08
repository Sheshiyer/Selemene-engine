"use client";

import { useState } from "react";
import type { BirthData } from "@/lib/api";

const styles = {
  form: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
    gap: "0.75rem",
    padding: "1.25rem",
    background: "var(--bg-panel)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius)",
  },
  label: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    fontWeight: 500,
    textTransform: "uppercase" as const,
    letterSpacing: "0.05em",
  },
  submit: {
    gridColumn: "1 / -1",
    padding: "0.625rem 1.5rem",
    background: "var(--gold)",
    color: "#070B1D",
    fontWeight: 700,
    fontSize: "0.875rem",
    borderRadius: "var(--radius)",
    border: "none",
    cursor: "pointer",
    transition: "opacity 0.15s",
    fontFamily: "'Space Grotesk', sans-serif",
  },
  submitDisabled: {
    opacity: 0.5,
    cursor: "not-allowed",
  },
};

interface BirthDataFormProps {
  onSubmit: (data: BirthData) => void;
  loading: boolean;
}

export default function BirthDataForm({ onSubmit, loading }: BirthDataFormProps) {
  const [date, setDate] = useState("");
  const [time, setTime] = useState("");
  const [timezone, setTimezone] = useState(
    Intl.DateTimeFormat().resolvedOptions().timeZone,
  );
  const [lat, setLat] = useState("");
  const [lng, setLng] = useState("");
  const [name, setName] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!date) return;
    onSubmit({
      date,
      time: time || undefined,
      timezone: timezone || undefined,
      latitude: lat ? parseFloat(lat) : undefined,
      longitude: lng ? parseFloat(lng) : undefined,
      name: name || undefined,
    });
  };

  return (
    <form style={styles.form} onSubmit={handleSubmit}>
      <label style={styles.label}>
        Birth Date *
        <input
          type="date"
          value={date}
          onChange={(e) => setDate(e.target.value)}
          required
        />
      </label>
      <label style={styles.label}>
        Birth Time
        <input
          type="time"
          value={time}
          onChange={(e) => setTime(e.target.value)}
          step="1"
        />
      </label>
      <label style={styles.label}>
        Timezone
        <input
          type="text"
          value={timezone}
          onChange={(e) => setTimezone(e.target.value)}
          placeholder="Asia/Kolkata"
        />
      </label>
      <label style={styles.label}>
        Latitude
        <input
          type="number"
          value={lat}
          onChange={(e) => setLat(e.target.value)}
          placeholder="12.9716"
          step="any"
        />
      </label>
      <label style={styles.label}>
        Longitude
        <input
          type="number"
          value={lng}
          onChange={(e) => setLng(e.target.value)}
          placeholder="77.5946"
          step="any"
        />
      </label>
      <label style={styles.label}>
        Name (optional)
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Your name"
        />
      </label>
      <button
        type="submit"
        disabled={loading || !date}
        style={{
          ...styles.submit,
          ...((loading || !date) ? styles.submitDisabled : {}),
        }}
      >
        {loading ? "Calculating…" : "Run Full-Spectrum Analysis"}
      </button>
    </form>
  );
}
