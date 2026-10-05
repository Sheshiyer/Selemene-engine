// ─── Section Manifest Validator ───────────────────────────────────────
// Validates the structural section contract for the reference-aligned route.
// Checks are structural (based on the declared section ID manifest), NOT
// prose regex searches for missing words.
//
// Rules enforced:
//   - No empty section IDs ('' or whitespace-only)
//   - No duplicate section IDs within the manifest
//   - Every pass result ID must appear in the manifest (unknown IDs block)
//   - Every required section ID in the manifest must appear in pass results
//   - Missing required sections are blockers
//
// This replaces any "missing word regex" approach described in the repair spec.

export interface SectionManifestEntry {
  /** Stable section identifier (e.g. 'opening', 'part1', 'part11') */
  id: string;
  /** Human-readable title for error messages */
  title?: string;
  /** Whether this section is required for final acceptance */
  required: boolean;
}

export interface SectionManifest {
  sections: SectionManifestEntry[];
}

export interface ManifestValidationResult {
  passed: boolean;
  blockers: string[];
  warnings: string[];
}

/**
 * Validate a SectionManifest itself for structural soundness:
 * - No empty IDs
 * - No duplicate IDs
 */
export function validateManifestStructure(manifest: SectionManifest): ManifestValidationResult {
  const blockers: string[] = [];
  const warnings: string[] = [];
  const seen = new Set<string>();
  if (!manifest.sections.length) blockers.push("manifest:empty");

  for (let i = 0; i < manifest.sections.length; i++) {
    const entry = manifest.sections[i];
    const id = entry.id?.trim();
    if (!id) {
      blockers.push(`manifest:empty_section_id — section at index ${i} has an empty or whitespace-only ID`);
      continue;
    }
    if (seen.has(id)) {
      blockers.push(`manifest:duplicate_section_id:${id} — section ID appears more than once`);
    } else {
      seen.add(id);
    }
  }

  return { passed: blockers.length === 0, blockers, warnings };
}

/**
 * Validate pass results against a structural section manifest.
 *
 * - Required sections that produced no pass result are blockers.
 * - Pass results whose ID does not appear in the manifest are blockers
 *   (guards against unknown sections being silently included in assembly).
 * - Optional sections that produced no pass result are warnings.
 */
export function validatePassResultsAgainstManifest(
  passResultIds: string[],
  manifest: SectionManifest,
): ManifestValidationResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  // First validate manifest structure itself
  const structureCheck = validateManifestStructure(manifest);
  blockers.push(...structureCheck.blockers);
  warnings.push(...structureCheck.warnings);
  if (blockers.length > 0) {
    // Cannot reason about results against a structurally invalid manifest
    return { passed: false, blockers, warnings };
  }

  const manifestIds = new Set(manifest.sections.map((s) => s.id));
  const resultIds = new Set(passResultIds);
  if (resultIds.size !== passResultIds.length) blockers.push('manifest:duplicate_result_id');

  // Unknown section IDs in results
  for (const id of resultIds) {
    if (!manifestIds.has(id)) {
      blockers.push(`manifest:unknown_section_id:${id} — pass result ID not declared in section manifest`);
    }
  }

  // Required sections missing from results
  for (const entry of manifest.sections) {
    if (entry.required && !resultIds.has(entry.id)) {
      blockers.push(`manifest:missing_required_section:${entry.id} — required section produced no pass result`);
    } else if (!entry.required && !resultIds.has(entry.id)) {
      warnings.push(`manifest:missing_optional_section:${entry.id} — optional section produced no pass result`);
    }
  }

  return { passed: blockers.length === 0, blockers, warnings };
}

/**
 * Build a SectionManifest from an ordered list of section IDs and a set of
 * required IDs. All sections are required by default unless requiredIds
 * is provided; if provided, only sections in the set are marked required.
 */
export function buildManifestFromIds(
  orderedIds: string[],
  requiredIds?: Set<string>,
): SectionManifest {
  return {
    sections: orderedIds.map((id) => ({
      id,
      required: requiredIds ? requiredIds.has(id) : true,
    })),
  };
}
