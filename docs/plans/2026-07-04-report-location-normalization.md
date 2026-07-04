# Report Location Normalization Design Note

## Purpose

Report generation must receive a confirmed normalized birthplace for every required subject before any chart or synthesis work begins. The normalized shape is a user-confirmed place record containing display name, latitude, longitude, and IANA timezone. The system must not silently choose an ambiguous geocoding result.

## Existing Precedent

`tools/humdes-extractor/humdes_to_selemene.py` already establishes the repo precedent for location enrichment: cached, rate-limited Nominatim lookup that stores `lat`, `lng`, `display_name`, `timezone`, and `humdes_sex` for downstream Selemene-compatible records.

The report intake layer should reuse this pattern conceptually for low-cost local and batch flows, while adding explicit user selection and confirmation instead of silent best-guess geocoding.

## Provider Strategy

| Surface | Default | Fallback | Rationale |
|---|---|---|---|
| CLI / local agent | Cached Nominatim picker | Manual latitude, longitude, and timezone | Free, rate-limitable, sufficient for operator workflows |
| Web UI | Google Places or Mapbox picker | Manual latitude, longitude, and timezone | Better disambiguation, search UX, and place-result quality |
| Batch / import | Pre-normalized latitude, longitude, and timezone | Cached Nominatim lookup when explicitly enabled | Reproducible imports should not depend on live geocoding by default |
| Private / sensitive mode | Manual coordinates only | None | Avoids sending birthplace text to any third-party geocoder |

## Manual Privacy Mode

Manual privacy mode is required for sensitive reports. In this mode:

- The system must not call Nominatim, Google Places, Mapbox, GeoNames, or any other third-party geocoder.
- The operator or user must provide display name, latitude, longitude, and timezone directly.
- The resulting location provider should be recorded as `manual` with `manual` confidence.
- The manually supplied place label should be treated as report input, not as a lookup key.

## Confirmation Contract

Every interactive flow must show the selected location back to the user before generation:

```text
Birthplace selected: Jamakhandi, Bagalkote, Karnataka, India
Latitude: 16.5046
Longitude: 75.2918
Timezone: Asia/Kolkata
Use this location?
```

The user must be able to accept the selected result, choose another candidate, or enter coordinates manually.

## Timezone Confirmation

Timezone must be confirmed as an IANA timezone string, such as `Asia/Kolkata`, `America/New_York`, or `Europe/London`.

No report should run until each required subject has either:

- A confirmed normalized location from a picker result.
- An explicit manual latitude, longitude, and IANA timezone override.

Ambiguous geocoding results should remain blocked until the user selects a specific candidate or switches to manual entry.
