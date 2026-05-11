"use client";

import { useState, useEffect } from "react";
import type { BirthData } from "@/lib/api";

const styles = {
  wrapper: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "1rem",
  },
  sectionHeading: {
    fontFamily: "var(--font-display)",
    fontSize: "1.25rem",
    fontWeight: 700,
    color: "var(--signal)",
    letterSpacing: "0.04em",
  },
  form: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
    gap: "0.875rem",
    padding: "1.375rem",
    background: "var(--surface-1)",
    border: "1px solid var(--line-mid)",
    borderRadius: "var(--r-md)",
  },
  label: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.5rem",
  },
  labelText: {
    fontSize: "0.72rem",
    color: "var(--muted)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
    fontFamily: "var(--font-body)",
    lineHeight: 1.2,
  },
  labelSanskrit: {
    display: "block",
    fontSize: "0.62rem",
    color: "var(--muted)",
    fontFamily: "var(--font-mono)",
    fontStyle: "italic",
    opacity: 0.65,
    marginTop: "-0.25rem",
    letterSpacing: "0.03em",
    fontWeight: 400,
  },
  geoRow: {
    display: "flex",
    gap: "0.5rem",
    alignItems: "flex-end",
  },
  geoBtn: {
    flexShrink: 0,
    padding: "0.4rem 0.625rem",
    background: "rgba(16,181,167,0.1)",
    border: "1px solid rgba(16,181,167,0.3)",
    borderRadius: "var(--r-sm)",
    color: "var(--c-emerald)",
    fontSize: "0.75rem",
    cursor: "pointer",
    fontFamily: "var(--font-mono)",
    whiteSpace: "nowrap" as const,
    transition: "background 0.15s, border-color 0.15s",
    lineHeight: 1,
  },
  rememberedHint: {
    gridColumn: "1 / -1",
    fontSize: "0.72rem",
    color: "var(--c-emerald)",
    fontFamily: "var(--font-mono)",
    letterSpacing: "0.04em",
    opacity: 0.8,
  },
  submit: {
    gridColumn: "1 / -1",
    padding: "0.75rem 1.5rem",
    // Ba Arc gradient
    background: "linear-gradient(90deg, var(--c-emerald) 0%, var(--signal) 100%)",
    color: "#070B1D",
    fontWeight: 700,
    fontSize: "0.95rem",
    borderRadius: "var(--r-sm)",
    border: "none",
    cursor: "pointer",
    transition: "opacity 0.15s",
    fontFamily: "var(--font-display)",
    letterSpacing: "0.06em",
    textTransform: "uppercase" as const,
  },
  submitDisabled: {
    opacity: 0.45,
    cursor: "not-allowed",
  },
};

interface BirthDataFormProps {
  onSubmit: (data: BirthData) => void;
  loading: boolean;
  initialData?: Partial<BirthData>;
}

export default function BirthDataForm({ onSubmit, loading, initialData }: BirthDataFormProps) {
  const [date, setDate] = useState(initialData?.date ?? "");
  const [time, setTime] = useState(initialData?.time ?? "");
  const [timezone, setTimezone] = useState(
    initialData?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone,
  );
  const [lat, setLat] = useState(initialData?.latitude != null ? String(initialData.latitude) : "");
  const [lng, setLng] = useState(initialData?.longitude != null ? String(initialData.longitude) : "");
  const [name, setName] = useState(initialData?.name ?? "");
  const [geoLoading, setGeoLoading] = useState(false);
  // True when initialData arrived from DB (all core fields populated)
  const isRemembered = Boolean(initialData?.date && initialData?.latitude != null);

  // Sync if initialData arrives after mount (async DB fetch)
  useEffect(() => {
    if (!initialData) return;
    if (initialData.date) setDate(initialData.date);
    if (initialData.time) setTime(initialData.time);
    if (initialData.timezone) setTimezone(initialData.timezone);
    if (initialData.latitude != null) setLat(String(initialData.latitude));
    if (initialData.longitude != null) setLng(String(initialData.longitude));
    if (initialData.name) setName(initialData.name);
  }, [initialData]);

  const handleGeolocate = () => {
    if (!navigator.geolocation) return;
    setGeoLoading(true);
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        setLat(pos.coords.latitude.toFixed(4));
        setLng(pos.coords.longitude.toFixed(4));
        setGeoLoading(false);
      },
      () => setGeoLoading(false),
    );
  };

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
    <div style={styles.wrapper}>
      <h2 style={styles.sectionHeading}>Janma Rekha</h2>
      <form style={styles.form} onSubmit={handleSubmit}>
        {isRemembered && (
          <span style={styles.rememberedHint}>◈ Remembered from last reading</span>
        )}

        <label style={styles.label}>
          <span style={styles.labelText}>
            Birth Date *
            <span style={styles.labelSanskrit}>janma tithi</span>
          </span>
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
            required
          />
        </label>

        <label style={styles.label}>
          <span style={styles.labelText}>
            Birth Time
            <span style={styles.labelSanskrit}>janma kala</span>
          </span>
          <input
            type="time"
            value={time}
            onChange={(e) => setTime(e.target.value)}
            step="1"
          />
        </label>

        <label style={styles.label}>
          <span style={styles.labelText}>
            Timezone
            <span style={styles.labelSanskrit}>kala-khanda</span>
          </span>
          <input
            type="text"
            value={timezone}
            onChange={(e) => setTimezone(e.target.value)}
            placeholder="Asia/Kolkata"
          />
        </label>

        <label style={styles.label}>
          <span style={styles.labelText}>
            Location
            <span style={styles.labelSanskrit}>janma-sthana</span>
          </span>
          <div style={styles.geoRow}>
            <input
              type="number"
              value={lat}
              onChange={(e) => setLat(e.target.value)}
              placeholder="Lat"
              step="any"
            />
            <input
              type="number"
              value={lng}
              onChange={(e) => setLng(e.target.value)}
              placeholder="Lng"
              step="any"
            />
            <button type="button" style={styles.geoBtn} onClick={handleGeolocate}>
              {geoLoading ? "…" : "⊕"}
            </button>
          </div>
        </label>

        <label style={styles.label}>
          <span style={styles.labelText}>
            Name
            <span style={styles.labelSanskrit}>nama (optional)</span>
          </span>
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
    </div>
  );
}
