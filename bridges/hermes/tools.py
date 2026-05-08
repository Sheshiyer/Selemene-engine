"""Noesis tool definitions for Hermes / OpenAI function-calling format.

Call get_noesis_tools() to get the full list of tool definitions ready to
pass as the `tools` argument to any OpenAI-compatible chat completions API.
"""
from typing import Any

NOESIS_BASE_URL = "https://selemene.tryambakam.space"

# All 16 engine IDs
ENGINES = [
    "panchanga", "human-design", "gene-keys", "vimshottari", "numerology",
    "biorhythm", "vedic-clock", "biofield", "face-reading", "nadabrahman",
    "transits", "tarot", "i-ching", "enneagram", "sacred-geometry", "sigil-forge",
]

# All 6 workflow IDs
WORKFLOWS = [
    "birth-blueprint", "daily-practice", "decision-support",
    "self-inquiry", "creative-expression", "full-spectrum",
]

# Human-readable descriptions for each engine
_ENGINE_DESCRIPTIONS: dict[str, str] = {
    "panchanga":       "Vedic calendar — tithi, nakshatra, yoga, karana, and auspicious timings",
    "human-design":    "Human Design bodygraph — type, strategy, authority, profile, and centers",
    "gene-keys":       "Gene Keys — shadow, gift, siddhi activation sequence for your hologenetic profile",
    "vimshottari":     "Vimshottari dasha — 120-year nested planetary period timeline",
    "numerology":      "Pythagorean + Chaldean numerology — life path, expression, soul urge, and destiny numbers",
    "biorhythm":       "Biological cycle analysis — physical, emotional, and intellectual sine wave rhythms",
    "vedic-clock":     "Vedic time — TCM organ clock, Ayurvedic muhurta, hora, and dosha timing",
    "biofield":        "Biofield + chakra analysis derived from birth data and planetary positions",
    "face-reading":    "Physiognomy — facial feature analysis mapped to personality and life patterns",
    "nadabrahman":     "Sound consciousness — nada yoga frequencies and vibrational resonance",
    "transits":        "Planetary transits — current aspects, Sade Sati, and transit-to-natal overlays",
    "tarot":           "Tarot card spread — archetypal guidance and symbolic storytelling",
    "i-ching":         "I-Ching hexagram divination — change patterns and Wu Wei guidance",
    "enneagram":       "Enneagram type analysis — core motivation, growth path, and stress/security moves",
    "sacred-geometry": "Sacred geometry — numerical ratios, Platonic solids, and geometric pattern analysis",
    "sigil-forge":     "Sigil creation — intention-to-symbol encoding for focused awareness practice",
}

_WORKFLOW_DESCRIPTIONS: dict[str, str] = {
    "birth-blueprint":    "Numerology + Human Design + Vimshottari — core life architecture reading",
    "daily-practice":     "Panchanga + Vedic Clock + Biorhythm — optimal timing for today",
    "decision-support":   "Multi-engine guidance for evaluating choices and timing decisions",
    "self-inquiry":       "Reflective witness prompts across consciousness systems",
    "creative-expression": "Archetypal and symbolic guidance for creative work",
    "full-spectrum":      "All 16 engines combined — comprehensive consciousness snapshot",
}

# Shared birth_data schema (reused by all tools)
_BIRTH_DATA_SCHEMA: dict[str, Any] = {
    "type": "object",
    "description": "Birth data for the subject. Required by most engines.",
    "properties": {
        "name": {
            "type": "string",
            "description": "Subject's name (required for numerology engine)",
        },
        "date": {
            "type": "string",
            "description": "Birth date in YYYY-MM-DD format",
            "pattern": r"^\d{4}-\d{2}-\d{2}$",
        },
        "time": {
            "type": "string",
            "description": "Birth time in HH:MM (24-hour) format. Defaults to noon if omitted.",
            "pattern": r"^\d{2}:\d{2}$",
        },
        "latitude": {
            "type": "number",
            "description": "Birth place latitude (decimal degrees, -90..90)",
        },
        "longitude": {
            "type": "number",
            "description": "Birth place longitude (decimal degrees, -180..180)",
        },
        "timezone": {
            "type": "string",
            "description": "IANA timezone string, e.g. 'Asia/Kolkata' or 'America/New_York'",
        },
    },
    "required": ["date", "latitude", "longitude", "timezone"],
}

