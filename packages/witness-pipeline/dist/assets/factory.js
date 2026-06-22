// ─── Premium Source-Pack Factory ───────────────────────────────────────
// Creates deterministic source packs: manifest, reading markdown, and
// reflection questions. HTML/PDF rendering is out of scope for this pass.
import { promises as fs } from 'node:fs';
import { join } from 'node:path';
const DEFAULT_REFLECTION_QUESTIONS = [
    'What is the one thing from this reading that feels most alive right now?',
    'Where do you notice resistance, and what is it protecting?',
    'What is the smallest step that honors what this reading named?',
];
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
function countDeterministicFacts(engineResults, deterministicOnly) {
    return engineResults.filter((e) => {
        if (e._error)
            return false;
        if (deterministicOnly)
            return DETERMINISTIC_ENGINES.has(e.engine_id);
        return DETERMINISTIC_ENGINES.has(e.engine_id);
    }).length;
}
export async function createSourcePack(input) {
    await fs.mkdir(input.outputDir, { recursive: true });
    const deterministicOnly = input.deterministicOnly ?? false;
    const factsCount = countDeterministicFacts(input.engineResults, deterministicOnly);
    const manifest = {
        person_id: input.personId,
        created_at: new Date().toISOString(),
        reading_length: input.readingMarkdown.length,
        engines: input.engineResults.map((e) => e.engine_id),
        quality: {
            facts_count: factsCount,
            gate_status: factsCount >= 3 ? 'ready' : 'blocked',
        },
    };
    const manifestPath = join(input.outputDir, 'manifest.json');
    const questionsPath = join(input.outputDir, 'reflection-questions.md');
    const readingPath = join(input.outputDir, 'reading.md');
    await fs.writeFile(manifestPath, JSON.stringify(manifest, null, 2), 'utf-8');
    await fs.writeFile(questionsPath, DEFAULT_REFLECTION_QUESTIONS.map((q) => `- ${q}`).join('\n'), 'utf-8');
    await fs.writeFile(readingPath, input.readingMarkdown, 'utf-8');
    return {
        outputDir: input.outputDir,
        manifest,
        reflectionQuestions: DEFAULT_REFLECTION_QUESTIONS,
        paths: {
            manifest: manifestPath,
            reading: readingPath,
            questions: questionsPath,
        },
    };
}
//# sourceMappingURL=factory.js.map