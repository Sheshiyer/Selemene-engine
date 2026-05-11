// Minimal ambient typing for `@aws-sdk/client-s3` so r2.ts compiles before
// `pnpm install -D @aws-sdk/client-s3` is run. The script is the only consumer
// at runtime; in the noesis-web Next.js bundle this code is not reached.

declare module '@aws-sdk/client-s3' {
  export interface S3ClientConfig {
    region: string;
    endpoint: string;
    credentials: { accessKeyId: string; secretAccessKey: string };
  }
  export interface PutObjectInput {
    Bucket: string;
    Key: string;
    Body: Buffer;
    ContentType?: string;
    CacheControl?: string;
  }
  export class PutObjectCommand {
    constructor(input: PutObjectInput);
  }
  export class S3Client {
    constructor(config: S3ClientConfig);
    send(command: unknown): Promise<unknown>;
  }
}
