import * as p from "@clack/prompts";
import chalk from "chalk";
import { loadConfig } from "../core/config.js";
import { checkHealth, fetchJSON } from "../core/http.js";
import type { OpenAPISpec } from "../core/types.js";

export async function checkCommand(): Promise<void> {
  p.intro(chalk.bgCyan.black(" @selemene/bridge check "));

  const config = loadConfig();
  if (!config) {
    p.log.error(
      "No .selemenerc.json found. Run `npx @selemene/bridge init` first."
    );
    process.exit(1);
  }

  // Check Rust server
  const rustHealth = await checkHealth(`${config.rustUrl}/health/live`);
  if (rustHealth.ok) {
    let endpointCount = "?";
    try {
      const spec = await fetchJSON<OpenAPISpec>(
        `${config.rustUrl}/api/openapi.json`,
        { apiKey: config.apiKey }
      );
      endpointCount = String(Object.keys(spec.paths ?? {}).length);
    } catch {
      // Ignore — we still know the server is up
    }
    p.log.success(
      `Rust server  ${chalk.green("UP")}  ${chalk.dim(config.rustUrl)}  ${chalk.cyan(endpointCount + " endpoints")}`
    );
  } else {
    p.log.error(
      `Rust server  ${chalk.red("DOWN")}  ${chalk.dim(config.rustUrl)}  ${chalk.dim(rustHealth.error ?? "")}`
    );
  }

  // Check TS server
  const tsHealth = await checkHealth(`${config.tsUrl}/health`);
  if (tsHealth.ok) {
    let endpointCount = "?";
    try {
      const spec = await fetchJSON<OpenAPISpec>(`${config.tsUrl}/docs/json`, {
        apiKey: config.apiKey,
      });
      endpointCount = String(Object.keys(spec.paths ?? {}).length);
    } catch {
      // Ignore
    }
    p.log.success(
      `TS server    ${chalk.green("UP")}  ${chalk.dim(config.tsUrl)}  ${chalk.cyan(endpointCount + " endpoints")}`
    );
  } else {
    p.log.error(
      `TS server    ${chalk.red("DOWN")}  ${chalk.dim(config.tsUrl)}  ${chalk.dim(tsHealth.error ?? "")}`
    );
  }

  // Summary
  const bothUp = rustHealth.ok && tsHealth.ok;
  const oneUp = rustHealth.ok || tsHealth.ok;

  if (bothUp) {
    p.outro(chalk.green("All servers healthy"));
  } else if (oneUp) {
    p.outro(chalk.yellow("Partial connectivity — some tools may be unavailable"));
  } else {
    p.outro(chalk.red("No servers reachable"));
  }
}
