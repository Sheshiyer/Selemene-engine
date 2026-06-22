import type { ParsedModeDoc, RegisterBand, SelemeneEngineOutput } from '../index.js';
export interface OrchestratorInput {
    subjectNames: string[];
    engineResultsBySubject: SelemeneEngineOutput[][];
    consciousnessLevel: number;
}
export interface PassResult {
    id: string;
    title: string;
    output: string;
}
export interface OrchestratorOutput {
    mode: string;
    subject_names: string[];
    register: RegisterBand;
    passes: PassResult[];
    assembled: string;
}
export interface LlmCall {
    (system: string, user: string, options: {
        max_tokens: number;
    }): Promise<string>;
}
export interface OrchestratorOptions {
    mode: ParsedModeDoc;
    llm: LlmCall;
}
export declare class IntegratedReadingOrchestrator {
    private mode;
    private llm;
    constructor(opts: OrchestratorOptions);
    run(input: OrchestratorInput): Promise<OrchestratorOutput>;
    private renderPassTemplate;
    private buildSystemPrompt;
    private buildOverlaySummary;
}
//# sourceMappingURL=integrated.d.ts.map