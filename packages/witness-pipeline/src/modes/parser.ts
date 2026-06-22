import yaml from 'js-yaml';
import type { ModeDocument, LessonEntry, PassSpec } from './types.js';

export function parseModeDocument(content: string, sourcePath: string): ModeDocument {
  const trimmed = content.trim();
  if (!trimmed.startsWith('---')) {
    throw new Error(`Missing frontmatter in ${sourcePath}`);
  }

  const end = trimmed.indexOf('---', 3);
  if (end === -1) {
    throw new Error(`Missing closing frontmatter delimiter in ${sourcePath}`);
  }

  const frontmatterText = trimmed.slice(3, end).trim();
  const bodyText = trimmed.slice(end + 3).trim();

  const frontmatter = yaml.load(frontmatterText) as Record<string, unknown>;
  if (!frontmatter || typeof frontmatter !== 'object') {
    throw new Error(`Malformed frontmatter YAML in ${sourcePath}`);
  }

  const templates: Record<string, string> = {};
  const sections = parseBodySections(bodyText);

  const requiredKeys = [
    'mode', 'subject_count', 'roles', 'target_words', 'architecture',
    'pass_plan', 'engine_overlay_weights', 'house_overlay', 'bridge_mandates', 'svg_topology',
  ];
  for (const key of requiredKeys) {
    if (!(key in frontmatter)) {
      throw new Error(`Missing required frontmatter key ${key} in ${sourcePath}`);
    }
  }

  const passPlan = (frontmatter.pass_plan as PassSpec[]).map((p) => {
    if (!p.id || !p.title || !p.template || typeof p.target_words !== 'number') {
      throw new Error(`Invalid pass spec in ${sourcePath}: ${JSON.stringify(p)}`);
    }
    if (!sections[p.template]) {
      throw new Error(`Template section ${p.template} not found in ${sourcePath}`);
    }
    return p;
  });

  for (const p of passPlan) {
    templates[p.template] = sections[p.template];
  }

  return {
    mode: String(frontmatter.mode),
    subject_count: frontmatter.subject_count as ModeDocument['subject_count'],
    roles: frontmatter.roles as string[],
    target_words: frontmatter.target_words as ModeDocument['target_words'],
    architecture: frontmatter.architecture as ModeDocument['architecture'],
    pass_plan: passPlan,
    engine_overlay_weights: frontmatter.engine_overlay_weights as Record<string, number>,
    house_overlay: frontmatter.house_overlay as number[],
    bridge_mandates: frontmatter.bridge_mandates as string[],
    svg_topology: frontmatter.svg_topology as ModeDocument['svg_topology'],
    register_variants: frontmatter.register_variants as ModeDocument['register_variants'],
    templates,
    overlay_rules: sections['overlay-rules'],
    glossary: sections['glossary'],
    interactions: sections['interactions'],
    lessons: parseLessons(sections['lessons'] ?? ''),
  };
}

function parseBodySections(body: string): Record<string, string> {
  const sections: Record<string, string> = {};
  const regex = /^##\s+(.+)$/gm;
  let match: RegExpExecArray | null;
  const matches: Array<{ name: string; start: number }> = [];
  while ((match = regex.exec(body)) !== null) {
    matches.push({ name: match[1].trim().toLowerCase().replace(/\s+/g, '-'), start: match.index });
  }
  for (let i = 0; i < matches.length; i++) {
    const current = matches[i];
    const next = matches[i + 1];
    const end = next ? next.start : body.length;
    const header = `## ${matches[i].name.replace(/-/g, ' ')}`;
    sections[current.name] = body.slice(current.start + header.length + 1, end).trim();
  }
  return sections;
}

function parseLessons(lessonsText: string): LessonEntry[] {
  const entries: LessonEntry[] = [];
  const headingRegex = /^###\s+(\d{4}-\d{2}-\d{2})\s*[-–—]\s*(.+)$/gm;
  let match: RegExpExecArray | null;
  const matches: Array<{ date: string; title: string; start: number }> = [];
  while ((match = headingRegex.exec(lessonsText)) !== null) {
    matches.push({ date: match[1], title: match[2].trim(), start: match.index });
  }
  for (let i = 0; i < matches.length; i++) {
    const current = matches[i];
    const next = matches[i + 1];
    const end = next ? next.start : lessonsText.length;
    const body = lessonsText.slice(current.start, end).trim();
    const fields: Record<string, string> = {};
    const fieldRegex = /^\*\*(.+?)\*\*:\s*(.+)$/gm;
    let fmatch: RegExpExecArray | null;
    while ((fmatch = fieldRegex.exec(body)) !== null) {
      fields[fmatch[1].toLowerCase().replace(/\s+/g, '_')] = fmatch[2].trim();
    }
    entries.push({ date: current.date, title: current.title, fields });
  }
  return entries;
}
