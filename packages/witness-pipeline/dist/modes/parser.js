// ─── Mode-doc parser ───────────────────────────────────────────────────
// Parses a reading-mode Markdown document into typed config + named body
// sections + structured lessons list.
//
// Frontmatter spec mirrors witness-agents/scripts/integratedreading/modes/_schema.md.
import { readFileSync } from 'node:fs';
import { load as loadYAML } from 'js-yaml';
const REQUIRED_KEYS = [
    'mode',
    'subject_count',
    'roles',
    'target_words',
    'architecture',
    'pass_plan',
    'engine_overlay_weights',
    'house_overlay',
    'bridge_mandates',
    'svg_topology',
];
const VALID_TOPOLOGIES = [
    'dyad-arc',
    'triad-triangle',
    'pentagon',
    'web-graph',
];
const VALID_ARCHITECTURES = ['linear', 'hierarchical'];
function assertModeConfig(fm, path) {
    if (!fm || typeof fm !== 'object') {
        throw new Error(`Mode doc ${path}: frontmatter missing or not an object`);
    }
    const obj = fm;
    for (const key of REQUIRED_KEYS) {
        if (!(key in obj)) {
            throw new Error(`Mode doc ${path}: missing required frontmatter key '${key}'`);
        }
    }
    if (!VALID_TOPOLOGIES.includes(obj.svg_topology)) {
        throw new Error(`Mode doc ${path}: invalid svg_topology '${obj.svg_topology}'. ` +
            `Valid: ${VALID_TOPOLOGIES.join(' | ')}`);
    }
    if (!VALID_ARCHITECTURES.includes(obj.architecture)) {
        throw new Error(`Mode doc ${path}: invalid architecture '${obj.architecture}'. ` +
            `Valid: ${VALID_ARCHITECTURES.join(' | ')}`);
    }
    if (!Array.isArray(obj.pass_plan) || obj.pass_plan.length === 0) {
        throw new Error(`Mode doc ${path}: pass_plan must be a non-empty array`);
    }
    for (const [i, p] of obj.pass_plan.entries()) {
        if (!p.id || !p.title || !p.template || typeof p.target_words !== 'number') {
            throw new Error(`Mode doc ${path}: pass_plan[${i}] missing one of {id, title, template, target_words}`);
        }
    }
    const subjectCount = obj.subject_count;
    if (typeof subjectCount.min !== 'number' || typeof subjectCount.max !== 'number') {
        throw new Error(`Mode doc ${path}: subject_count.{min,max} must be numbers`);
    }
    if (subjectCount.min > subjectCount.max) {
        throw new Error(`Mode doc ${path}: subject_count.min > subject_count.max`);
    }
}
function splitFrontmatter(raw, path) {
    if (!raw.startsWith('---')) {
        throw new Error(`Mode doc ${path}: missing leading '---' frontmatter delimiter`);
    }
    const closeIdx = raw.indexOf('\n---', 3);
    if (closeIdx === -1) {
        throw new Error(`Mode doc ${path}: missing closing '---' frontmatter delimiter`);
    }
    const yaml = raw.slice(3, closeIdx).trim();
    const bodyStart = raw.indexOf('\n', closeIdx + 4);
    const body = bodyStart === -1 ? '' : raw.slice(bodyStart + 1);
    return { yaml, body };
}
function splitSections(body) {
    const sections = {};
    const headerRe = /^## (.+?)\s*$/gm;
    const matches = [];
    let m;
    while ((m = headerRe.exec(body)) !== null) {
        const slug = m[1]
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, '-')
            .replace(/^-+|-+$/g, '');
        matches.push({ slug, index: m.index, headerEnd: m.index + m[0].length });
    }
    for (let i = 0; i < matches.length; i++) {
        const start = matches[i].headerEnd;
        const end = i + 1 < matches.length ? matches[i + 1].index : body.length;
        sections[matches[i].slug] = body.slice(start, end).trim();
    }
    return sections;
}
function parseLessons(lessonsBody) {
    if (!lessonsBody || !lessonsBody.trim())
        return [];
    const entries = [];
    const entryRe = /^### (\d{4}-\d{2}-\d{2})\s*[—–-]\s*(.+?)$/gm;
    const headers = [];
    let m;
    while ((m = entryRe.exec(lessonsBody)) !== null) {
        headers.push({
            date: m[1],
            title: m[2].trim(),
            index: m.index,
            headerEnd: m.index + m[0].length,
        });
    }
    for (let i = 0; i < headers.length; i++) {
        const body = lessonsBody.slice(headers[i].headerEnd, i + 1 < headers.length ? headers[i + 1].index : lessonsBody.length);
        const fields = {};
        for (const fieldName of ['Question', 'Variants', 'Winner', 'Adopted', 'Reference']) {
            const re = new RegExp(`\\*\\*${fieldName}:\\*\\*\\s*(.+?)(?=\\n\\*\\*|\\n\\n|$)`, 's');
            const fm = body.match(re);
            if (fm) {
                const value = fm[1].trim();
                if (fieldName === 'Variants') {
                    fields.variants = value
                        .split(/[,\/]\s*/)
                        .map((v) => v.trim())
                        .filter(Boolean);
                }
                else {
                    fields[fieldName.toLowerCase()] = value;
                }
            }
        }
        entries.push({ date: headers[i].date, title: headers[i].title, ...fields });
    }
    return entries;
}
function validateRegisterVariants(fm, sections, path) {
    if (!fm.register_variants)
        return;
    for (const band of ['l1_l3', 'l4_l5']) {
        const variant = fm.register_variants[band];
        if (!variant?.overrides)
            continue;
        for (const ov of variant.overrides) {
            const pass = fm.pass_plan.find((p) => p.id === ov.pass_id);
            if (!pass) {
                throw new Error(`Mode doc ${path}: register_variants.${band}.overrides[].pass_id '${ov.pass_id}' has no matching pass in pass_plan`);
            }
            if (!sections[ov.template]) {
                throw new Error(`Mode doc ${path}: register_variants.${band}.overrides for pass '${ov.pass_id}' references template '${ov.template}' which has no '## ${ov.template}' section`);
            }
        }
    }
}
/** Parse a mode-doc Markdown file at `path`. Throws on missing required fields. */
export function parseModeDoc(path) {
    const raw = readFileSync(path, 'utf-8');
    const { yaml, body } = splitFrontmatter(raw, path);
    let frontmatter;
    try {
        frontmatter = loadYAML(yaml);
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        throw new Error(`Mode doc ${path}: malformed YAML frontmatter — ${message}`);
    }
    assertModeConfig(frontmatter, path);
    const sections = splitSections(body);
    const lessons = parseLessons(sections.lessons);
    for (const pass of frontmatter.pass_plan) {
        if (!sections[pass.template]) {
            throw new Error(`Mode doc ${path}: pass_plan[${pass.id}].template '${pass.template}' has no matching '## ${pass.template}' section`);
        }
    }
    validateRegisterVariants(frontmatter, sections, path);
    return {
        frontmatter,
        sections,
        lessons,
        raw_path: path,
    };
}
export function parseModeDocument(content, sourcePath) {
    const { yaml, body } = splitFrontmatter(content, sourcePath);
    let frontmatter;
    try {
        frontmatter = loadYAML(yaml);
    }
    catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        throw new Error(`Mode doc ${sourcePath}: malformed YAML frontmatter — ${message}`);
    }
    assertModeConfig(frontmatter, sourcePath);
    const sections = splitSections(body);
    const lessons = parseLessons(sections.lessons);
    for (const pass of frontmatter.pass_plan) {
        if (!sections[pass.template]) {
            throw new Error(`Mode doc ${sourcePath}: pass_plan[${pass.id}].template '${pass.template}' has no matching '## ${pass.template}' section`);
        }
    }
    validateRegisterVariants(frontmatter, sections, sourcePath);
    return { frontmatter, sections, lessons, raw_path: sourcePath };
}
export function summarizeLessons(lessons, maxEntries = 5) {
    if (lessons.length === 0)
        return '';
    const recent = lessons.slice(-maxEntries);
    const lines = recent.map((l) => {
        const adopted = l.adopted ? ` — Adopted: ${l.adopted}` : '';
        return `• ${l.date} ${l.title}${adopted}`;
    });
    return `## Prior Autoresearch Findings\n\n${lines.join('\n')}`;
}
export function getPassTemplate(doc, pass_id, register) {
    const pass = doc.frontmatter.pass_plan.find((p) => p.id === pass_id);
    if (!pass) {
        throw new Error(`No pass with id '${pass_id}' in mode '${doc.frontmatter.mode}'`);
    }
    const variant = doc.frontmatter.register_variants?.[register];
    const override = variant?.overrides?.find((o) => o.pass_id === pass_id);
    const templateName = override?.template ?? pass.template;
    const content = doc.sections[templateName];
    if (content === undefined) {
        throw new Error(`Pass '${pass_id}' (register ${register}) resolves to template '${templateName}' which has no matching '## ${templateName}' section in mode '${doc.frontmatter.mode}'`);
    }
    return content;
}
export function getTargetWordsForRegister(doc, register) {
    const variant = doc.frontmatter.register_variants?.[register];
    return variant?.target_words ?? doc.frontmatter.target_words;
}
//# sourceMappingURL=parser.js.map