import * as p from "@clack/prompts";
import chalk from "chalk";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { checkHealth } from "../core/http.js";
import { writeConfig } from "../core/config.js";
import type { SelemeneConfig } from "../core/types.js";
import { mergeSpecs } from "../generators/merge.js";
import { generateClaudeTools } from "../generators/claude.js";
import { generateOpenAIFunctions } from "../generators/openai.js";
import { generateLangChainTools } from "../generators/langchain.js";

export async function initCommand(): Promise<void> {
  p.intro(chalk.bgCyan.black(" @selemene/bridge "));

  console.log(
    chalk.dim(
      "  Set up Selemene Engine agent bridges for Claude, OpenAI, or LangChain\n"
    )
  );

  // 1. Rust URL
  const rustUrl = await p.text({
    message: "Selemene Rust engine URL",
    placeholder: "http://localhost:8080",
    defaultValue: "http://localhost:8080",
    validate: (v) => {
      try {
        new URL(v);
      } catch {
        return "Please enter a valid URL";
      }
    },
  });
  if (p.isCancel(rustUrl)) return handleCancel();

  // 2. TS URL
  const tsUrl = await p.text({
    message: "Selemene TypeScript engines URL",
    placeholder: "http://localhost:3001",
    defaultValue: "http://localhost:3001",
    validate: (v) => {
      try {
        new URL(v);
      } catch {
        return "Please enter a valid URL";
      }
    },
  });
  if (p.isCancel(tsUrl)) return handleCancel();

  // 3. API Key
  const apiKey = await p.password({
    message: "API key (optional, press Enter to skip)",
  });
  if (p.isCancel(apiKey)) return handleCancel();

  // 4. Validate connectivity
  const spin = p.spinner();
  spin.start("Checking server connectivity...");

  const [rustHealth, tsHealth] = await Promise.all([
    checkHealth(`${rustUrl}/health/live`),
    checkHealth(`${tsUrl}/health`),
  ]);

  if (rustHealth.ok && tsHealth.ok) {
    spin.stop(chalk.green("Both servers are reachable"));
  } else if (rustHealth.ok || tsHealth.ok) {
    const messages: string[] = [];
    if (!rustHealth.ok)
      messages.push(`Rust server: ${rustHealth.error ?? "unreachable"}`);
    if (!tsHealth.ok)
      messages.push(`TS server: ${tsHealth.error ?? "unreachable"}`);
    spin.stop(
      chalk.yellow(`Partial connectivity — ${messages.join(", ")}`)
    );
    p.log.warn(
      "Some servers are unavailable. Generated tools may be incomplete."
    );
  } else {
    spin.stop(chalk.yellow("Neither server is reachable"));
    p.log.warn(
      "Servers are offline. You can still configure and generate later with `selemene-bridge generate`."
    );
  }

  // 5. Framework selection
  const frameworks = await p.multiselect({
    message: "Which frameworks do you need tool definitions for?",
    options: [
      { value: "claude", label: "Claude (Anthropic)", hint: "tools.json" },
      {
        value: "openai",
        label: "OpenAI",
        hint: "function-calling functions.json",
      },
      {
        value: "langchain",
        label: "LangChain / CrewAI",
        hint: "Python StructuredTool",
      },
    ],
    required: true,
  });
  if (p.isCancel(frameworks)) return handleCancel();

  // 6. Output directory
  const outputDir = await p.text({
    message: "Output directory for generated files",
    placeholder: "./selemene-tools",
    defaultValue: "./selemene-tools",
  });
  if (p.isCancel(outputDir)) return handleCancel();

  // 7. Fetch + merge + generate
  spin.start("Fetching OpenAPI specs and generating tool definitions...");

  const mergeResult = await mergeSpecs({
    rustUrl: rustUrl as string,
    tsUrl: tsUrl as string,
    apiKey: apiKey || undefined,
  });

  const config: SelemeneConfig = {
    version: "1.0",
    rustUrl: rustUrl as string,
    tsUrl: tsUrl as string,
    apiKey: apiKey || undefined,
    frameworks: frameworks as Array<"claude" | "openai" | "langchain">,
    outputDir: outputDir as string,
    lastGenerated: new Date().toISOString(),
  };

  let totalFiles = 0;
  const summaries: string[] = [];

  if (mergeResult.totalPaths === 0 && mergeResult.errors.length > 0) {
    spin.stop(
      chalk.yellow(
        "No OpenAPI specs could be fetched. Config saved — run `generate` when servers are up."
      )
    );
  } else {
    for (const framework of frameworks as string[]) {
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
        default:
          continue;
      }

      for (const file of result.files) {
        mkdirSync(dirname(file.path), { recursive: true });
        writeFileSync(file.path, file.content);
        totalFiles++;
      }
      summaries.push(result.summary);
    }

    spin.stop(
      chalk.green(
        `Generated ${totalFiles} file(s) from ${mergeResult.totalPaths} API endpoints`
      )
    );
  }

  // 8. Write config
  const configPath = writeConfig(config);

  // 9. Summary
  p.log.success(`Config written to ${chalk.cyan(configPath)}`);

  if (summaries.length > 0) {
    p.log.info(summaries.join("\n  "));
  }

  if (mergeResult.errors.length > 0) {
    p.log.warn(`Warnings:\n  ${mergeResult.errors.join("\n  ")}`);
  }

  // 10. Next steps
  p.note(getNextSteps(frameworks as string[], outputDir as string), "Next steps");
  p.outro(chalk.green("Setup complete!"));
}

function handleCancel(): void {
  p.cancel("Setup cancelled.");
  process.exit(0);
}

function getNextSteps(frameworks: string[], outputDir: string): string {
  const lines: string[] = [];

  if (frameworks.includes("claude")) {
    lines.push(
      chalk.bold("Claude:"),
      `  Load ${outputDir}/claude/tools.json into your Claude API call:`,
      chalk.dim(
        '  const tools = JSON.parse(fs.readFileSync("./selemene-tools/claude/tools.json"))'
      ),
      ""
    );
  }

  if (frameworks.includes("openai")) {
    lines.push(
      chalk.bold("OpenAI:"),
      `  Use ${outputDir}/openai/functions.json as the tools parameter:`,
      chalk.dim(
        '  const tools = JSON.parse(fs.readFileSync("./selemene-tools/openai/functions.json"))'
      ),
      ""
    );
  }

  if (frameworks.includes("langchain")) {
    lines.push(
      chalk.bold("LangChain:"),
      `  Import from ${outputDir}/langchain/selemene_tools.py:`,
      chalk.dim(
        "  from selemene_tools import get_all_tools"
      ),
      ""
    );
  }

  lines.push(
    chalk.dim("Regenerate anytime: npx @selemene/bridge generate"),
    chalk.dim("Health check:       npx @selemene/bridge check"),
    chalk.dim("Full diagnostic:    npx @selemene/bridge doctor")
  );

  return lines.join("\n");
}
