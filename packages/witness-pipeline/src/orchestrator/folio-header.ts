// ─── Folio B-surface Relationship Header Renderer (Phase 6) ─────────────
// Declarative, non-presumptive header for long-form Folio output.
// Consumes relationship_header contract + labeled subjects + mapping_goal + sensitivity.
// Voice per docs/design-system/SYSTEM.md: parchment canvas, ink-bronze body, ink-iron headings, illuminated style.
// Header must appear at top of long-form reading before engine data.
// Output format (markdown): starts with # heading, then subjects/mapping/sensitivity lines.
// Never presumes romance; uses explicit roles + "non-predictive pattern witness".

export interface SubjectRoleInfo {
  role: string;
  name: string;
  label?: string;
}

export interface RelationshipContextInfo {
  type: string;
  mapping_goal: string;
  sensitivity_level: 'low' | 'medium' | 'high';
}

export interface FolioHeaderInput {
  subjectRoles: SubjectRoleInfo[];
  relationshipContext: RelationshipContextInfo;
}

/**
 * Render a Folio-style relationship header.
 * Produces markdown starting with # for ink-iron heading cue.
 * Title form: "{Role-Pair} {TypeLabel} — non-predictive pattern witness"
 * Examples:
 *   "Mother-Son Lineage Mapping — non-predictive pattern witness"
 *   "Business-Partners Synergy Audit — non-predictive pattern witness"
 */
export function renderFolioRelationshipHeader(input: FolioHeaderInput): string {
  const roles = input.subjectRoles || [];
  const ctx = input.relationshipContext;

  if (!ctx) {
    return '';
  }

  // Build human title from roles + type (declarative, non-presumptive)
  const rolePair = buildRolePair(roles);
  const typeLabel = mapTypeToLabel(ctx.type);
  const title = `${rolePair} ${typeLabel} — non-predictive pattern witness`;

  // Subjects line with optional labels
  const subjectsLine = roles.length > 0
    ? roles.map(r => {
        const base = `${r.name} (${r.role}`;
        const withLabel = r.label ? `${base}, ${r.label}` : base;
        return `${withLabel})`;
      }).join(', ')
    : '';

  const goalLine = ctx.mapping_goal ? `Mapping goal: ${ctx.mapping_goal}` : '';
  const sensLine = `Sensitivity: ${ctx.sensitivity_level}`;

  const lines = [
    `# ${title}`,
    subjectsLine ? `Subjects: ${subjectsLine}` : '',
    goalLine,
    sensLine,
  ].filter(Boolean);

  return lines.join('\n');
}

function capitalizeRole(role: string): string {
  // Convert kebab/snake to Title-Case words for display
  return role
    .split(/[-_]/)
    .map(w => w.length ? w[0].toUpperCase() + w.slice(1) : '')
    .join('-');
}

function buildRolePair(roles: SubjectRoleInfo[]): string {
  if (!roles.length) return 'Relationship';
  // For identical repeated roles (e.g. two business-partner), use plural form
  const unique = Array.from(new Set(roles.map(r => r.role)));
  if (unique.length === 1) {
    const r = unique[0];
    if (r === 'business-partner') return 'Business-Partners';
    if (r === 'partner') return 'Partners';
    // generic plural for repeated role
    return capitalizeRole(r) + 's';
  }
  // Dyad with distinct roles: Mother-Son, Parent-Child etc.
  return roles.map(r => capitalizeRole(r.role)).join('-');
}

function mapTypeToLabel(type: string): string {
  switch (type) {
    case 'family':
      return 'Lineage Mapping';
    case 'business-partners':
      return 'Synergy Audit';
    case 'unmarried-partners':
    case 'married-partners':
      return 'Dyad Witness';
    case 'friends':
      return 'Field Mapping';
    default:
      return 'Relationship Mapping';
  }
}
