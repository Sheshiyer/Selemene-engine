import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";
import { SelemeneConfigSchema, type SelemeneConfig } from "./types.js";

const CONFIG_FILENAME = ".selemenerc.json";

export function getConfigPath(dir: string = process.cwd()): string {
  return resolve(dir, CONFIG_FILENAME);
}

export function loadConfig(dir?: string): SelemeneConfig | null {
  const configPath = getConfigPath(dir);
  if (!existsSync(configPath)) {
    return null;
  }

  const raw = JSON.parse(readFileSync(configPath, "utf-8"));
  const result = SelemeneConfigSchema.safeParse(raw);

  if (!result.success) {
    throw new Error(
      `Invalid config at ${configPath}: ${result.error.message}`
    );
  }

  return result.data;
}

export function writeConfig(config: SelemeneConfig, dir?: string): string {
  const configPath = getConfigPath(dir);
  const validated = SelemeneConfigSchema.parse(config);
  writeFileSync(configPath, JSON.stringify(validated, null, 2) + "\n");
  return configPath;
}
