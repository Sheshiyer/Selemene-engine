import type { SelemeneEngineOutput } from '../index.js';
export interface AuditInput {
    personId: string;
    readingMarkdown: string;
    engineResults: SelemeneEngineOutput[];
    deterministicOnly?: boolean;
}
export interface AuditResult {
    person_id: string;
    blockers: string[];
    warnings: string[];
    facts_count: number;
    passed: boolean;
}
export declare function runChainAudit(input: AuditInput): AuditResult;
//# sourceMappingURL=audit.d.ts.map