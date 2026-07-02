export interface R2Config {
  accountId: string;
  accessKeyId: string;
  secretAccessKey: string;
  bucket: string;
  publicBaseUrl: string;
}

export function requireR2Config(env: NodeJS.ProcessEnv = process.env): R2Config {
  const config = {
    accountId: env.R2_ACCOUNT_ID ?? "",
    accessKeyId: env.R2_ACCESS_KEY_ID ?? "",
    secretAccessKey: env.R2_SECRET_ACCESS_KEY ?? "",
    bucket: env.R2_RAGA_CLIPS_BUCKET ?? "selemene-raga-clips",
    publicBaseUrl: env.R2_PUBLIC_BASE_URL ?? "",
  };

  const missing = [
    !config.accountId && "R2_ACCOUNT_ID",
    !config.accessKeyId && "R2_ACCESS_KEY_ID",
    !config.secretAccessKey && "R2_SECRET_ACCESS_KEY",
    !config.publicBaseUrl && "R2_PUBLIC_BASE_URL",
  ].filter(Boolean);

  if (missing.length) {
    throw new Error(`Missing R2 env vars: ${missing.join(", ")}`);
  }

  return config;
}

export function publicR2Url(publicBaseUrl: string, key: string): string {
  return `${publicBaseUrl.replace(/\/+$/, "")}/${key.replace(/^\/+/, "")}`;
}

export async function uploadToR2(key: string, buffer: Buffer, config = requireR2Config()): Promise<string> {
  const endpoint = `https://${config.accountId}.r2.cloudflarestorage.com/${config.bucket}/${key}`;
  const response = await fetch(endpoint, {
    method: "PUT",
    headers: {
      "Content-Type": "audio/mpeg",
      "Cache-Control": "public, max-age=31536000, immutable",
      "Authorization": `Bearer ${config.secretAccessKey}`,
      "X-Access-Key-Id": config.accessKeyId,
    },
    body: buffer,
  });

  if (!response.ok) {
    throw new Error(`R2 upload failed: ${response.status} ${await response.text()}`);
  }

  return publicR2Url(config.publicBaseUrl, key);
}
