# @selemene/bridge

> CLI tool for setting up Selemene Engine agent bridges — Claude, OpenAI, LangChain.

One command to generate tool definitions for any AI framework from a running Selemene Engine instance.

## Quick Start

```bash
npx @selemene/bridge init
```

The interactive wizard will:
1. Ask for your Selemene server URLs
2. Validate connectivity
3. Let you pick frameworks (Claude, OpenAI, LangChain)
4. Fetch OpenAPI specs and generate tool definitions
5. Write a `.selemenerc.json` config for future use

## Commands

| Command | Description |
|---------|-------------|
| `npx @selemene/bridge init` | Interactive setup wizard |
| `npx @selemene/bridge generate` | Regenerate tools from existing config |
| `npx @selemene/bridge check` | Quick server connectivity check |
| `npx @selemene/bridge doctor` | Full diagnostic (config, servers, specs, files) |

## Framework Output

### Claude (Anthropic)

Generates `tools.json` — an array of Claude-compatible tool definitions:

```json
[
  {
    "name": "engines_panchanga_calculate",
    "description": "Calculate panchanga for a given date and location",
    "input_schema": {
      "type": "object",
      "properties": { ... },
      "required": ["date", "latitude", "longitude"]
    }
  }
]
```

**Usage:**
```typescript
import { readFileSync } from "fs";
import Anthropic from "@anthropic-ai/sdk";

const tools = JSON.parse(readFileSync("./selemene-tools/claude/tools.json", "utf-8"));
const client = new Anthropic();

const response = await client.messages.create({
  model: "claude-sonnet-4-5-20250929",
  max_tokens: 1024,
  tools,
  messages: [{ role: "user", content: "What's the panchanga for today in Mumbai?" }],
});
```

### OpenAI

Generates `functions.json` — OpenAI function-calling format:

```json
[
  {
    "type": "function",
    "function": {
      "name": "engines_panchanga_calculate",
      "description": "Calculate panchanga for a given date and location",
      "parameters": { "type": "object", "properties": { ... } }
    }
  }
]
```

**Usage:**
```typescript
import OpenAI from "openai";
import { readFileSync } from "fs";

const tools = JSON.parse(readFileSync("./selemene-tools/openai/functions.json", "utf-8"));
const client = new OpenAI();

const response = await client.chat.completions.create({
  model: "gpt-4o",
  tools,
  messages: [{ role: "user", content: "Calculate my biorhythm for today" }],
});
```

### LangChain / CrewAI

Generates `selemene_tools.py` — Python module with Pydantic models and StructuredTool instances:

```python
from selemene_tools import get_all_tools

tools = get_all_tools()

# Use with LangChain agent
from langchain.agents import initialize_agent
agent = initialize_agent(tools, llm, agent="structured-chat-zero-shot-react-description")

# Use with CrewAI
from crewai import Agent
agent = Agent(role="Vedic Analyst", tools=tools)
```

## Engines

The bridge covers all 14 Selemene consciousness engines:

| # | Engine | Source | Description |
|---|--------|--------|-------------|
| 1 | Panchanga | Rust | Vedic calendar (tithi, nakshatra, yoga, karana) |
| 2 | Numerology | Rust | Pythagorean + Chaldean number analysis |
| 3 | Biorhythm | Rust | Physical, emotional, intellectual cycles |
| 4 | Human Design | Rust | Ra Uru Hu's system — type, authority, profile |
| 5 | Gene Keys | Rust | Richard Rudd's 64 keys — shadow, gift, siddhi |
| 6 | Vimshottari | Rust | Vedic planetary period system |
| 7 | Biofield | Rust | Energy body resonance patterns |
| 8 | Vedic Clock | Rust | Muhurta, hora, ghati time divisions |
| 9 | Tarot | Rust | Archetypal card readings |
| 10 | Natal Chart | TS | Full birth chart calculation |
| 11 | Transit | TS | Current planetary transits |
| 12 | Compatibility | TS | Synastry and composite charts |
| 13 | Divisional | TS | D9, D10 and other divisional charts |
| 14 | Dasha | TS | Extended dasha calculations |

Plus 6 orchestrated workflows (witness, snapshot, compass, timeline, compatibility, transit).

## Config File

The wizard creates `.selemenerc.json` in your project root:

```json
{
  "version": "1.0",
  "rustUrl": "http://localhost:8080",
  "tsUrl": "http://localhost:3001",
  "apiKey": "nk_...",
  "frameworks": ["claude", "openai"],
  "outputDir": "./selemene-tools",
  "lastGenerated": "2026-02-10T06:00:00Z"
}
```

## Requirements

- Node.js 18+ (uses native `fetch`)
- A running Selemene Engine instance (Rust server, optionally TS engines)

## Troubleshooting

**"Neither server is reachable"**
- Ensure Selemene is running: `cargo run` or check your deployment URL
- Check firewall/network: `curl http://localhost:8080/health/live`

**"Partial connectivity"**
- The TS engines server is optional. If only Rust is running, you'll get 9 engines instead of 14.

**"No .selemenerc.json found"**
- Run `npx @selemene/bridge init` first to create the config.

**Regenerating after API changes**
- Run `npx @selemene/bridge generate` to re-fetch specs and regenerate all tool definitions.

## License

MIT
