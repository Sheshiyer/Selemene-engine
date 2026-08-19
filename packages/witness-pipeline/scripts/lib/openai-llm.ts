import { execSync } from 'node:child_process';

export interface OpenAIOptions {
  model?: string;
  temperature?: number;
}

function loadOpenAIKey(): string {
  const key = process.env.OPENAI_API_KEY;
  if (key) return key;
  try {
    const home = process.env.HOME || '/Users/sheshnarayaniyer';
    const envContent = require('node:fs').readFileSync(`${home}/.claude/.env`, 'utf8');
    const match = envContent.match(/OPENAI_API_KEY=(.+)/);
    if (match) return match[1].trim();
  } catch {}
  throw new Error('OPENAI_API_KEY not found in env or ~/.claude/.env');
}

export function callOpenAI(
  system: string,
  user: string,
  options: { max_tokens: number; model?: string; temperature?: number },
): Promise<string> {
  const apiKey = loadOpenAIKey();
  const model = options.model ?? 'gpt-4o-mini';
  const temperature = options.temperature ?? 0.7;

  const body = JSON.stringify({
    model,
    messages: [
      { role: 'system', content: system },
      { role: 'user', content: user },
    ],
    max_tokens: Math.min(options.max_tokens, 16384),
    temperature,
  });

  return new Promise((resolve, reject) => {
    const https = require('node:https');
    const req = https.request(
      {
        hostname: 'api.openai.com',
        port: 443,
        path: '/v1/chat/completions',
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiKey}`,
        },
        timeout: 120000,
      },
      (res: any) => {
        let data = '';
        res.on('data', (chunk: string) => (data += chunk));
        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);
            if (parsed.error) {
              reject(new Error(`OpenAI API error: ${parsed.error.message}`));
              return;
            }
            const content = parsed.choices?.[0]?.message?.content;
            if (!content) {
              reject(new Error('No content in OpenAI response'));
              return;
            }
            resolve(content);
          } catch (e) {
            reject(e);
          }
        });
        res.on('error', (e: Error) => reject(e));
      },
    );
    req.on('error', (e: Error) => reject(e));
    req.write(body);
    req.end();
  });
}