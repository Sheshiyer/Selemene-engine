// Minimal ambient typing for `pg` so db.ts compiles before `pnpm install`.
// Replaced by @types/pg once that lands in node_modules.

declare module 'pg' {
  export interface PoolConfig {
    connectionString?: string;
    max?: number;
    idleTimeoutMillis?: number;
    connectionTimeoutMillis?: number;
  }
  export interface QueryResult<T = unknown> {
    rows: T[];
    rowCount: number | null;
  }
  export class Pool {
    constructor(config?: PoolConfig);
    query<T = unknown>(text: string, values?: unknown[]): Promise<QueryResult<T>>;
    end(): Promise<void>;
  }
}

declare module 'server-only' {}
