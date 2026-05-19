/**
 * Dyad components — canonical guided-flow chrome for Noesis.
 *
 * See SYSTEM.md → DYAD-CHAMBER PATTERN section for design rationale.
 * See app/get-reading/page.tsx for the canonical 5-step implementation.
 */

export {
  DyadChamber,
  WitnessFigure,
  SPEAKER_COLOR,
  SPEAKER_LABEL,
  type Speaker,
} from "./DyadChamber";

export {
  StepIndicator,
  SigilToken,
  type DyadStep,
  type SigilSymbol,
} from "./StepIndicator";
