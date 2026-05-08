# Hermes Integration (Noesis)

Connect NousResearch Hermes models to all 16 Noesis consciousness engines using OpenAI-compatible function calling.

## Integration Surface

- Base URL: `https://selemene.tryambakam.space`
- Auth: `X-API-Key: nk_<api_key>` (same as all Noesis integrations)
- Engine execute: `POST /api/v1/engines/{engine_id}/calculate`
- Workflow execute: `POST /api/v1/workflows/{workflow_id}/execute`
- Workflow info: `GET /api/v1/workflows/{workflow_id}/info`

All 16 engines are available. TypeScript engines (`tarot`, `i-ching`, `enneagram`, `sacred-geometry`, `sigil-forge`) are internally sidecar-bridged and transparent to Hermes callers.

## Bridge Location

```
bridges/hermes/
  __init__.py      — package entry point
  tools.py         — get_noesis_tools() → 22 OpenAI function definitions
  agent.py         — HermesAgent loop (Hermes-3 standard + Hermes-2 XML)
  README.md        — quick start
  requirements.txt — httpx>=0.27.0
```

## Setup

```bash
pip install -r bridges/hermes/requirements.txt

export HERMES_BASE_URL="http://localhost:11434/v1"   # ollama or any OpenAI-compat server
export HERMES_MODEL="hermes3"
export HERMES_API_KEY="ollama"
export NOESIS_API_KEY="nk_your_key_here"
```

## Usage

```python
from bridges.hermes import HermesAgent, HermesAgentConfig

agent = HermesAgent()   # reads env vars

# Single query
result = agent.run(
    "Calculate my birth blueprint. "
    "Born 1991-08-13 at 13:19 in Bangalore (12.9716N 77.5946E, Asia/Kolkata)."
)
print(result)

# Streaming
for token in agent.stream("What Panchanga timing applies today?"):
    print(token, end="", flush=True)
```

## Supported Models

| Model | Notes |
|-------|-------|
| `hermes3` (Llama-3.1-8B) | Recommended for local use via ollama |
| `hermes3:70b` | Higher quality, requires ~48 GB VRAM |
| `NousResearch/Hermes-3-Llama-3.1-70B-Turbo` | Together AI cloud |
| `NousResearch/Hermes-2-Pro-Mistral-7B` | Smaller, XML format (auto-detected) |

## Tool Format Auto-Detection

The bridge auto-detects the tool call format from the model name:
- **Hermes-3 / Llama-3.1-based**: uses standard OpenAI `tool_calls` field
- **Hermes-2-Pro / older models**: uses `<tool_call>` XML tags in content

Override with `HermesAgentConfig(force_xml_format=False/True)`.

## Tool List (22 tools)

### Meta tools
- `noesis_list_engines` — list all 16 engines
- `noesis_list_workflows` — list all 6 workflows
- `noesis_engine_info` — schema for a specific engine
- `noesis_workflow_info` — details for a specific workflow

### Engine tools
All 16 engines exposed as `noesis_engine_<engine_id>`:

```
noesis_engine_panchanga        noesis_engine_human_design
noesis_engine_gene_keys        noesis_engine_vimshottari
noesis_engine_numerology       noesis_engine_biorhythm
noesis_engine_vedic_clock      noesis_engine_biofield
noesis_engine_face_reading     noesis_engine_nadabrahman
noesis_engine_transits         noesis_engine_tarot
noesis_engine_i_ching          noesis_engine_enneagram
noesis_engine_sacred_geometry  noesis_engine_sigil_forge
```

### Workflow tools
All 6 workflows exposed as `noesis_workflow_<workflow_id>`:

```
noesis_workflow_birth_blueprint    noesis_workflow_daily_practice
noesis_workflow_decision_support   noesis_workflow_self_inquiry
noesis_workflow_creative_expression noesis_workflow_full_spectrum
```

## Request Contract

```json
{
  "birth_data": {
    "name": "Optional",
    "date": "1991-08-13",
    "time": "13:19",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  },
  "precision": "Standard",
  "options": {}
}
```

- `name` required only for `noesis_engine_numerology`
- `birth_data` required for all workflow tools
- For `noesis_engine_sigil_forge`: pass intention as `options.question`

## Verification

```bash
# 1. Check Noesis API is reachable
curl -s https://selemene.tryambakam.space/health/live

# 2. Verify engines (16 expected)
curl -s https://selemene.tryambakam.space/api/v1/engines \
  -H "X-API-Key: $NOESIS_API_KEY" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('engines', d)))"

# 3. Single engine call
curl -s -X POST https://selemene.tryambakam.space/api/v1/engines/panchanga/calculate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"birth_data":{"date":"1991-08-13","time":"13:19","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"}}'

# 4. Test bridge tool loading (no model required)
python3 -c "
from bridges.hermes.tools import get_noesis_tools
tools = get_noesis_tools()
print(f'Tools loaded: {len(tools)}')
print([t[\"function\"][\"name\"] for t in tools[:5]])
"

# 5. Full agent test (requires Hermes model running)
python3 -c "
from bridges.hermes import HermesAgent
agent = HermesAgent()
print(agent.run('List the available Noesis engines'))
"
```

## Error Handling

All tool call errors are returned as JSON in the tool message:

```json
{"error": "HTTP 401", "detail": "Invalid or expired API key"}
{"error": "HTTP 404", "detail": "Engine not found"}
{"error": "connection refused"}
```

The agent loop continues after tool errors — Hermes will decide whether to retry or report the error in its final answer.

## Difference from OpenClaw Bridge

| Aspect | OpenClaw | Hermes |
|--------|----------|--------|
| Language | TypeScript/Node | Python |
| Hosting | OpenClaw skill runtime | Any Python process |
| Model | Claude (via OpenClaw) | NousResearch Hermes |
| Format | Skill YAML + REST | OpenAI function calling |
| Tool definition | Generated JSON files | Python objects (`get_noesis_tools()`) |
| Auto-regeneration | `npx @selemene/bridge generate` | Not needed (hardcoded schemas) |

## Cloud Provider Examples

### Together AI

```python
import os
from bridges.hermes import HermesAgent, HermesAgentConfig

config = HermesAgentConfig(
    hermes_base_url="https://api.together.xyz/v1",
    hermes_model="NousResearch/Hermes-3-Llama-3.1-70B-Turbo",
    hermes_api_key=os.environ["TOGETHER_API_KEY"],
    noesis_api_key=os.environ["NOESIS_API_KEY"],
)
agent = HermesAgent(config)
```

### LM Studio (local)

```python
config = HermesAgentConfig(
    hermes_base_url="http://localhost:1234/v1",
    hermes_model="hermes-3-llama-3.1-8b",
    hermes_api_key="lm-studio",
)
```

## See Also

- [OpenClaw Integration](OPENCLAW_INTEGRATION.md)
- [MCP Integration](MCP_INTEGRATION.md)
- [Bridge tooling](../../bridges/README.md)
- [Engine list](../../docs/ENGINES.md)
- [API Quickstart](../../docs/API_QUICKSTART.md)
