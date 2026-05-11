// Palta (पल्टा) — swara permutation pattern. The Carnatic equivalent of an
// arpeggiator. NOT broken chords (raga has no chords); rather standardized
// practice patterns that traverse the 7-swara raga in characteristic shapes.
//
// Sourced from the *Sarali Varisai* through *Alankara* practice traditions
// codified by Purandara Dasa for Carnatic pedagogy.

export type PaltaName =
  /** Sarali Varisai — basic 4-note ascending paired patterns. */
  | 'sarali'
  /** Jantai Varisai — paired-repeat patterns (SS RR GG MM). */
  | 'jantai'
  /** Dattu Varisai — alternating-skip patterns (SG RM GP MD). */
  | 'dattu'
  /** Tara Sthayi Varisai — patterns reaching the upper Sa'. */
  | 'tara'
  /** Mel-Sthayi — patterns climbing into the upper register repeatedly. */
  | 'mel-sthayi'
  /** Adi Tala Alankara — 8-beat decorative figure. */
  | 'alankara';

export interface Palta {
  name: PaltaName;
  display: string;
  /** Short description for UI tooltip. */
  description: string;
  /**
   * Generator that takes the 8-element swara index sequence
   * [Sa, R, G, M, Pa, D, N, Sa'] (indices into the swara array, NOT shruti
   * indices) and returns an expanded sequence of swara indices forming the
   * palta pattern. Length varies per palta.
   */
  generate: (swaras: readonly number[]) => readonly number[];
}
