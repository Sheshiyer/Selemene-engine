// ─── MicroYantra — dispatcher for 8 inline glyph kinds ─────────────────
// Reads { kind, arg } and renders the matching tiny SVG sigil. Used both
// directly as a React element and as a hydration target for placeholder
// spans injected by microYantraEnhance.ts during HTML rendering.
//
// Per integrated-reading-design-v2.md § 5.5 — inline micro-yantras add
// "density of meaning" by visually anchoring every technical term in the
// reading prose.

import { PlanetGlyph, type PlanetName } from "./icons/PlanetGlyph";
import { HouseGlyph } from "./icons/HouseGlyph";
import { NakshatraGlyph, type NakshatraName } from "./icons/NakshatraGlyph";
import { YogaGlyph, type YogaName } from "./icons/YogaGlyph";
import { DoshaGlyph, type DoshaName } from "./icons/DoshaGlyph";
import { DashaTransitionGlyph } from "./icons/DashaTransitionGlyph";
import { GeneKeyGlyph } from "./icons/GeneKeyGlyph";
import { TarotMinorGlyph, type TarotSuit } from "./icons/TarotMinorGlyph";

export type MicroYantraKind =
  | "planet"
  | "house"
  | "nakshatra"
  | "yoga"
  | "dosha"
  | "dasha-transition"
  | "gene-key"
  | "tarot-minor";

export interface MicroYantraProps {
  kind: MicroYantraKind;
  /** Kind-specific argument. See dispatch below for shape per kind. */
  arg: string;
  size?: number;
  title?: string;
}

const PLANET_NAMES: PlanetName[] = [
  "sun",
  "moon",
  "mars",
  "mercury",
  "jupiter",
  "venus",
  "saturn",
  "rahu",
  "ketu",
];

const NAKSHATRA_NAMES: NakshatraName[] = [
  "ashwini",
  "bharani",
  "krittika",
  "rohini",
  "mrigashira",
  "ardra",
  "punarvasu",
  "pushya",
  "ashlesha",
  "magha",
  "purvaphalguni",
  "uttaraphalguni",
  "hasta",
  "chitra",
  "swati",
  "vishakha",
  "anuradha",
  "jyeshtha",
  "mool",
  "purvashadha",
  "uttarashadha",
  "shravana",
  "dhanishta",
  "shatabhisha",
  "purvabhadrapada",
  "uttarabhadrapada",
  "revati",
];

const YOGA_NAMES: YogaName[] = [
  "raj",
  "gajakesari",
  "saraswati",
  "dhana",
  "vipreet-raj",
];

const DOSHA_NAMES: DoshaName[] = [
  "sade-sati",
  "kala-sarpa",
  "mangal-dosha",
  "pitru-dosha",
  "kemadruma",
];

const TAROT_SUITS: TarotSuit[] = ["cups", "wands", "swords", "pentacles"];

function isPlanet(s: string): s is PlanetName {
  return (PLANET_NAMES as readonly string[]).includes(s);
}
function isNakshatra(s: string): s is NakshatraName {
  return (NAKSHATRA_NAMES as readonly string[]).includes(s);
}
function isYoga(s: string): s is YogaName {
  return (YOGA_NAMES as readonly string[]).includes(s);
}
function isDosha(s: string): s is DoshaName {
  return (DOSHA_NAMES as readonly string[]).includes(s);
}
function isTarotSuit(s: string): s is TarotSuit {
  return (TAROT_SUITS as readonly string[]).includes(s);
}

export function MicroYantra({ kind, arg, size, title }: MicroYantraProps) {
  switch (kind) {
    case "planet": {
      const a = arg.toLowerCase();
      if (!isPlanet(a)) return null;
      return <PlanetGlyph planet={a} size={size} title={title} />;
    }
    case "house": {
      const n = parseInt(arg, 10);
      if (!Number.isFinite(n) || n < 1 || n > 12) return null;
      return <HouseGlyph house={n} size={size} title={title} />;
    }
    case "nakshatra": {
      const a = arg.toLowerCase();
      if (!isNakshatra(a)) return null;
      return <NakshatraGlyph nakshatra={a} size={size} title={title} />;
    }
    case "yoga": {
      const a = arg.toLowerCase();
      if (!isYoga(a)) return null;
      return <YogaGlyph yoga={a} size={size} title={title} />;
    }
    case "dosha": {
      const a = arg.toLowerCase();
      if (!isDosha(a)) return null;
      return <DoshaGlyph dosha={a} size={size} title={title} />;
    }
    case "dasha-transition": {
      // Format: "from->to" e.g. "rahu->jupiter"
      const [fromRaw, toRaw] = arg.toLowerCase().split("->");
      if (!fromRaw || !toRaw) return null;
      if (!isPlanet(fromRaw) || !isPlanet(toRaw)) return null;
      return (
        <DashaTransitionGlyph from={fromRaw} to={toRaw} size={size} title={title} />
      );
    }
    case "gene-key": {
      const n = parseInt(arg, 10);
      if (!Number.isFinite(n) || n < 1 || n > 64) {
        return <GeneKeyGlyph size={size} title={title} />;
      }
      return <GeneKeyGlyph key64={n} size={size} title={title} />;
    }
    case "tarot-minor": {
      // Format: "suit" or "suit:rank" e.g. "cups:3"
      const [suitRaw, rankRaw] = arg.toLowerCase().split(":");
      if (!isTarotSuit(suitRaw)) return null;
      return (
        <TarotMinorGlyph
          suit={suitRaw}
          rank={rankRaw}
          size={size}
          title={title}
        />
      );
    }
    default:
      return null;
  }
}