# Base engine_input schema
_ENGINE_INPUT_BASE: dict[str, Any] = {
    "type": "object",
    "properties": {
        "birth_data": _BIRTH_DATA_SCHEMA,
        "current_time": {
            "type": "string",
            "description": "Override the 'now' timestamp (ISO-8601). Defaults to server time.",
        },
        "precision": {
            "type": "string",
            "enum": ["Standard", "High", "Extreme"],
            "description": "Calculation precision level. 'Standard' is sufficient for most uses.",
            "default": "Standard",
        },
        "options": {
            "type": "object",
            "description": "Engine-specific options. For sigil-forge, pass intention text as options.question.",
            "additionalProperties": True,
        },
    },
}


def _engine_tool(engine_id: str) -> dict[str, Any]:
    """Build an OpenAI function definition for a single engine calculate call."""
    description = _ENGINE_DESCRIPTIONS.get(
        engine_id, f"Calculate {engine_id} consciousness metrics"
    )
    schema = dict(_ENGINE_INPUT_BASE)
    if engine_id == "numerology":
        # name is required for numerology
        schema = dict(_ENGINE_INPUT_BASE)
        birth = dict(_BIRTH_DATA_SCHEMA)
        birth["required"] = ["name", "date", "latitude", "longitude", "timezone"]
        schema["properties"] = {**_ENGINE_INPUT_BASE["properties"], "birth_data": birth}

    return {
        "type": "function",
        "function": {
            "name": f"noesis_engine_{engine_id.replace('-', '_')}",
            "description": (
                f"Noesis engine: {description}. "
                "Returns engine_id, result (structured data), witness_prompt, and calculation_time_ms."
            ),
            "parameters": {
                **schema,
                "required": [],  # birth_data is optional at top level (some engines use current_time only)
            },
        },
    }


def _workflow_tool(workflow_id: str) -> dict[str, Any]:
    """Build an OpenAI function definition for a workflow execute call."""
    description = _WORKFLOW_DESCRIPTIONS.get(workflow_id, f"Execute {workflow_id} workflow")
    return {
        "type": "function",
        "function": {
            "name": f"noesis_workflow_{workflow_id.replace('-', '_')}",
            "description": (
                f"Noesis workflow: {description}. "
                "Returns workflow_id, engine_outputs (Record<engine_id, EngineOutput>), "
                "synthesis (themes/alignments/tensions), total_time_ms."
            ),
            "parameters": {
                **_ENGINE_INPUT_BASE,
                "required": ["birth_data"],
            },
        },
    }


def _meta_tools() -> list[dict[str, Any]]:
    """Utility tools: list engines, list workflows, get engine info."""
    return [
        {
            "type": "function",
            "function": {
                "name": "noesis_list_engines",
                "description": "List all 16 available Noesis consciousness engines with their IDs and descriptions.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_list_workflows",
                "description": "List all 6 available Noesis multi-engine workflows.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_engine_info",
                "description": "Get detailed info (input schema, description) for a specific engine.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "engine_id": {
                            "type": "string",
                            "enum": ENGINES,
                            "description": "The engine ID to get info for",
                        }
                    },
                    "required": ["engine_id"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_workflow_info",
                "description": "Get detailed info (engines included, description) for a specific workflow.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "workflow_id": {
                            "type": "string",
                            "enum": WORKFLOWS,
                            "description": "The workflow ID to get info for",
                        }
                    },
                    "required": ["workflow_id"],
                },
            },
        },
    ]


def get_noesis_tools(
    include_engines: list[str] | None = None,
    include_workflows: list[str] | None = None,
    include_meta: bool = True,
) -> list[dict[str, Any]]:
    """Return all Noesis tool definitions in OpenAI function-calling format.

    Args:
        include_engines: Engine IDs to include. Defaults to all 16.
        include_workflows: Workflow IDs to include. Defaults to all 6.
        include_meta: Include meta-tools (list_engines, engine_info, etc.). Default True.

    Returns:
        List of tool definitions ready to pass as `tools=` to an OpenAI chat call.
    """
    engines = include_engines or ENGINES
    workflows = include_workflows or WORKFLOWS

    tools: list[dict[str, Any]] = []
    if include_meta:
        tools.extend(_meta_tools())
    for engine_id in engines:
        tools.append(_engine_tool(engine_id))
    for workflow_id in workflows:
        tools.append(_workflow_tool(workflow_id))
    return tools
