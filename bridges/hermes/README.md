# Hermes Bridge for Noesis

Connect [NousResearch Hermes](https://huggingface.co/NousResearch) models to all 16 Noesis consciousness engines via OpenAI-compatible function calling.

Works with any OpenAI-compatible server: **ollama**, **llama.cpp**, **vLLM**, **Together AI**, **Groq**, or a local `hermes3` instance.

## Supported Models

| Model | Format | Notes |
|-------|--------|-------|
| Hermes-3-Llama-3.1-8B / 70B | OpenAI tool calling | Recommended |
| Hermes-3-Llama-3.2-3B | OpenAI tool calling | Lightweight |
| Hermes-2-Pro-Mistral-7B | XML `<tool_call>` tags | Auto-detected |
| Hermes-2-Pro-Llama-3 | XML `<tool_call>` tags | Auto-detected |
| Any OpenAI-compatible server | Standard or XML | Configurable |

## Quick Start

### 1. Install dependencies

```bash
pip install -r bridges/hermes/requirements.txt
```

### 2. Set environment variables

```bash
# Hermes model server (ollama example)
export HERMES_BASE_URL="http://localhost:11434/v1"
export HERMES_MODEL="hermes3"
export HERMES_API_KEY="ollama"   # Any non-empty value for ollama

# Noesis API
export NOESIS_API_KEY="nk_your_api_key"
```

### 3. Pull the Hermes model (ollama)

```bash
ollama pull hermes3
```

### 4. Run the agent

```python
from bridges.hermes import HermesAgent, HermesAgentConfig

config = HermesAgentConfig()   # reads env vars
agent = HermesAgent(config)

result = agent.run(
    "Calculate my birth blueprint. "
    "Born 1991-08-13 at 13:19 in Bangalore, India (12.9716°N, 77.5946°E, Asia/Kolkata)."
)
print(result)
```

Or stream the response:

```python
for token in agent.stream("What are my biorhythms today?"):
    print(token, end="", flush=True)
```

## Configuration Reference

| Env var | Default | Description |
|---------|---------|-------------|
| `HERMES_BASE_URL` | `http://localhost:11434/v1` | OpenAI-compatible chat completions base URL |
| `HERMES_MODEL` | `hermes3` | Model name to request from the server |
| `HERMES_API_KEY` | `ollama` | API key (use `"ollama"` for local ollama, or your provider key) |
| `NOESIS_BASE_URL` | `https://selemene.tryambakam.space` | Noesis API base URL |
| `NOESIS_API_KEY` | *(required)* | Noesis `X-API-Key` header value (`nk_...`) |

Python overrides:

```python
config = HermesAgentConfig(
    hermes_base_url="https://api.together.xyz/v1",
    hermes_model="NousResearch/Hermes-3-Llama-3.1-70B-Turbo",
    hermes_api_key="your_together_key",
    noesis_api_key="nk_your_key",
    max_iterations=15,
    timeout_secs=120.0,
    force_xml_format=False,  # override auto-detection
)
```

## Tool Surface

The bridge exposes **22 tools** by default:

### Meta tools (4)
| Tool | Description |
|------|-------------|
| `noesis_list_engines` | List all 16 engines |
| `noesis_list_workflows` | List all 6 workflows |
| `noesis_engine_info` | Get schema for a specific engine |
| `noesis_workflow_info` | Get details for a specific workflow |

### Engine tools (16)
| Tool | Engine |
|------|--------|
| `noesis_engine_panchanga` | Vedic calendar |
| `noesis_engine_human_design` | Human Design bodygraph |
| `noesis_engine_gene_keys` | Gene Keys |
| `noesis_engine_vimshottari` | Vimshottari dasha |
| `noesis_engine_numerology` | Numerology |
| `noesis_engine_biorhythm` | Biorhythm cycles |
| `noesis_engine_vedic_clock` | Vedic timing |
| `noesis_engine_biofield` | Biofield + chakras |
| `noesis_engine_face_reading` | Physiognomy |
| `noesis_engine_nadabrahman` | Sound consciousness |
| `noesis_engine_transits` | Planetary transits |
| `noesis_engine_tarot` | Tarot |
| `noesis_engine_i_ching` | I-Ching |
| `noesis_engine_enneagram` | Enneagram |
| `noesis_engine_sacred_geometry` | Sacred geometry |
| `noesis_engine_sigil_forge` | Sigil creation |

### Workflow tools (6)
| Tool | Workflow |
|------|----------|
| `noesis_workflow_birth_blueprint` | Core life architecture |
| `noesis_workflow_daily_practice` | Today's optimal timing |
| `noesis_workflow_decision_support` | Multi-engine guidance |
| `noesis_workflow_self_inquiry` | Reflective prompts |
| `noesis_workflow_creative_expression` | Archetypal creative |
| `noesis_workflow_full_spectrum` | All engines |

## Selective Tool Loading

Use `get_noesis_tools()` to load a subset:

```python
from bridges.hermes.tools import get_noesis_tools

# Only load timing engines + one workflow
tools = get_noesis_tools(
    include_engines=["panchanga", "vedic-clock", "biorhythm", "transits"],
    include_workflows=["daily-practice"],
    include_meta=True,
)
```

## Tool Call Format

### Hermes-3 (standard OpenAI) — auto-detected

Hermes-3 uses the standard OpenAI `tool_calls` field. No special handling required.

### Hermes-2-Pro (XML legacy) — auto-detected

Hermes-2-Pro emits tool calls as XML in the assistant message content:

```xml
<tool_call>
{"name": "noesis_engine_panchanga", "arguments": {"birth_data": {"date": "1991-08-13", "latitude": 12.9716, "longitude": 77.5946, "timezone": "Asia/Kolkata"}}}
</tool_call>
```

The bridge parses these automatically. No configuration needed.

## Request Contract

All engine/workflow tools accept:

```json
{
  "birth_data": {
    "name": "Optional Name",
    "date": "YYYY-MM-DD",
    "time": "HH:MM",
    "latitude": 12.9716,
    "longitude": 77.5946,
    "timezone": "Asia/Kolkata"
  },
  "current_time": "2026-05-08T07:30:00Z",
  "precision": "Standard",
  "options": {}
}
```

Notes:
- `birth_data` is required for birth-chart engines and all workflows
- `name` inside `birth_data` is required for the `numerology` engine
- `current_time` defaults to server time
- For `sigil-forge`, pass the intention as `options.question`

## Using with Cloud Providers

### Together AI

```python
config = HermesAgentConfig(
    hermes_base_url="https://api.together.xyz/v1",
    hermes_model="NousResearch/Hermes-3-Llama-3.1-70B-Turbo",
    hermes_api_key=os.environ["TOGETHER_API_KEY"],
)
```

### Groq

```python
config = HermesAgentConfig(
    hermes_base_url="https://api.groq.com/openai/v1",
    hermes_model="llama-3.1-70b-versatile",  # Groq doesn't host Hermes directly; use compatible model
    hermes_api_key=os.environ["GROQ_API_KEY"],
    force_xml_format=False,
)
```

### LM Studio

```python
config = HermesAgentConfig(
    hermes_base_url="http://localhost:1234/v1",
    hermes_model="hermes-3-llama-3.1-8b",
    hermes_api_key="lm-studio",
)
```

## Verification

After setup, run the connectivity check:

```bash
python -c "
from bridges.hermes.agent import NoesisExecutor, HermesAgentConfig
e = NoesisExecutor(HermesAgentConfig())
r = e.noesis_list_engines()
print(f'Engines available: {len(r.get(\"engines\", []))}')
"
```

Expected output: `Engines available: 16`

Full end-to-end test:

```python
from bridges.hermes import HermesAgent
agent = HermesAgent()
print(agent.run("List the available Noesis engines."))
```

## Relationship to Other Bridges

| Bridge | Location | Format |
|--------|----------|--------|
| CLI generator | `bridges/cli/` | TypeScript, OpenAI + Claude JSON files |
| LangChain/CrewAI | `bridges/langchain/` | Python StructuredTool |
| Universal tool server | `bridges/universal-tool-server/` | MCP-compatible FastAPI |
| **Hermes (this)** | `bridges/hermes/` | OpenAI function calling + XML |
| OpenClaw | `docs/api/OPENCLAW_INTEGRATION.md` | skill-based |

## Architecture

```
User prompt
    ↓
HermesAgent.run()
    ↓ chat/completions (Hermes model)
    ↓ ← tool_calls or <tool_call> XML
    ↓
NoesisExecutor.dispatch()
    ↓ HTTP POST/GET
Noesis API (selemene.tryambakam.space)
    ↓
Engine/Workflow result → tool message
    ↓ (loop until no more tool calls)
Final answer
```
