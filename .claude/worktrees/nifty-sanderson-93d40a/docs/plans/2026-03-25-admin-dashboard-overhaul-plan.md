# Admin Dashboard Overhaul Plan

Date: 2026-03-25
Repo: `Sheshiyer/Selemene-engine`
Surface: `apps/admin-web`

## Discovery Summary

- Planning depth: deeply detailed
- Delivery mode: production redesign
- Release model: phased rollout
- Quality bar:
  - strong UX consistency
  - accessibility and keyboard support
  - responsive behavior
  - screenshot-level visual QA
  - deploy/canary readiness
- Team topology:
  - planner / tech lead
  - frontend implementation
  - backend / infra as needed
  - validation / QA

## Design Direction

This redesign should not imitate a generic SaaS dashboard. The target is a high-agency operational cockpit with the tone of a seasoned cartographer:

- maps, not prescriptions
- grounded, direct, respectful-challenging
- engineered calm rather than neon futurism

### Brand Inputs

- Primary dark: `Void Teal #0A1628`
- Light field: `Phosphor Cream #F0EDE3`
- Accent metal: `Solar Bronze #C4873B`
- Neutral support: `Titanium Mist #8A9DA8`
- Organic support: `Chlorophyll #4A7C59`
- Typography:
  - `Exo 2` for display numerics, high-signal lockups, and data signatures
  - `Space Grotesk` for interface text and control labels

### Reference Translation

From the provided HTML variants, keep:

- HUD-style status framing
- split-grid editorial composition
- dense drill-down reveal behavior
- mono micro-labels for telemetry
- strong border logic and panel segmentation

Do not keep:

- generic black-and-green cyberpunk color treatment
- thin-stroke techno clichés detached from Tryambakam branding
- purely decorative animation without management value

## Assumptions And Constraints

- Keep the existing route map:
  - `/dashboard`
  - `/users`
  - `/api-keys`
  - `/history-sync`
  - `/analytics`
  - `/system`
  - `/audit`
  - `/login`
- Preserve current backend contracts unless an issue explicitly extends them.
- The shipped API key management modal is baseline and should be visually integrated, not rebuilt from zero.
- Responsive support is mandatory down to mobile, even if the dashboard remains desktop-first.

## Agent Ownership Model

- Planner / Tech Lead:
  - design system decisions
  - issue graph
  - sequencing
  - integration review
- Frontend:
  - shell
  - primitives
  - page redesigns
  - motion and responsiveness
- Backend:
  - telemetry hooks
  - any contract gaps surfaced by redesign
  - rollout / deploy support
- Validation:
  - visual regression
  - accessibility checks
  - keyboard journeys
  - release gate evidence

## Phase Map

### Phase P1 — Brand System And Shell Foundation

Goal: establish the new visual language, token system, and structural shell.

### Phase P2 — Shared Interaction Infrastructure

Goal: build reusable management patterns so individual pages do not reinvent tables, drawers, filters, or feedback.

### Phase P3 — Page-Level Redesign

Goal: redesign each route into a coherent operational surface under the new system.

### Phase P4 — Workflow Hardening And Release

Goal: close the loop with bulk actions, QA, telemetry, and rollout readiness.

## Detailed Phase 1 Wave / Swarm Layout

### Wave W1 — Identity And Grammar

#### Swarm S1 — Brand extraction

- capture the redesign brief from the board and local HTML references
- convert colors and typography into explicit tokens
- define editorial tone and microcopy constraints

#### Swarm S2 — Visual grammar

- define ornament, linework, iconography, and illustration rules
- define when bronze filigree appears and where it is banned
- define mono telemetry usage and numeric hierarchy

### Wave W2 — Structural shell

#### Swarm S1 — Shell rebuild

- redesign sidebar, topbar, route framing, and context rails
- align shell spacing to the new grid and panel system

#### Swarm S2 — Primitive library

- create panel, card, modal, drawer, status, and metric primitives
- codify edge states and interaction feedback

### Wave W3 — UX guardrails

#### Swarm S1 — Reliability layer

- loading / empty / error / success surfaces
- focus rings, keyboard movement, z-index discipline
- responsive collapse rules

## Full Task List

The redesign is mapped to 34 GitHub issues.

