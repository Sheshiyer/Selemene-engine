# Accessibility QA Checklist — Admin Web

> ADR-32 | Area: QA | Owner: Tech Lead

## Scope

This document records the accessibility validation pass for the admin shell redesign. Each section maps to a UI area and its keyboard/AT acceptance criteria.

---

## 1. Shell Navigation

| Journey | Expected behaviour | Status |
|---|---|---|
| Tab through sidebar nav links | Focus moves sequentially; active link has `aria-current="page"` | ✅ Implemented |
| Enter on locked nav item | Navigates (permission error shown on destination); lock state conveyed via `aria-disabled` | ✅ Implemented |
| Tab to topbar action rail buttons | All 4 buttons reachable; focus rings visible (2px Solar Bronze outline) | ✅ CSS `:focus-visible` rule present |

## 2. Command Palette (Cmd/Ctrl + K)

| Journey | Expected behaviour | Status |
|---|---|---|
| Activate with keyboard shortcut | Palette opens; focus placed on search input | ✅ `requestAnimationFrame` focus call |
| Search input role | `role="combobox"` with `aria-autocomplete="list"`, `aria-expanded`, `aria-controls`, `aria-activedescendant` | ✅ Implemented |
| Arrow key navigation | Up/Down moves highlight; `aria-activedescendant` updates to point at active item ID | ✅ Implemented |
| Enter to select | Executes item; palette closes; focus returns naturally | ✅ |
| Escape to close | Palette closes | ✅ `useEscapeToClose` |
| Screen reader announces active item | `aria-activedescendant` + `aria-selected` on `role="option"` items | ✅ Implemented |

## 3. Modal Dialogs (ModalSurface)

| Journey | Expected behaviour | Status |
|---|---|---|
| Dialog role and label | `role="dialog"`, `aria-modal="true"`, `aria-labelledby` pointing to title `<h3>` | ✅ Implemented |
| Escape to close | Closes dialog | ✅ `useEscapeToClose` |
| Close button | `aria-label="Close dialog"` | ✅ |
| Focus trap | Focus should not leave the modal while open | ⚠️ **Deferred** — no focus trap hook implemented; manual testing required |

## 4. Drawer Panels (DrawerSurface)

| Journey | Expected behaviour | Status |
|---|---|---|
| Drawer role and label | `role="dialog"`, `aria-modal="true"`, `aria-labelledby` pointing to title | ✅ Implemented |
| Escape to close | Closes drawer | ✅ |
| Close button | `aria-label="Close drawer"` | ✅ |

## 5. Data Tables

| Journey | Expected behaviour | Status |
|---|---|---|
| Table headers | `<th scope="col">` on all column headers | 🔲 **Needs review** — route-level tables use custom markup |
| Row selection checkboxes | `aria-label` describing selected row | 🔲 **Needs review** |
| Sortable columns | `aria-sort` attribute when sorted | 🔲 **Needs review** |

## 6. Bulk Action Bar

| Journey | Expected behaviour | Status |
|---|---|---|
| Bar announced | `aria-label="{itemLabel} bulk actions"` on section | ✅ Already present |
| Select/Clear buttons | Standard buttons, focusable | ✅ |
| Disabled state on Clear | `disabled` attribute when `selectedCount === 0` | ✅ |

## 7. Form Inputs

| Journey | Expected behaviour | Status |
|---|---|---|
| Filter/search inputs | Associated `<label>` or `aria-label` | 🔲 **Needs review per route** |
| Error messages | `aria-describedby` linking input to error text | 🔲 **Needs review** |

---

## Known Deferred Items

- **Focus trap in modals/drawers**: No `focus-trap` library installed. A future sprint should add focus trapping so keyboard users cannot tab out of an open modal.
- **Table-level a11y**: Individual route pages (users, api-keys, audit) need per-column `scope="col"` headers and `aria-sort` reviewed.
- **Dynamic ARIA live regions**: Bulk action count changes and state banners should use `aria-live="polite"` for screen reader announcements.

## Test Evidence

Run keyboard-only journeys in Chrome with VoiceOver (macOS) or NVDA (Windows):
1. Login → Dashboard via keyboard only
2. Open/close command palette, select a route item
3. Open a drawer, tab through fields, close
4. Use bulk action bar on Users page

Attach screen recordings as PR comments when closing deferred items.
