# Gamakas — Microtonal Pitch Ornamentation

> *"Without gamaka, there is no rāga. Without rāga, there is only sound."* — Tyagaraja

A swara without gamaka is a frequency. A swara *with* gamaka is the swara's *life*. This module implements the five canonical gamakas from Sārṅgadeva's *Saṅgīta Ratnākara* (Book IV) as pitch envelopes that decorate each swara's central frequency.

## Mathematical Specification

For a swara with central frequency `f₀ Hz` and hold duration `T seconds`, each gamaka generates a time-varying frequency `f(t)` for `t ∈ [0, T]`.

### 1. Kampita (कम्पित) — light vibrato

```
f(t) = f₀ · 2^(δ · sin(2πr·t) / 1200)
```

where `δ` ∈ [10, 30] cents (vibrato width) and `r` ∈ [4, 8] Hz (rate).

**Use:** sustained swaras in slow ālāpana. Heart-rate-matched (~6 Hz) ↔ kampita is *physiological*.

### 2. Andolana (आन्दोलन) — slow oscillation between swara and neighbor

```
f(t) = f₀ · 2^(δ · sin(2πr·t) / 1200)
```

with `δ` ∈ [30, 80] cents and `r` ∈ [0.8, 2.5] Hz.

**Use:** tonic-functioning notes in raga where the neighboring shruti is *implied* but never landed on. The wider amplitude evokes the neighbor without naming it.

### 3. Kurula (कुरुल) — curved slide between adjacent shrutis

```
f(t) = f₀ · 2^( c(t/T_g) / 1200 )    for t ∈ [0, T_g]
f(t) = f₁                            for t ∈ [T_g, T]
```

where `c(x)` is a smooth ease-in-out curve from `0` to `target_cents`, `T_g = glideFraction · T`, and `f₁ = f₀ · 2^(target_cents/1200)`.

The default curve is `c(x) = target · (3x² − 2x³)` (cubic Hermite) — smoother than linear glide.

**Use:** transitions between Re-Ga or Dha-Ni in slow paces — the slide is *itself* the music.

### 4. Nokku (नोक्कु) — grace note above target

```
f(t) = f_grace = f₀ · 2^(graceCents/1200)    for t ∈ [0, T_g]
f(t) = f₀                                    for t ∈ [T_g, T]
```

where `T_g = graceFraction · T`. Default `graceCents = +100` (one shruti up), `graceFraction = 0.08`.

**Use:** rapid grace note "kissed" before settling — the *imagined* shruti above. Common on Ma in Carnatic Ma1-rich ragas.

### 5. Sphurita (स्फुरित) — bent attack rising into the swara

```
f(t) = f_start + (f₀ − f_start) · ease(t / T_a)    for t ∈ [0, T_a]
f(t) = f₀                                          for t ∈ [T_a, T]
```

with `f_start = f₀ · 2^(startCents/1200)`, `T_a = attackFraction · T`. Default `startCents = -60`, `attackFraction = 0.1`. Ease function is exponential (faster start, slower landing).

**Use:** the bent attack of a sitar mizrab strike. Approaches from below — never from above.

## Strudel Mapping (Phase-2 implementation note)

Each gamaka compiles to a Strudel `freq().penv()` chain:

```js
// kampita on Sa @ 220Hz
freq("220").vib(6).vibmod(20)    // Strudel: vib = rate, vibmod = depth in cents

// nokku on Ma @ 293.33Hz, +100¢ grace for 8% of hold
note(73)
  .penv("<100 0>")               // pitch envelope: +100¢ then 0¢
  .pattack(0.08).pdecay(0.02).pcurve(0)
```

These are placeholders — Phase 2 Swarm A (T-019..T-024) finalizes the exact mini-notation strings.

## Why It Matters

The Mahapurusha body-map says Mayamalavagaula's Re1 (256/243) maps to "morning surrender" therapy. But surrender, in the body, is *not* a static frequency at 231.77 Hz. Surrender is the body's nervous system *settling* — and that settling shows up in audio as **andolana**: a slow 1.5 Hz wobble around the swara, simulating the breath itself easing in and out.

Gamakas are how the rāga *meets the body*. Without them, the body-map is a dictionary; with them, it's a ritual.

---

**References:**
- Sārṅgadeva, *Saṅgīta Ratnākara*, Book IV (gamaka taxonomy)
- T. Viswanathan & M. Allen, *Music in South India* (2004), ch. 4 (gamaka in performance)
- Strudel pitch envelope docs — https://strudel.cc/learn/effects/
