import * as p from "@clack/prompts";
import chalk from "chalk";
import { existsSync, statSync } from "node:fs";
import { loadConfig, getConfigPath } from "../core/config.js";
import { checkHealth, fetchJSON } from "../core/http.js";
import type { OpenAPISpec } from "../core/types.js";

interface DiagRow {
  check: string;
  status: "pass" | "warn" | "fail";
  detail: string;
}

function statusIcon(s: DiagRow["status"]): string {
  switch (s) {
    case "pass":
      return chalk.green("PASS");
    case "warn":
      return chalk.yellow("WARN");
    case "fail":
      return chalk.red("FAIL");
  }
}

export async function doctorCommand(): Promise<void> {
  p.intro(chalk.bgCyan.black(" @selemene/bridge doctor "));

  const rows: DiagRow[] = [];

  // 1. Config file
  const configPath = getConfigPath();
  const config = loadConfig();

  if (config) {
    rows.push({
      check: "Config file",
      status: "pass",
      detail: configPath,
    });
  } else {
    rows.push({
      check: "Config file",
      status: "fail",
      detail: `Not found at ${configPath} — run 'init' first`,
    });
    printDiag(rows);
    p.outro(chalk.red("Cannot continue without config."));
    return;
  }

  // 2. Config validation
  rows.push({
    check: "Config valid",
    status: "pass",
    detail: `v${config.version}, ${config.frameworks.length} framework(s)`,
  });

  // 3. Rust server
  const spin = p.spinner();
  spin.start("Running diagnostics...");

  const rustHealth = await checkHealth(`${config.rustUrl}/health/live`);
  if (rustHealth.ok) {
    rows.push({ check: "Rust server", status: "pass", detail: config.rustUrl });

    // 4. Rust OpenAPI
    try {
      const spec = await fetchJSON<OpenAPISpec>(
        `${config.rustUrl}/api/openapi.json`,
        { apiKey: config.apiKey }
      );
      const count = Object.keys(spec.paths ?? {}).length;
      rows.push({
        check: "Rust OpenAPI",
        status: "pass",
        detail: `${count} endpoints`,
      });
    } catch (err) {
      rows.push({
        check: "Rust OpenAPI",
        status: "warn",
        detail: `Server up but /api/openapi.json failed: ${err}`,
      });
    }
  } else {
    rows.push({
      check: "Rust server",
      status: "fail",
      detail: `${config.rustUrl} — ${rustHealth.error ?? "unreachable"}`,
    });
    rows.push({
      check: "Rust OpenAPI",
      status: "fail",
      detail: "Server unreachable",
    });
  }

  // 5. TS server
  const tsHealth = await checkHealth(`${config.tsUrl}/health`);
  if (tsHealth.ok) {
    rows.push({ check: "TS server", status: "pass", detail: config.tsUrl });

    // 6. TS OpenAPI
    try {
      const spec = await fetchJSON<OpenAPISpec>(`${config.tsUrl}/docs/json`, {
        apiKey: config.apiKey,
      });
      const count = Object.keys(spec.paths ?? {}).length;
      rows.push({
        check: "TS OpenAPI",
        status: "pass",
        detail: `${count} endpoints`,
      });
    } catch (err) {
      rows.push({
        check: "TS OpenAPI",
        status: "warn",
        detail: `Server up but /docs/json failed: ${err}`,
      });
    }
  } else {
    rows.push({
      check: "TS server",
      status: "fail",
      detail: `${config.tsUrl} — ${tsHealth.error ?? "unreachable"}`,
    });
    rows.push({
      check: "TS OpenAPI",
      status: "fail",
      detail: "Server unreachable",
    });
  }

  // 7. Generated files
  const fileChecks: Array<{ framework: string; path: string }> = [];
  if (config.frameworks.includes("claude")) {
    fileChecks.push({
      framework: "Claude tools",
      path: `${config.outputDir}/claude/tools.json`,
    });
  }
  if (config.frameworks.includes("openai")) {
    fileChecks.push({
      framework: "OpenAI functions",
      path: `${config.outputDir}/openai/functions.json`,
    });
  }
  if (config.frameworks.includes("langchain")) {
    fileChecks.push({
      framework: "LangChain tools",
      path: `${config.outputDir}/langchain/selemene_tools.py`,
    });
  }

  for (const fc of fileChecks) {
    if (existsSync(fc.path)) {
      const stats = statSync(fc.path);
      const sizeKb = (stats.size / 1024).toFixed(1);
      rows.push({
        check: fc.framework,
        status: "pass",
        detail: `${fc.path} (${sizeKb} KB)`,
      });
    } else {
      rows.push({
        check: fc.framework,
        status: "warn",
        detail: `${fc.path} not found — run 'generate'`,
      });
    }
  }

  // 8. Last generated
  if (config.lastGenerated) {
    const ago = timeSince(new Date(config.lastGenerated));
    rows.push({
      check: "Last generated",
      status: "pass",
      detail: `${ago} ago`,
    });
  } else {
    rows.push({
      check: "Last generated",
      status: "warn",
      detail: "Never — run 'generate'",
    });
  }

  spin.stop("Diagnostics complete");
  printDiag(rows);

  const failCount = rows.filter((r) => r.status === "fail").length;
  const warnCount = rows.filter((r) => r.status === "warn").length;

  if (failCount > 0) {
    p.outro(
      chalk.red(`${failCount} issue(s) found, ${warnCount} warning(s)`)
    );
  } else if (warnCount > 0) {
    p.outro(chalk.yellow(`All good with ${warnCount} warning(s)`));
  } else {
    p.outro(chalk.green("Everything looks healthy!"));
  }
}

function printDiag(rows: DiagRow[]): void {
  console.log();
  for (const row of rows) {
    const icon = statusIcon(row.status);
    console.log(
      `  ${icon}  ${chalk.bold(row.check.padEnd(16))} ${chalk.dim(row.detail)}`
    );
  }
  console.log();
}

function timeSince(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  return `${days}d`;
}
