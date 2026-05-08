# Noesis Agent Bridges

Expose Noesis as reflection-first, self-authorship tools for AI agents. One OpenAPI spec in, framework-native tool definitions out.

## How It Works

```
Noesis (Rust/TS API)
    ↓ OpenAPI spec (https://selemene.tryambakam.space/api/openapi.json)
Bridge Generators (Python / TypeScript CLI)
    ↓
├── Claude tool definitions (tools.json)
├── OpenAI function definitions (functions.json)
└── LangChain/CrewAI StructuredTools (selemene_tools.py)
```

The generators read the live OpenAPI specs from a running Selemene instance and produce framework-native tool definitions. No manual schema maintenance required — regenerate after any API change.

## Directory Layout

| Directory | Description | Status |
|-----------|-------------|--------|
| [`cli/`](cli/README.md) | `@selemene/bridge` TypeScript CLI — interactive setup wizard | Active |
| [`langchain/`](langchain/) | LangChain/CrewAI Python bridge — StructuredTool generation | Active |
| `claude/` | Generated Claude tool definitions output | Output dir |
| `openai/` | Generated OpenAI function definitions output | Output dir |
| [`universal-tool-server/`](universal-tool-server/) | MCP-compatible FastAPI server | Scaffold |
| [`hermes/`](hermes/) | NousResearch Hermes function-calling bridge | Active |
| [`cron/`](cron/) | Scheduled jobs (daily witness, hourly panchanga) | Scaffold |
| [`tests/`](tests/) | Bridge integration tests | Active |

## Quick Start

```bash
npx @selemene/bridge init
```

The interactive wizard validates server connectivity, lets you pick frameworks, and generates tool definitions in one pass.

For detailed CLI usage, options, and framework-specific examples, see the [CLI documentation](cli/README.md).

## Engines Covered

The bridge generates tools for all 16 Noesis engines:

- **11 Rust engines** (live): Panchanga, Numerology, Biorhythm, Human Design, Gene Keys, Vimshottari, Biofield, Vedic Clock, Face Reading, Nadabrahman, Transits
- **5 TypeScript engines** (live): Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge

Plus 6 orchestrated workflows.

## Requirements

- Node.js 18+ (for CLI)
- Python 3.10+ (for LangChain bridge, optional)
- A running Noesis instance (or use `https://selemene.tryambakam.space`)

## License

MIT
