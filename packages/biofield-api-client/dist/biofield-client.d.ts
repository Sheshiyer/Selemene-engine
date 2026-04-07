import type { BiofieldCaptureResult, BiofieldReadingDetail, BiofieldReadingSummary, BiofieldSession, CloseBiofieldSessionRequest, CreateBiofieldSessionRequest } from "@selemene/biofield-domain";
export interface BiofieldClientOptions {
    authToken?: string;
    fetchImpl?: typeof fetch;
}
export interface ListBiofieldReadingsParams {
    limit?: number;
    offset?: number;
}
export declare class BiofieldClientError extends Error {
    readonly status: number;
    readonly details?: unknown;
    constructor(message: string, status: number, details?: unknown);
}
export declare class BiofieldClient {
    private readonly baseUrl;
    private readonly authToken?;
    private readonly fetchImpl;
    constructor(baseUrl: string, options?: BiofieldClientOptions);
    createSession(input?: CreateBiofieldSessionRequest): Promise<BiofieldSession>;
    closeSession(sessionId: string, input?: CloseBiofieldSessionRequest): Promise<BiofieldSession>;
    getSession(sessionId: string): Promise<BiofieldSession>;
    listReadings(params?: ListBiofieldReadingsParams): Promise<BiofieldReadingSummary[]>;
    getReading(readingId: string): Promise<BiofieldReadingDetail>;
    uploadCapture(sessionId: string, payload: FormData): Promise<BiofieldCaptureResult>;
    private request;
}
//# sourceMappingURL=biofield-client.d.ts.map