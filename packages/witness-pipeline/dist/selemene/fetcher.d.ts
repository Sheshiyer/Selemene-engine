import { type SelemeneEngineId, type SelemeneEngineOutput, type BirthData } from './types.js';
export { SELEMENE_ENGINE_IDS, type SelemeneEngineId, type SelemeneEngineOutput, type BirthData } from './types.js';
export declare const SELEMENE_BASE_URL = "https://selemene.tryambakam.space";
export interface FetchOptions {
    api_key: string;
    base_url?: string;
    timeout_ms?: number;
    engines?: SelemeneEngineId[];
    fetchImpl?: typeof fetch;
}
export declare function fetchAllEngines(birthData: BirthData, opts: FetchOptions): Promise<SelemeneEngineOutput[]>;
export declare function loadSelemeneKey(): Promise<string | undefined>;
//# sourceMappingURL=fetcher.d.ts.map