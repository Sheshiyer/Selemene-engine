import type { SelemeneEngineOutput } from '../index.js';

export function formatEngineResultsForPrompt(
  engineResults: SelemeneEngineOutput[],
): string {
  const keyFacts = extractKeyFactsLine(engineResults);
  const blocks = engineResults.map((engine) => {
    const result = engine.result ?? {};
    return `### ${engine.engine_id}\n\n${jsonBlock(result)}`;
  });
  return `## Deterministic Engine Results\n\n### Key facts\n\n${keyFacts}\n\n${blocks.join('\n\n')}`;
}

function getResult(engines: SelemeneEngineOutput[], id: string): Record<string, unknown> {
  const r = engines.find((e) => e.engine_id === id)?.result;
  return typeof r === 'object' && r !== null ? (r as Record<string, unknown>) : {};
}

function extractKeyFactsLine(engines: SelemeneEngineOutput[]): string {
  const facts: string[] = [];

  const panchanga = getResult(engines, 'panchanga');
  const vimshottari = getResult(engines, 'vimshottari');
  const humanDesign = getResult(engines, 'human-design');
  const geneKeys = getResult(engines, 'gene-keys');
  const biofield = getResult(engines, 'biofield');

  if (panchanga.nakshatra_name) facts.push(`nakshatra ${panchanga.nakshatra_name}`);
  if (panchanga.tithi_name) facts.push(`tithi ${panchanga.tithi_name}`);
  if (panchanga.vara_name) facts.push(`vara ${panchanga.vara_name}`);
  if (panchanga.yoga_name) facts.push(`yoga ${panchanga.yoga_name}`);

  const vcp = vimshottari.current_period as Record<string, unknown> | undefined;
  if (vcp?.mahadasha && typeof vcp.mahadasha === 'object' && vcp.mahadasha !== null) {
    const planet = (vcp.mahadasha as Record<string, unknown>).planet;
    if (planet) facts.push(`dasha ${planet}`);
  }
  if (vcp?.antardasha && typeof vcp.antardasha === 'object' && vcp.antardasha !== null) {
    const planet = (vcp.antardasha as Record<string, unknown>).planet;
    if (planet) facts.push(`antardasha ${planet}`);
  }

  if (humanDesign.hd_type) facts.push(`HD type ${humanDesign.hd_type}`);
  if (humanDesign.profile) facts.push(`profile ${humanDesign.profile}`);
  if (humanDesign.authority) facts.push(`authority ${humanDesign.authority}`);

  const hdChannels = humanDesign.active_channels;
  if (Array.isArray(hdChannels) && hdChannels.length > 0) {
    const channels = hdChannels
      .map((c) => (typeof c === 'string' ? c : (c as Record<string, unknown>)?.channel ?? (c as Record<string, unknown>)?.name))
      .filter(Boolean)
      .join(', ');
    facts.push(`active channels ${channels}`);
  }

  const gkKeys = geneKeys.active_keys;
  if (Array.isArray(gkKeys) && gkKeys.length > 0) {
    const keys = gkKeys
      .map((k) => (typeof k === 'string' ? k : (k as Record<string, unknown>)?.key ?? (k as Record<string, unknown>)?.name))
      .filter(Boolean)
      .join(', ');
    facts.push(`gene keys ${keys}`);
  }

  const bfAreas = biofield.areas_of_attention;
  if (Array.isArray(bfAreas) && bfAreas.length > 0) {
    const areas = bfAreas
      .map((a) => (typeof a === 'string' ? a : (a as Record<string, unknown>)?.area ?? (a as Record<string, unknown>)?.name))
      .filter(Boolean)
      .join(', ');
    facts.push(`biofield attention ${areas}`);
  }

  return facts.length > 0 ? facts.join('; ') + '.' : 'No key facts extracted.';
}

function jsonBlock(value: unknown): string {
  return '```json\n' + JSON.stringify(value, null, 2) + '\n```';
}
