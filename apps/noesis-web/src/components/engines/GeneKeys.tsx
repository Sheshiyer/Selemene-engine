const SHADOW_COLOR = "#C65D3B";
const GIFT_COLOR = "#10B5A7";
const SIDDHI_COLOR = "#C5A017";

const SPECTRUM_GRADIENT =
  "linear-gradient(90deg, rgba(198,93,59,0.7) 0%, rgba(16,181,167,0.7) 50%, rgba(197,160,23,0.9) 100%)";

const styles = {
  container: { display: "flex", flexDirection: "column" as const, gap: "1rem" },
  sectionTitle: {
    fontSize: "0.75rem",
    color: "var(--gold)",
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
  },
  sphereGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
    gap: "0.75rem",
  },
  sphere: {
    background: "var(--field)",
    borderRadius: "var(--radius)",
    padding: "0.75rem 0.75rem 0.625rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.5rem",
  },
  sphereName: {
    fontSize: "0.85rem",
    fontWeight: 700,
    color: "var(--text)",
  },
  gateNum: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
    marginTop: "-0.25rem",
  },
  spectrumBar: {
    width: "100%",
    height: 8,
    borderRadius: 4,
    background: SPECTRUM_GRADIENT,
    marginTop: "0.125rem",
  },
  spectrumLabels: {
    display: "flex",
    justifyContent: "space-between" as const,
    alignItems: "flex-start" as const,
    marginTop: "0.25rem",
    gap: "0.25rem",
  },
  spectrumLabel: {
    display: "flex",
    flexDirection: "column" as const,
    alignItems: "center" as const,
    gap: "0.125rem",
    minWidth: 0,
  },
  spectrumLabelLeft: {
    alignItems: "flex-start" as const,
  },
  spectrumLabelCenter: {
    alignItems: "center" as const,
  },
  spectrumLabelRight: {
    alignItems: "flex-end" as const,
  },
  labelTitle: {
    fontSize: "0.6rem",
    fontWeight: 700,
    textTransform: "uppercase" as const,
    letterSpacing: "0.04em",
    lineHeight: 1.2,
  },
  labelValue: {
    fontSize: "0.72rem",
    fontWeight: 500,
    lineHeight: 1.3,
  },
  tick: {
    width: 2,
    height: 6,
    borderRadius: 1,
  },
  sub: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
  },
};

function str(v: unknown): string {
  if (v === null || v === undefined) return "—";
  return String(v);
}

function obj(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v))
    return v as Record<string, unknown>;
  return {};
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

const SPHERE_NAMES = ["Life's Work", "Evolution", "Radiance", "Purpose"];

interface SpectrumProps {
  shadow: string;
  gift: string;
  siddhi: string;
}

function Spectrum({ shadow, gift, siddhi }: SpectrumProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column" as const }}>
      {/* Gradient bar */}
      <div style={styles.spectrumBar} />

      {/* Three labeled tick columns */}
      <div style={styles.spectrumLabels}>
        {/* Shadow — left aligned */}
        <div
          style={{
            ...styles.spectrumLabel,
            ...styles.spectrumLabelLeft,
            flex: "1 1 0",
          }}
        >
          <div
            style={{ ...styles.tick, background: SHADOW_COLOR, marginTop: 4 }}
          />
          <span style={{ ...styles.labelTitle, color: SHADOW_COLOR }}>
            Shadow
          </span>
          <span style={{ ...styles.labelValue, color: SHADOW_COLOR }}>
            {shadow}
          </span>
        </div>

        {/* Gift — center aligned */}
        <div
          style={{
            ...styles.spectrumLabel,
            ...styles.spectrumLabelCenter,
            flex: "1 1 0",
          }}
        >
          <div
            style={{ ...styles.tick, background: GIFT_COLOR, marginTop: 4 }}
          />
          <span style={{ ...styles.labelTitle, color: GIFT_COLOR }}>Gift</span>
          <span style={{ ...styles.labelValue, color: GIFT_COLOR }}>{gift}</span>
        </div>

        {/* Siddhi — right aligned */}
        <div
          style={{
            ...styles.spectrumLabel,
            ...styles.spectrumLabelRight,
            flex: "1 1 0",
          }}
        >
          <div
            style={{ ...styles.tick, background: SIDDHI_COLOR, marginTop: 4 }}
          />
          <span style={{ ...styles.labelTitle, color: SIDDHI_COLOR }}>
            Siddhi
          </span>
          <span style={{ ...styles.labelValue, color: SIDDHI_COLOR }}>
            {siddhi}
          </span>
        </div>
      </div>
    </div>
  );
}

interface SphereCardProps {
  name: string;
  gate: string | null;
  shadow: string;
  gift: string;
  siddhi: string;
}

function SphereCard({ name, gate, shadow, gift, siddhi }: SphereCardProps) {
  return (
    <div style={styles.sphere}>
      <span style={styles.sphereName}>{name}</span>
      {gate !== null && <span style={styles.gateNum}>Gate {gate}</span>}
      <Spectrum shadow={shadow} gift={gift} siddhi={siddhi} />
    </div>
  );
}

interface GeneKeysProps {
  result: Record<string, unknown>;
}

export default function GeneKeys({ result }: GeneKeysProps) {
  const activation = obj(result.activation_sequence ?? result.activation);
  const spheres = arr(activation.spheres ?? result.spheres);
  const partner = result.programming_partner ?? result.partner;

  return (
    <div style={styles.container}>
      <span style={styles.sectionTitle}>Activation Sequence</span>
      <div style={styles.sphereGrid}>
        {spheres.length > 0
          ? spheres.map((s, i) => {
              const sphere = obj(s);
              return (
                <SphereCard
                  key={i}
                  name={str(
                    sphere.name ?? SPHERE_NAMES[i] ?? `Sphere ${i + 1}`
                  )}
                  gate={sphere.gate != null ? str(sphere.gate) : null}
                  shadow={str(sphere.shadow)}
                  gift={str(sphere.gift)}
                  siddhi={str(sphere.siddhi)}
                />
              );
            })
          : SPHERE_NAMES.map((name, i) => {
              const key = name.toLowerCase().replace(/['\s]/g, "_");
              const sphere = obj(activation[key] ?? result[key]);
              return (
                <SphereCard
                  key={i}
                  name={name}
                  gate={sphere.gate != null ? str(sphere.gate) : null}
                  shadow={str(sphere.shadow)}
                  gift={str(sphere.gift)}
                  siddhi={str(sphere.siddhi)}
                />
              );
            })}
      </div>

      {partner != null && (
        <div>
          <span style={styles.sectionTitle}>Programming Partner</span>
          <p style={styles.sub}>{str(partner)}</p>
        </div>
      )}
    </div>
  );
}
