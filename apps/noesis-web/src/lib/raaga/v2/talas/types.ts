// Tala — rhythmic cycle. Frozen contract for v2.
//
// Carnatic talas are described by:
//   - beats        : total aksharas (beats) per cycle
//   - structure    : a sequence of "anga" group sizes summing to `beats`
//   - euclid       : (k, n) Euclidean approximation for Strudel `euclid(k,n)`
//                    where k = number of strong beats and n = total beats
//
// Example — Adi tala (4 + 2 + 2 = 8): structure [4,2,2], k=3 strong beats
// (samam, edam, anti-samam).

export type TalaName =
  | 'adi'           // 4 + 2 + 2 = 8 beats — the most common tala
  | 'rupakam'       // 1 + 2 = 3 beats (sometimes 2 + 4 = 6)
  | 'misra-chapu'   // 3 + 4 = 7 beats
  | 'khanda-chapu'  // 2 + 3 = 5 beats
  | 'tisra-eka'     // 3 beats
  | 'jhampa';       // 7 + 1 + 2 = 10 beats (Misra Jhampa)

export interface Tala {
  name: TalaName;
  /** Display name (Sanskrit IAST). */
  display: string;
  /** Total aksharas per cycle. */
  beats: number;
  /** Anga structure — group sizes summing to `beats`. */
  structure: readonly number[];
  /** Euclidean (k, n) for Strudel `euclid(k, n)` accent pattern. */
  euclid: readonly [number, number];
  /** Beat indices (0-based) where strong accent (samam-class) lands. */
  accentBeats: readonly number[];
  /** Short description for UI tooltips. */
  description: string;
}
