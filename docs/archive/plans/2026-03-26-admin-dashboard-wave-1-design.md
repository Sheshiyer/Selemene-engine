# Admin Dashboard Redesign Wave 1 Design

Date: 2026-03-26
Scope: `P1 / W1`
Issues: `ADR-01`, `ADR-02`, `ADR-03`, `ADR-04`
Surface: `apps/admin-web`

## Scope Decision

This pass treats "Wave 1" as the first executable redesign wave in the dependency graph: `P1 / W1`.

I am not claiming the later `W1` issues across `P2`, `P3`, and `P4` here because they depend on:

- `ADR-05` shell rebuild
- `ADR-06` layout / panel / card primitives
- `ADR-07` modal / drawer / action rail primitives
- `ADR-14` motion system
- page-level redesign tasks not yet landed

Wave 1 in this pass therefore means:

- capture current-state baseline
- freeze the token system
- freeze typography hierarchy
- freeze ornament and iconography rules

## Baseline Inventory

### Public Routes

- `/admin/login`
- `/admin/login/discord-callback`
- `/admin/auth/discord/callback`

### Protected Routes

- `/admin/dashboard`
- `/admin/users`
- `/admin/api-keys`
- `/admin/history-sync`
- `/admin/analytics`
- `/admin/system`
- `/admin/audit`

### Current Shared UI Characteristics

Derived from current source:

- [layout.tsx](../../apps/admin-web/app/(protected)/layout.tsx) uses a left sidebar plus topbar shell with teal-on-dark styling.
- [page-shell.tsx](../../apps/admin-web/src/components/page-shell.tsx) is the shared page container for protected routes.
- [globals.css](../../apps/admin-web/app/globals.css) is the primary collision zone for palette, spacing, and component chrome.
- [login-client.tsx](../../apps/admin-web/app/(public)/login/login-client.tsx) is structurally stable and should be visually recast later without changing auth flow semantics.

### Before Evidence

Captured from the local app on 2026-03-26:

- Desktop login baseline: `docs/assets/admin-dashboard-wave-1-baseline/login-desktop.png`
- Mobile login baseline: `docs/assets/admin-dashboard-wave-1-baseline/login-mobile.png`

Protected routes are session-gated, so this pass records their baseline from source inventory and shared shell structure rather than pretending to have authenticated screenshots.

## Current-State Problems

The current UI works functionally, but visually it is still closer to a generic cyberpunk admin starter than the Tryambakam Noesis brand system.

Problems to fix at the grammar level:

- teal accent dominates instead of bronze-led hierarchy
- headings and numerics do not carry a distinct display voice
- mono usage is inconsistent and not limited to telemetry
- panel chrome is competent but generic
- shell surfaces do not yet communicate "seasoned cartographer" calm
- the palette does not reflect the warm cream / bronze / chlorophyll board

## Wave 1 Decisions

### Tone

The product voice for admin surfaces is:

- grounded
- direct
- respectful-challenging
- operational, not theatrical

Microcopy rule:

- describe state clearly
- do not over-celebrate
- do not sound like a neon sci-fi prop
- prefer management language over marketing language

Examples:

- prefer `Session unavailable` over `Oops`
- prefer `Operational Surface` over `Magic View`
- prefer `Refresh` / `Rotate` / `Delete` over cute phrasing

## Token System

### Source Colors

- `Void Teal #0A1628`
- `Phosphor Cream #F0EDE3`
- `Solar Bronze #C4873B`
- `Titanium Mist #8A9DA8`
- `Chlorophyll #4A7C59`

### Semantic Mapping

| Token | Value | Use |
|---|---|---|
| `--bg` | `#0A1628` | global application field |
| `--bg-elevated` | `#112035` | elevated panels and cards |
| `--bg-soft` | `#182A3F` | nested controls |
| `--bg-panel` | `rgba(14, 24, 39, 0.9)` | glass-dark surfaces |
| `--field` | `rgba(240, 237, 227, 0.06)` | form control base |
| `--line` | `rgba(138, 157, 168, 0.28)` | borders |
| `--line-strong` | `rgba(138, 157, 168, 0.52)` | active borders |
| `--text` | `#F0EDE3` | primary text |
| `--text-muted` | `#B7C0C6` | helper text |
| `--accent` | `#C4873B` | bronze emphasis |
| `--accent-strong` | `#DFAA69` | hover / focus / premium emphasis |
| `--support` | `#4A7C59` | success / healthy state |
| `--signal` | `#8A9DA8` | technical support tone |
| `--danger` | `#EF6B73` | destructive state |

### Palette Rules

- Bronze is the primary emphasis color.
- Chlorophyll is reserved for positive system state and success only.
- Titanium Mist is the structural neutral, not the main accent.
- Cream should read as signal text and soft field glow, not flat white background slabs inside the admin shell.

## Typography Hierarchy

### Families

- Display: `Exo 2`
- Interface text: `Space Grotesk`
- Telemetry / metadata: `IBM Plex Mono`

### Rules

| Layer | Family | Use |
|---|---|---|
| display | `Exo 2` | route titles, metric numerics, major lockups |
| sans | `Space Grotesk` | body text, controls, tables, summaries |
| mono | `IBM Plex Mono` | labels, pills, telemetry, tiny metadata |

### Hierarchy Logic

- Route titles: uppercase `Exo 2`, modest tracking
- Metric values: `Exo 2`, larger than section headings
- Labels / table headers / pills: mono uppercase
- Body copy: `Space Grotesk`, readable and restrained

Mono is not allowed for entire cards or paragraphs.

## Ornament And Iconography Rules

### Allowed Ornament

- thin bronze glows at panel edges
- subtle inner frame treatment on major panels
- single-line ornamental dividers in hero or section break contexts
- low-opacity bronze radial highlights at top corners

### Banned Ornament

- decorative curls inside data tables
- bronze flourishes behind charts
- ornamental noise on every card
- neon glows, scanner lines, or blinking HUD theatrics

### Iconography

- line-based SVG icons only
- stroke weight should feel engineered, not playful
- no emoji
- no filled cartoon icon set

### Telemetry Rule

Mono telemetry is allowed for:

- table headers
- status pills
- eyebrow labels
- tiny metadata captions

Mono telemetry is banned for:

- route summaries
- modal descriptions
- long helper paragraphs

## Implementation Target For This Pass

Wave 1 should leave the codebase with:

- the brand token system in global CSS
- `Exo 2` wired at the root layout
- shared heading / metric / label hierarchy visible in the current shell
- reusable visual grammar helpers (`filigree-frame`, `eyebrow`, `telemetry-caption`, `ornament-rule`)

Wave 1 should not yet claim:

- shell rebuild completion
- login redesign completion
- dashboard or users redesign completion
- bulk actions, breadcrumbs, or destructive-flow taxonomy completion

## Verification Target

For code in this pass:

- `npm --prefix apps/admin-web run typecheck`
- `npm --prefix apps/admin-web run lint`
- `npm --prefix apps/admin-web run build`

For evidence in this pass:

- baseline desktop login screenshot exists
- baseline mobile login screenshot exists
- source inventory is documented for all protected routes
