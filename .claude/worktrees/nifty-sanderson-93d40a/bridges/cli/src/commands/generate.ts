import * as p from "@clack/prompts";
import chalk from "chalk";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { loadConfig, writeConfig } from "../core/config.js";
import { mergeSpecs } from "../generators/merge.js";
import { generateClaudeTools } from "../generators/claude.js";
import { generateOpenAIFunctions } from "../generators/openai.js";
import { generateLangChainTools } from "../generators/langchain.js";

export async function generateCommand(): Promise<void> {
  p.intro(chalk.bgCyan.black(" @selemene/bridge generate "));

  const config = loadConfig();
  if (!config) {
    p.log.error(
      "No .selemenerc.json found. Run `npx @selemene/bridge init` first."
    );
    process.exit(1);
  }

  const spin = p.spinner();
  spin.start("Fetching OpenAPI specs...");

  const mergeResult = await mergeSpecs({
    rustUrl: config.rustUrl,
    tsUrl: config.tsUrl,
    apiKey: config.apiKey,
  });

  if (mergeResult.totalPaths === 0) {
    spin.stop(chalk.red("No API endpoints found. Are the servers running?"));
    if (mergeResult.errors.length > 0) {
      for (const err of mergeResult.errors) {
        p.log.warn(err);
      }
    }
    process.exit(1);
  }

  spin.stop(
    chalk.green(
      `Fetched ${mergeResult.totalPaths} endpoints (${mergeResult.rustPathCount} Rust, ${mergeResult.tsPathCount} TS)`
    )
  );

  let totalFiles = 0;

  for (const framework of config.frameworks) {
    let result;
    switch (framework) {
      case "claude":
        result = generateClaudeTools(mergeResult.spec, config);
        break;
      case "openai":
        result = generateOpenAIFunctions(mergeResult.spec, config);
        break;
      case "langchain":
        result = generateLangChainTools(mergeResult.spec, config);
        break;
    }

    for (const file of result.files) {
      mkdirSync(dirname(file.path), { recursive: true });
      writeFileSync(file.path, file.content);
      totalFiles++;
    }

    p.log.success(`${framework}: ${result.summary}`);
  }

  // Update lastGenerated
  config.lastGenerated = new Date().toISOString();
  writeConfig(config);

  p.outro(
    chalk.green(`Regenerated ${totalFiles} file(s) from ${mergeResult.totalPaths} endpoints`)
  );
}
