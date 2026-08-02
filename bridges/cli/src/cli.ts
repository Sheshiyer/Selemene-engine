#!/usr/bin/env node
import { Command } from "commander";
import { initCommand } from "./commands/init.js";
import { generateCommand } from "./commands/generate.js";
import { checkCommand } from "./commands/check.js";
import { doctorCommand } from "./commands/doctor.js";

const program = new Command();

program
  .name("selemene-bridge")
  .description(
    "CLI tool for setting up Selemene Engine agent bridges — Claude, OpenAI, LangChain"
  )
  .version("3.3.1");

program
  .command("init")
  .description(
    "Interactive wizard to configure servers, select frameworks, and generate tool definitions"
  )
  .action(initCommand);

program
  .command("generate")
  .description(
    "Regenerate tool definitions from existing .selemenerc.json config"
  )
  .action(generateCommand);

program
  .command("check")
  .description("Quick connectivity check — ping both servers and report health")
  .action(checkCommand);

program
  .command("doctor")
  .description(
    "Full diagnostic — config, servers, OpenAPI specs, generated files"
  )
  .action(doctorCommand);

program.parse();
