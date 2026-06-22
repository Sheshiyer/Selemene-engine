// ─── Integrated Reading Orchestrator ─────────────────────────────────
// Multi-pass reading orchestrator driven by a parsed mode document.
import { getPassTemplate, summarizeLessons } from '../modes/parser.js';
function resolveRegister(level) {
    return level <= 3 ? 'l1_l3' : 'l4_l5';
}
function resolveTargetWords(doc, register, passId) {
    const variant = doc.frontmatter.register_variants?.[register];
    const base = variant?.target_words ?? doc.frontmatter.target_words;
    if (!passId)
        return base;
    const pass = doc.frontmatter.pass_plan.find((p) => p.id === passId);
    if (!pass)
        return base;
    // If the variant overrides this pass, use a tight range around the pass target.
    const override = variant?.overrides?.find((o) => o.pass_id === passId);
    if (override) {
        return { min: Math.round(pass.target_words * 0.9), max: pass.target_words };
    }
    return { min: Math.round(pass.target_words * 0.9), max: pass.target_words };
}
export class IntegratedReadingOrchestrator {
    mode;
    llm;
    constructor(opts) {
        this.mode = opts.mode;
        this.llm = opts.llm;
    }
    async run(input) {
        const register = resolveRegister(input.consciousnessLevel);
        const passOutputs = [];
        let assembled = '';
        for (const pass of this.mode.frontmatter.pass_plan) {
            const prior = assembled.slice(-4000);
            const templateContent = getPassTemplate(this.mode, pass.id, register);
            const prompt = this.renderPassTemplate(templateContent, pass, input, prior, register);
            const system = this.buildSystemPrompt(pass, input, register);
            const { max } = resolveTargetWords(this.mode, register, pass.id);
            const output = await this.llm(system, prompt, { max_tokens: Math.round(max * 2) });
            passOutputs.push({ id: pass.id, title: pass.title, output });
            assembled += `\n\n## ${pass.title}\n\n${output}`;
        }
        return {
            mode: this.mode.frontmatter.mode,
            subject_names: input.subjectNames,
            register,
            passes: passOutputs,
            assembled: assembled.trim(),
        };
    }
    renderPassTemplate(template, pass, input, priorPass, register) {
        const overlaySummary = this.buildOverlaySummary();
        const bridgeMandates = this.mode.frontmatter.bridge_mandates.map((m) => `- ${m}`).join('\n');
        const lessonsSummary = summarizeLessons(this.mode.lessons, 5);
        return template
            .replace(/\{\{subject_names\}\}/g, input.subjectNames.join(', '))
            .replace(/\{\{prior_pass\}\}/g, priorPass)
            .replace(/\{\{overlay_summary\}\}/g, overlaySummary)
            .replace(/\{\{bridge_mandates\}\}/g, bridgeMandates)
            .replace(/\{\{lessons_summary\}\}/g, lessonsSummary)
            .replace(/\{\{register\}\}/g, register)
            .replace(/\{\{pass_id\}\}/g, pass.id)
            .replace(/\{\{target_words\}\}/g, String(pass.target_words));
    }
    buildSystemPrompt(pass, input, register) {
        const { min, max } = resolveTargetWords(this.mode, register, pass.id);
        return `You are writing pass "${pass.title}" (id: ${pass.id}) for the ${this.mode.frontmatter.mode} reading mode.
Register band: ${register}.
Target length: ~${pass.target_words} words (acceptable range ${min}-${max}).
Subjects: ${input.subjectNames.join(', ')}.
${this.mode.sections['overlay-rules'] ?? ''}`;
    }
    buildOverlaySummary() {
        const weights = Object.entries(this.mode.frontmatter.engine_overlay_weights)
            .map(([k, v]) => `${k}: ${v}`)
            .join(', ');
        return `Engine weights: ${weights}; Houses: ${this.mode.frontmatter.house_overlay.join(', ')}`;
    }
}
//# sourceMappingURL=integrated.js.map