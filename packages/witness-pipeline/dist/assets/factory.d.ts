import type { SelemeneEngineOutput } from '../index.js';
export interface SourcePackInput {
    personId: string;
    readingMarkdown: string;
    engineResults: SelemeneEngineOutput[];
    outputDir: string;
    deterministicOnly?: boolean;
}
export interface SourcePack {
    outputDir: string;
    manifest: {
        person_id: string;
        created_at: string;
        reading_length: number;
        engines: string[];
        quality: {
            facts_count: number;
            gate_status: string;
        };
    };
    reflectionQuestions: string[];
    paths: {
        manifest: string;
        reading: string;
        questions: string;
    };
}
export declare function createSourcePack(input: SourcePackInput): Promise<SourcePack>;
//# sourceMappingURL=factory.d.ts.map