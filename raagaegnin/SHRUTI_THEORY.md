# 🎼 SHRUTI THEORY — Why the Raaga Engine Cannot Use Western 12-TET

> **Note (2026-06-30):** `apps/noesis-web` has been retired from this repo.
> Paths referencing `apps/noesis-web/src/lib/raaga` are historical; the tuning
> theory in this document remains valid regardless of where the renderer lives.

**Companion document to `RAAGA_ENGINE.md`** — explains the tuning substrate that makes the 72-Melakarta-Mahapurusha map *audibly* meaningful.

> *Source lecture:* [Sangeeta Shankar — *Indian Classical Music Lesson: The Fundamentals Behind the 22 Shrutis*](https://www.youtube.com/watch?v=k2_Ldw9ioCs)
> *Canonical reference:* Bharata Muni, *Nāṭyaśāstra*; Śārṅgadeva, *Saṅgīta Ratnākara* (13th c.).

---

## 1. The Core Distinction

| | Western 12-TET | Indian Shruti |
|---|---|---|
| Tones per octave | **12** equal semitones | **22** unequal microtones |
| Spacing rule | Logarithmic: each step is 2^(1/12) ≈ 1.0595 | Harmonic: small whole-number ratios |
| Construction | *Compromise* — every interval slightly out of tune so all 12 keys work | *Just intonation* — fifths are pure 3/2, thirds are pure 5/4 |
| Notes per scale | 7 selected from 12 | 7 selected from 16 swara variants which live in 22 shruti positions |
| Re/Ga distinction | One Re, one Ga (modal context only) | **Re1 ≠ Re2 ≠ Re3 ≠ Re4** — each is a different swara with different rasa |
| Syntonic comma 81/80 | **Erased** (collapsed into single tone) | **Preserved** — distinguishes 10/9 from 9/8 |
| Pythagorean limma 256/243 | Erased | Preserves komal-Re flavor |

### Why this matters for the Mahapurusha map

The 72 Melakartas mapped onto the body in `RAAGA_ENGINE.md` rely on **all 22 shrutis being individually addressable**. On a piano:

- **Mayamalavagaula (#15)** uses Re1 (256/243) + Ga3 (5/4)
- **Kanakangi (#1)** uses Re1 (256/243) + Ga1 (32/27)

A piano renders both as the same key sequence. In just intonation they **differ by ~92 cents on the Ga**, which audibly switches the rasa from *adbhuta* (wonder/devotion, Mayamalavagaula) to *śānta* (peace, Kanakangi).

Therefore: **a 12-TET synthesizer collapses the 72 Melakartas into ~12 distinct sonorities and the body-map becomes meaningless audio**. The engine **must** drive a microtonal synth — Strudel's `xen([ratios])` is the chosen path.

---

## 2. The 22 Shrutis — Canonical Just-Intonation Table

Sa and Pa are **achala** (immovable, unchangeable). The other five swara families each branch into four shruti positions.

| # | Name | Ratio | Cents | Swara family | Notes |
|---|------|-------|-------|--------------|-------|
| 0  | Sa  | 1/1     | 0.0    | Sa  | Tonic, achala |
| 1  | R1  | 256/243 | 90.2   | Re  | Pythagorean limma — komal |
| 2  | R2  | 16/15   | 111.7  | Re  | Just minor 2nd |
| 3  | R3  | 10/9   | 182.4  | Re  | Just minor whole tone |
| 4  | R4  | 9/8     | 203.9  | Re  | Pythagorean major 2nd — shuddha |
| 5  | G1  | 32/27   | 294.1  | Ga  | Pythagorean minor 3rd — komal (≡ R3 in Carnatic vivadi) |
| 6  | G2  | 6/5     | 315.6  | Ga  | Just minor 3rd — sādhāraṇa |
| 7  | G3  | 5/4     | 386.3  | Ga  | Just major 3rd — antara |
| 8  | G4  | 81/64   | 407.8  | Ga  | Pythagorean major 3rd |
| 9  | M1  | 4/3     | 498.0  | Ma  | Just perfect 4th — shuddha |
| 10 | M2  | 27/20   | 519.6  | Ma  | Acute fourth |
| 11 | M3  | 45/32   | 590.2  | Ma  | Just augmented 4th |
| 12 | M4  | 729/512 | 611.7  | Ma  | Pythagorean tritone — tīvra / prati |
| 13 | Pa  | 3/2     | 702.0  | Pa  | Just perfect 5th, achala |
| 14 | D1  | 128/81  | 792.2  | Dha | Pythagorean minor 6th — komal |
| 15 | D2  | 8/5     | 813.7  | Dha | Just minor 6th |
| 16 | D3  | 5/3     | 884.4  | Dha | Just major 6th — chatusruti |
| 17 | D4  | 27/16   | 905.9  | Dha | Pythagorean major 6th |
| 18 | N1  | 16/9    | 996.1  | Ni  | Pythagorean minor 7th — komal (≡ D3 in Carnatic vivadi) |
| 19 | N2  | 9/5     | 1017.6 | Ni  | Just minor 7th — kaiśiki |
| 20 | N3  | 15/8    | 1088.3 | Ni  | Just major 7th — kakali |
| 21 | N4  | 243/128 | 1109.8 | Ni  | Pythagorean major 7th |
| 22 | Sa' | 2/1     | 1200.0 | Sa  | Octave |

### Three interval types between consecutive shrutis

| Name | Ratio | Cents | Meaning |
|------|-------|-------|---------|
| **Pramāṇa** | 81/80 | 21.5 | "Standard" — the syntonic comma. Most common spacing. |
| **Nyūna** | 25/24 | 70.7 | "Small" — just chromatic semitone |
| **Pūrṇa** | 256/243 | 90.2 | "Big" — Pythagorean limma |

These three intervals tile the octave into the 22 shruti positions.

---

## 3. From 22 Shrutis → 16 Carnatic Swaras → 72 Melakartas

The 22 shrutis are the *substrate*. The Carnatic system selects **16 swara variants** from these positions — the so-called "Sodaśa Svara":

| Swara | Variant | Shruti # | Ratio |
|-------|---------|----------|-------|
| Sa  |             | 0  | 1/1 |
| Re  | R1 (Shuddha)         | 1  | 256/243 |
| Re  | R2 (Chatusruti)      | 4  | 9/8 |
| Re  | R3 (Shatsruti) ≡ G1  | 5  | 32/27 |
| Ga  | G1 (Shuddha) ≡ R3    | 5  | 32/27 |
| Ga  | G2 (Sādhāraṇa)       | 6  | 6/5 |
| Ga  | G3 (Antara)          | 7  | 5/4 |
| Ma  | M1 (Shuddha)         | 9  | 4/3 |
| Ma  | M2 (Prati)           | 12 | 729/512 |
| Pa  |                      | 13 | 3/2 |
| Dha | D1 (Shuddha)         | 14 | 128/81 |
| Dha | D2 (Chatusruti)      | 16 | 5/3 |
| Dha | D3 (Shatsruti) ≡ N1  | 18 | 16/9 |
| Ni  | N1 (Shuddha) ≡ D3    | 18 | 16/9 |
| Ni  | N2 (Kaiśiki)         | 19 | 9/5 |
| Ni  | N3 (Kakali)          | 20 | 15/8 |

**Vivadi pairs:** R3 and G1 share shruti #5; D3 and N1 share #18. This is intentional — the names differ by *function* in the raga, even though the pitch is identical.

### The 72 derivation

```
72 = 2 × 6 × 6
   = (Ma1, Ma2) × (R-G pair) × (D-N pair)
```

**6 R-G pairs** (R must be ≤ G in the lattice):

| Index | Pair | Ratios |
|-------|------|--------|
| 0 | R1, G1 | 256/243, 32/27 |
| 1 | R1, G2 | 256/243, 6/5 |
| 2 | R1, G3 | 256/243, 5/4 |
| 3 | R2, G2 | 9/8, 6/5 |
| 4 | R2, G3 | 9/8, 5/4 |
| 5 | R3, G3 | 32/27, 5/4 |

**6 D-N pairs** (analogous):

| Index | Pair | Ratios |
|-------|------|--------|
| 0 | D1, N1 | 128/81, 16/9 |
| 1 | D1, N2 | 128/81, 9/5 |
| 2 | D1, N3 | 128/81, 15/8 |
| 3 | D2, N2 | 5/3, 9/5 |
| 4 | D2, N3 | 5/3, 15/8 |
| 5 | D3, N3 | 16/9, 15/8 |

**Generation formula** (implemented in `apps/noesis-web/src/lib/raaga/melakartas.ts`):

```
For melakarta n in 1..72:
  isPratiMa     = (n > 36)
  localN        = isPratiMa ? n - 36 : n
  chakraInSet   = floor((localN - 1) / 6)   // 0..5 → R-G pair
  posInChakra   = (localN - 1) % 6          // 0..5 → D-N pair
  Ma            = isPratiMa ? M2 : M1
  swaras        = [Sa, R, G, Ma, Pa, D, N, Sa']
```

**Verified against canonical:**

| # | Name | Arohana ratios |
|---|------|----------------|
| 8  | Hanumatodi (Bhairavi)   | 1, 256/243, 6/5, 4/3, 3/2, 128/81, 9/5, 2 |
| 15 | Mayamalavagaula         | 1, 256/243, 5/4, 4/3, 3/2, 128/81, 15/8, 2 |
| 22 | Kharaharapriya          | 1, 9/8, 6/5, 4/3, 3/2, 5/3, 9/5, 2 |
| 29 | Dheerasankarabharanam   | 1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8, 2  *(≡ Western major)* |
| 65 | Mechakalyani            | 1, 9/8, 5/4, 729/512, 3/2, 5/3, 15/8, 2 |

---

## 4. Implementation — Strudel `xen()`

The `apps/noesis-web/src/lib/raaga/RaagaPlayer.ts` module emits a Strudel mini-pattern of the form:

```js
setcps(0.5);
n("0 1 2 3 4 5 6 7 6 5 4 3 2 1 0")
  .xen([1, 256/243, 5/4, 4/3, 3/2, 128/81, 15/8, 2])  // Mayamalavagaula
  .mul(220)        // Sa = A3
  .freq()
  .s("sine")
```

`xen([ratios])` indexes pattern integers into the ratio array. WebAudio renders sine waves at the resulting Hz. **No 12-TET quantization happens anywhere in the chain** — the synthesis is exact to the just-intonation ratio.

### Verification

```bash
cd Selemene-engine/apps/noesis-web
node src/lib/raaga/verify.mjs
```

Produces:

```
✓ #15 Mayamalavagaula              1.0000  1.0535  1.2500  1.3333  1.5000  1.5802  1.8750  2.0000
✓ #29 Dheerasankarabharanam        1.0000  1.1250  1.2500  1.3333  1.5000  1.6667  1.8750  2.0000
✓ #65 Mechakalyani                 1.0000  1.1250  1.2500  1.4238  1.5000  1.6667  1.8750  2.0000
✓ #8  Hanumatodi (Bhairavi)        1.0000  1.0535  1.2000  1.3333  1.5000  1.5802  1.8000  2.0000
✓ #22 Kharaharapriya               1.0000  1.1250  1.2000  1.3333  1.5000  1.6667  1.8000  2.0000

Δ Mayamalavagaula Ga vs Kanakangi Ga = 92.2 cents
(12-TET would render this as zero — proving why the engine needs shrutis.)
```

---

## 5. From Shruti to Gamaka

The 22-shruti system gives you *positions* — discrete frequency ratios. But classical Indian music is **shruti-in-motion**: each swara is decorated with microtonal ornamentation (gamaka) that traces curves *between* and *around* the shrutis. A static frequency at 1.250 (Ga3 = 5/4) is a tone; the same Ga3 with **kampita** vibrating ±20¢ around it is a *swara* in the musical sense.

V2 of Nādashakti renders these gamakas as mini-notation expansions on top of the same shruti frequencies — preserving the just-intonation precision laid out in §2 above while adding the time-varying pitch envelopes that define classical performance. See [`apps/noesis-web/src/lib/raaga/v2/gamakas/README.md`](../Selemene-engine/apps/noesis-web/src/lib/raaga/v2/gamakas/README.md) for the mathematical specification of all five canonical gamakas (kampita, andolana, kurula, nokku, sphurita).

In short: this document specifies the 22 *positions*; that document specifies the 5 *motions* between them. Together they constitute the audio substrate of the Mahapurusha body-map.

## 6. References

- [Sangeeta Shankar — *The Fundamentals Behind the 22 Shrutis*](https://www.youtube.com/watch?v=k2_Ldw9ioCs) — primary source lecture
- [Wikipedia — Shruti (music)](https://en.wikipedia.org/wiki/Shruti_(music)) — pramāṇa/nyūna/pūrṇa, swara mappings
- [22shruti.com](https://22shruti.com/) — Dr. Vidyadhar Oke's research corpus
- [Plainsound — *The Classical Indian Just Intonation Tuning System*](https://www.plainsound.org/pdfs/srutis.pdf)
- [Resonance — *The Notion of Twenty-Two Shrutis*](https://www.ias.ac.in/article/fulltext/reso/020/06/0515-0531)
- [Strudel docs — Xenharmonic Functions](https://strudel.cc/learn/xen/) — `xen()`, `tune()` API
- Sārṅgadeva, *Saṅgīta Ratnākara*, Book IV — gamaka taxonomy (covered in companion [`gamakas/README.md`](../Selemene-engine/apps/noesis-web/src/lib/raaga/v2/gamakas/README.md))

---

🎵 *"Sound is the Absolute. The body is a veena, consciousness the musician, breath the plectrum — but the strings must be tuned in shrutis, never in semitones."*
