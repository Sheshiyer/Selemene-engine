// ─── Asset Chain Audit ─────────────────────────────────────────────────
// Validates a source pack against deterministic-fact gates.
const DETERMINISTIC_ENGINES = new Set([
    'panchanga',
    'vimshottari',
    'human-design',
    'gene-keys',
    'numerology',
    'biorhythm',
    'vedic-clock',
    'transits',
    'enneagram',
]);
const ORACLE_ENGINES = new Set(['tarot', 'i-ching', 'sacred-geometry', 'sigil-forge']);
const SOMATIC_ENGINES = new Set(['biofield', 'face-reading', 'nadabrahman']);
export function runChainAudit(input) {
    const blockers = [];
    const warnings = [];
    const deterministic = input.engineResults.filter((e) => DETERMINISTIC_ENGINES.has(e.engine_id) && !e._error);
    const factsCount = deterministic.length;
    if (factsCount < 3) {
        blockers.push(`Only ${factsCount} deterministic engines present; need at least 3.`);
    }
    if (input.deterministicOnly) {
        for (const e of input.engineResults) {
            if (ORACLE_ENGINES.has(e.engine_id)) {
                blockers.push(`Oracle engine ${e.engine_id} present but deterministic-only mode is enabled.`);
            }
            if (SOMATIC_ENGINES.has(e.engine_id)) {
                blockers.push(`Somatic engine ${e.engine_id} present but deterministic-only mode is enabled.`);
            }
        }
    }
    if (input.readingMarkdown.length < 100) {
        warnings.push('Reading markdown is very short.');
    }
    return {
        person_id: input.personId,
        blockers,
        warnings,
        facts_count: factsCount,
        passed: blockers.length === 0,
    };
}
//# sourceMappingURL=audit.js.map