# FreeAstrologyAPI — Live Endpoint Discovery (2026-05-19)

> Outcome of a hands-on probe of `https://json.freeastrologyapi.com` with a
> live API key. Documents the **actual** working surface vs the surface the
> `noesis-vedic-api` Rust crate *thinks* exists.

## TL;DR

The `noesis-vedic-api` crate calls **10 endpoints**. Only **2** of them resolve.
The other 8 return `HTTP 403 — "Missing Authentication Token"` (AWS API
Gateway's response for "this route does not exist on the deployed API").

This means **70 % of the Rust crate is non-functional against the live
vendor API**. The crate is shaped against a hypothesised schema, not the
real one.

## Probe parameters

- Endpoint base: `https://json.freeastrologyapi.com`
- Auth header: `x-api-key: <key>` (confirmed working)
- Method: `POST` with JSON body (vendor uses POST even for what look like reads)
- Body shape (consistent across all working endpoints):
  ```json
  {
    "year": 1991, "month": 8, "date": 13,
    "hours": 13, "minutes": 31, "seconds": 0,
    "latitude": 12.9716, "longitude": 77.5946, "timezone": 5.5,
    "config": { "observation_point": "topocentric", "ayanamsha": "lahiri" }
  }
  ```
- `HTTP 403 "Missing Authentication Token"` ≡ route does not exist
- `HTTP 429` ≡ rate limited (route does exist, retry after delay)
- `HTTP 200` ≡ working

## Working endpoints (confirmed HTTP 200)

| Path                              | Returns                                                                 |
|-----------------------------------|-------------------------------------------------------------------------|
| `POST /planets`                   | Ascendant (lagna) + 13 grahas (Sun..Pluto + Rahu/Ketu): `fullDegree`, `normDegree`, `current_sign` (1=Aries..12=Pisces), `house_number`, `isRetro` |
| `POST /planets/extended`          | Same plus extra fields                                                  |
| `POST /western/planets`           | Tropical equivalent of `/planets`                                       |
| `POST /western/houses`            | 12 house cusps with sign assignments (Placidus default)                 |
| `POST /d2-chart-info`             | Hora chart (D2)                                                         |
| `POST /d3-chart-info`             | Drekkana chart (D3)                                                     |
| `POST /d4-chart-info`             | Chaturthamsa (D4)                                                       |
| `POST /d5-chart-info`             | Panchamamsa (D5)                                                        |
| `POST /d6-chart-info`             | Shashthamsa (D6)                                                        |
| `POST /d7-chart-info`             | Saptamamsa (D7)                                                         |
| `POST /d8-chart-info`             | Ashtamamsa (D8)                                                         |
| `POST /d9-chart-info` ≡ `/navamsa-chart-info` | Navamsa (D9)                                                |
| `POST /d10-chart-info`            | Dashamamsa (D10)                                                        |
| `POST /d12-chart-info`            | Dvadasamsa (D12)                                                        |
| `POST /d16-chart-info`            | Shodasamsa (D16)                                                        |
| `POST /d20-chart-info`            | Vimsamsa (D20)                                                          |
| `POST /d24-chart-info`            | Chaturvimsamsa (D24)                                                    |
| `POST /d27-chart-info`            | Saptavimsamsa / Nakshatramsa (D27)                                      |
| `POST /d30-chart-info`            | Trimsamsa (D30)                                                         |
| `POST /d40-chart-info`            | Khavedamsa (D40)                                                        |
| `POST /d45-chart-info`            | Akshavedamsa (D45)                                                      |
| `POST /d60-chart-info`            | Shashtiamsa (D60)                                                       |
| `POST /horoscope-chart-svg-code`  | Inline SVG markup of natal chart                                        |
| `POST /navamsa-chart-svg-code`    | Inline SVG markup of D9                                                 |

That gives us — **for free, no engine work** — **lagna + 13 planets + 12 houses
+ all 16 divisional charts + SVG rendering**.

## Non-existent endpoints (HTTP 403, route absent)

### Called by `noesis-vedic-api` but do not exist

| Rust module                  | Endpoint called                  | Vendor status |
|------------------------------|----------------------------------|---------------|
| `panchang/api.rs`            | `GET /panchang`                  | 403           |
| `panchang/api.rs`            | `GET /sunrise-sunset`            | 403           |
| `vimshottari/api.rs`         | `POST /vimshottari-dasha`        | 403           |
| `birth_chart/api.rs`         | `POST /horoscope-chart`          | 403           |
| `vargas/api.rs`              | `POST /horoscope-chart/varga`    | 403           |
| `transits/api.rs`            | `POST /transits`                 | 403           |
| `yogas/api.rs`               | `POST /yogas`                    | 403           |
| `shadbala/api.rs`            | `POST /shadbala`                 | 403           |
| `ashtakavarga/api.rs`        | `POST /ashtakavarga`             | 403           |
| `muhurta/api.rs`             | `POST /muhurta`                  | 403           |

### Other plausible names that also do not exist

`/houses`, `/aspects`, `/natal-aspects`, `/planet-aspects`, `/maha-dasha`,
`/antar-dasha`, `/mahadashas`, `/pratyantar-dasha`, `/current-mahadasha`,
`/tithi`, `/nakshatra`, `/yoga`, `/karana`, `/hindu-month`, `/sun-rise`,
`/sun-set`, `/festival-list`, `/ayanamsha-info`, `/dasha-periods`,
`/sub-dashas`.

## Sample `POST /planets` response

```json
{
  "statusCode": 200,
  "input": { "year": 1991, "month": 8, "date": 13, "hours": 13, "minutes": 31,
             "seconds": 0, "latitude": 12.9716, "longitude": 77.5946,
             "timezone": 5.5,
             "config": { "observation_point": "topocentric",
                         "ayanamsha": "lahiri" } },
  "output": [{
    "0":  { "name": "Ascendant", "fullDegree": 222.118, "normDegree": 12.118,
            "isRetro": "false", "current_sign": 8 },
    "1":  { "name": "Sun",     "fullDegree": 116.351, "normDegree": 26.351,
            "isRetro": "false", "current_sign": 4,  "house_number": 9 },
    "2":  { "name": "Moon",    "fullDegree": 160.257, "normDegree": 10.257,
            "isRetro": "false", "current_sign": 6,  "house_number": 11 },
    "3":  { "name": "Mars",    "fullDegree": 144.203, "normDegree": 24.203,
            "isRetro": "false", "current_sign": 5,  "house_number": 10 },
    "4":  { "name": "Mercury", "fullDegree": 130.886, "normDegree": 10.886,
            "isRetro": "true",  "current_sign": 5,  "house_number": 10 },
    "5":  { "name": "Jupiter", "fullDegree": 119.764, "normDegree": 29.764,
            "isRetro": "false", "current_sign": 4,  "house_number": 9 },
    "6":  { "name": "Venus",   "fullDegree": 130.821, "normDegree": 10.821,
            "isRetro": "true",  "current_sign": 5,  "house_number": 10 },
    "7":  { "name": "Saturn",  "fullDegree": 278.523, "normDegree":  8.523,
            "isRetro": "true",  "current_sign": 10, "house_number": 3 },
    "8":  { "name": "Rahu",    "fullDegree": 263.515, "normDegree": 23.515,
            "isRetro": "true",  "current_sign": 9,  "house_number": 2 },
    "9":  { "name": "Ketu",    "fullDegree":  83.515, "normDegree": 23.515,
            "isRetro": "true",  "current_sign": 3,  "house_number": 8 },
    "10": { "name": "Uranus",  "fullDegree": 256.633, "normDegree": 16.633,
            "isRetro": "true",  "current_sign": 9,  "house_number": 2 },
    "11": { "name": "Neptune", "fullDegree": 260.735, "normDegree": 20.735,
            "isRetro": "true",  "current_sign": 9,  "house_number": 2 },
    "12": { "name": "Pluto",   "fullDegree": 203.894, "normDegree": 23.894,
            "isRetro": "false", "current_sign": 8,  "house_number": 1 }
  }]
}
```

Key fields per planet:
- `fullDegree` ∈ [0, 360): absolute ecliptic longitude (sidereal when
  `ayanamsha=lahiri`)
- `normDegree` ∈ [0, 30): degree within current sign
- `current_sign` ∈ [1, 12]: sign number, Aries=1
- `house_number` ∈ [1, 12]: whole-sign house counted from Ascendant
- `isRetro`: stringified bool

## Implication for validation harness

The Vedic-validation initiative must split its ground truth into **two**
sources, because no single source covers everything:

| Field set                | Ground-truth source                                    |
|--------------------------|--------------------------------------------------------|
| Lagna, 13 planets, signs, houses, retrograde flag, divisional charts | `POST /planets`, `POST /d*-chart-info`, `POST /western/houses` |
| Tithi, Nakshatra, Yoga, Karana, Vaara, Sunrise/Sunset | `pyswisseph` script at `tools/humdes-extractor/compute_vedic_kundali.py` (Swiss-ephemeris truth) |
| Vimshottari Mahadasha timeline | same pyswisseph script (Lahiri-checked)               |
| Yogas (Raj, Dhana, etc.) | same pyswisseph script                                |

The pyswisseph script is **already deterministic and reproducible** because
all 89 humdes fixtures already have birth coordinates + time. We use it as
authority for everything FreeAstrologyAPI doesn't supply.

## Implication for `noesis-vedic-api`

Three options, in order of project value:

1. **Retire the 8 dead modules** (`panchang`, `vimshottari`, `transits`,
   `yogas`, `shadbala`, `ashtakavarga`, `muhurta`, `birth_chart` *as
   currently shaped*) and rebuild a thin client around the working
   endpoints only (`planets`, `western/houses`, `dN-chart-info`,
   `*-svg-code`).
2. **Tag the dead modules** with `#[cfg(feature = "mock-only")]` and keep
   them for tests; clearly mark them as not wired to a live API.
3. **Replace them with native engines** (`engine-panchanga`,
   `engine-vimshottari`, `engine-transits` already exist and compute
   their own — they just need to be re-exposed through `noesis-vedic-api`
   as a façade).

Recommendation: option **3**. The native Rust engines + pyswisseph already
give us better fidelity than the vendor would; we just need to surface
them under the `noesis-vedic-api` API.

## Probe artefact

Raw probe results: `/tmp/endpoint-probe-2.txt`,
`/tmp/endpoint-probe-3.txt`, `/tmp/planets.json`, `/tmp/houses.json`.
(Local-only, not committed.)