| ID | Phase | Wave | Swarm | Area | Title | Dependencies |
|---|---|---|---|---|---|---|
| ADR-01 | P1 | W1 | S1 | product | Capture redesign brief and baseline screenshots | none |
| ADR-02 | P1 | W1 | S1 | frontend | Define Tryambakam color token system | ADR-01 |
| ADR-03 | P1 | W1 | S2 | frontend | Define typography hierarchy and numeric language | ADR-02 |
| ADR-04 | P1 | W1 | S2 | frontend | Define ornament, iconography, and illustration rules | ADR-02 |
| ADR-05 | P1 | W2 | S1 | frontend | Rebuild protected shell v2 | ADR-02, ADR-03, ADR-04 |
| ADR-06 | P1 | W2 | S1 | frontend | Build layout, panel, and card primitives | ADR-05 |
| ADR-07 | P1 | W2 | S2 | frontend | Build modal, drawer, and action rail primitives | ADR-06 |
| ADR-08 | P1 | W3 | S1 | qa | Build global loading, empty, error, and success states | ADR-06 |
| ADR-09 | P2 | W1 | S1 | frontend | Create data table v2 interaction model | ADR-06, ADR-07 |
| ADR-10 | P2 | W1 | S1 | frontend | Create metric tile and trend primitives | ADR-06 |
| ADR-11 | P2 | W1 | S2 | frontend | Create filter bar, search, and chip patterns | ADR-06 |
| ADR-12 | P2 | W1 | S2 | frontend | Create timeline and event stream primitives | ADR-06 |
| ADR-13 | P2 | W2 | S1 | frontend | Add responsive navigation and mobile behavior | ADR-05 |
| ADR-14 | P2 | W2 | S1 | frontend | Add motion system and reduced-motion rules | ADR-05, ADR-06 |
| ADR-15 | P2 | W2 | S2 | qa | Add accessibility, focus, and keyboard contracts | ADR-07, ADR-08 |
| ADR-16 | P2 | W3 | S1 | frontend | Add command palette and global quick actions | ADR-05, ADR-11 |
| ADR-17 | P3 | W1 | S1 | frontend | Redesign public login and auth entry | ADR-05, ADR-06 |
| ADR-18 | P3 | W1 | S2 | frontend | Redesign dashboard overview | ADR-09, ADR-10, ADR-14 |
| ADR-19 | P3 | W1 | S2 | frontend | Redesign users index | ADR-09, ADR-11 |
| ADR-20 | P3 | W2 | S1 | frontend | Build user detail drawer and action clusters | ADR-07, ADR-19 |
| ADR-21 | P3 | W2 | S2 | frontend | Re-theme API keys around the new design system | ADR-07, ADR-09 |
| ADR-22 | P3 | W2 | S2 | frontend | Redesign history sync page | ADR-09, ADR-12 |
| ADR-23 | P3 | W3 | S1 | frontend | Redesign analytics page | ADR-10, ADR-11, ADR-14 |
| ADR-24 | P3 | W3 | S1 | frontend | Redesign system health page | ADR-10, ADR-12 |
| ADR-25 | P3 | W3 | S2 | frontend | Redesign audit log page | ADR-09, ADR-12 |
| ADR-26 | P3 | W3 | S2 | frontend | Add page-level state variants across all screens | ADR-08, ADR-18, ADR-19, ADR-21, ADR-22, ADR-23, ADR-24, ADR-25 |
| ADR-27 | P4 | W1 | S1 | product | Add bulk action patterns for users and keys | ADR-19, ADR-20, ADR-21 |
| ADR-28 | P4 | W1 | S1 | frontend | Add breadcrumbs, deep links, and context persistence | ADR-05, ADR-16 |
| ADR-29 | P4 | W1 | S2 | frontend | Add destructive action safety taxonomy | ADR-07, ADR-21, ADR-27 |
| ADR-30 | P4 | W2 | S1 | backend | Add admin interaction telemetry and performance instrumentation | ADR-18, ADR-23, ADR-24 |
| ADR-31 | P4 | W2 | S1 | qa | Add screenshot-based visual regression coverage | ADR-18, ADR-19, ADR-21, ADR-22, ADR-23, ADR-24, ADR-25 |
| ADR-32 | P4 | W2 | S2 | qa | Add accessibility QA and keyboard journey tests | ADR-15, ADR-26 |
| ADR-33 | P4 | W3 | S1 | frontend | Run cross-page design polish and consistency pass | ADR-26, ADR-31, ADR-32 |
| ADR-34 | P4 | W3 | S2 | docs | Prepare rollout, canary checklist, and final acceptance signoff | ADR-30, ADR-31, ADR-32, ADR-33 |

## Dependency Rationale

- P1 freezes the aesthetic and structural contracts.
- P2 turns those contracts into reusable interaction primitives.
- P3 applies the system route by route.
- P4 hardens behavior, closes QA gaps, and prepares release.

The main collision zones are:

- `apps/admin-web/app/globals.css`
- `apps/admin-web/app/(protected)/layout.tsx`
- shared primitives under `apps/admin-web/src/components`

Those should be serialized at wave boundaries or split into explicit ownership slices.

## Verification Strategy

- every page redesign must pass:
  - `npm --prefix apps/admin-web run typecheck`
  - `npm --prefix apps/admin-web run lint`
  - `npm --prefix apps/admin-web run build`
- visual work must include before/after screenshots
- interactive flows must include keyboard-path checks
- responsive verification must include mobile and desktop evidence
- release readiness must include live canary checks on the deployed admin portal

## GitHub Sync Strategy

- one issue per task
- titles encoded as `[P?][W?][S?] ADR-?? — ...`
- labels:
  - `roadmap`
  - `enhancement`
  - `taskmaster`
  - phase label
  - wave label
  - area label
- dependencies recorded directly in each issue body
- the plan file should be referenced from issue bodies as the canonical source document

## Risks And Fallbacks

- Risk: redesign drifts into decorative slop
  - Fallback: enforce token system and page archetypes before route implementation
- Risk: shell rewrite destabilizes route permissions or auth checks
  - Fallback: isolate shell visuals from auth logic until validation passes
- Risk: table / drawer / modal primitives fork per page
  - Fallback: stop page work until shared primitives are frozen
- Risk: animation harms performance or seriousness
  - Fallback: keep motion subtle, telemetry-oriented, and optional under reduced-motion
