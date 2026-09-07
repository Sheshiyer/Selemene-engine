"""Canonical Noesis tool definitions for Hermes/OpenAI function calling.

The catalogue is generated from the versioned engine registry. Public tools are
protected Rust operations; Full Spectrum remains visible as unsupported until
a lossless aggregate adapter exists.
"""
from __future__ import annotations

from pathlib import Path
import json
from typing import Any

NOESIS_BASE_URL = "https://selemene.tryambakam.space"


def _registry_rows() -> list[dict[str, Any]]:
    path = Path(__file__).resolve().parents[1] / ".." / "contracts" / "v1" / "registries" / "engines.json"
    try:
        payload = json.loads(path.resolve().read_text())
        return payload["engines"]
    except (OSError, ValueError, KeyError, TypeError):
        return []


_REGISTRY = _registry_rows()
RUNTIME_ENGINE_IDS = tuple(row["id"] for row in _REGISTRY)
ENGINES = tuple(
    row["id"]
    for row in _REGISTRY
    if row.get("public_mirror_group") and row.get("operations", {}).get("calculate") == "supported"
)
PUBLIC_MIRROR_COUNT = len(ENGINES)
RUNTIME_ENGINE_COUNT = len(RUNTIME_ENGINE_IDS)
WORKFLOWS = (
    "birth-blueprint",
    "daily-practice",
    "decision-support",
    "self-inquiry",
    "creative-expression",
    "full-spectrum",
)
SUPPORTED_WORKFLOWS = WORKFLOWS[:-1]

# Human-readable descriptions for each public mirror. Registry identity remains
# authoritative; descriptions are presentation metadata only.
_ENGINE_DESCRIPTIONS: dict[str, str] = {
    "panchanga": "Vedic calendar — tithi, nakshatra, yoga, karana, and auspicious timings",
    "human-design": "Human Design bodygraph — type, strategy, authority, profile, and centers",
    "gene-keys": "Gene Keys — shadow, gift, siddhi activation sequence for your hologenetic profile",
    "vimshottari": "Vimshottari dasha — 120-year nested planetary period timeline",
    "numerology": "Pythagorean + Chaldean numerology — life path, expression, soul urge, and destiny numbers",
    "biorhythm": "Biological cycle analysis — physical, emotional, and intellectual sine wave rhythms",
    "vedic-clock": "Vedic time — TCM organ clock, Ayurvedic muhurta, hora, and dosha timing",
    "biofield": "Biofield + chakra analysis derived from birth data and planetary positions",
    "face-reading": "Physiognomy — facial feature analysis mapped to personality and life patterns",
    "nadabrahman": "Sound consciousness — nada yoga frequencies and vibrational resonance",
    "transits": "Planetary transits — current aspects, Sade Sati, and transit-to-natal overlays",
    "tarot": "Tarot card spread — archetypal guidance and symbolic storytelling",
    "i-ching": "I-Ching hexagram divination — change patterns and Wu Wei guidance",
    "enneagram": "Enneagram type analysis — core motivation, growth path, and stress/security moves",
    "sacred-geometry": "Sacred geometry — numerical ratios, Platonic solids, and geometric pattern analysis",
    "sigil-forge": "Sigil creation — intention-to-symbol encoding for focused awareness practice",
    "raaga": "Raga consciousness — melakarta, swaras, ratios and vibrational resonance",
}

_WORKFLOW_DESCRIPTIONS: dict[str, str] = {
    "birth-blueprint": "Numerology + Human Design + Vimshottari — core life architecture reading",
    "daily-practice": "Panchanga + Vedic Clock + Biorhythm — optimal timing for today",
    "decision-support": "Multi-engine guidance for evaluating choices and timing decisions",
    "self-inquiry": "Reflective witness prompts across consciousness systems",
    "creative-expression": "Archetypal and symbolic guidance for creative work",
    "full-spectrum": "Unsupported pending a lossless aggregate adapter",
}

_BIRTH_DATA_SCHEMA: dict[str, Any] = {
    "type": "object",
    "description": "Birth data for the subject. Required by most engines.",
    "properties": {
        "name": {"type": "string", "description": "Subject's name"},
        "date": {"type": "string", "description": "Birth date in YYYY-MM-DD format", "pattern": r"^\d{4}-\d{2}-\d{2}$"},
        "time": {"type": "string", "description": "Birth time in HH:MM (24-hour) format", "pattern": r"^\d{2}:\d{2}$"},
        "latitude": {"type": "number", "description": "Birth place latitude (-90..90)"},
        "longitude": {"type": "number", "description": "Birth place longitude (-180..180)"},
        "timezone": {"type": "string", "description": "IANA timezone string"},
    },
    "required": ["date", "latitude", "longitude", "timezone"],
}

_ENGINE_INPUT_BASE: dict[str, Any] = {
    "type": "object",
    "properties": {
        "birth_data": _BIRTH_DATA_SCHEMA,
        "current_time": {"type": "string", "description": "Override now as ISO-8601"},
        "precision": {"type": "string", "enum": ["Standard", "High", "Extreme"], "default": "Standard"},
        "options": {"type": "object", "additionalProperties": True},
    },
}


def _engine_tool(engine_id: str) -> dict[str, Any]:
    description = _ENGINE_DESCRIPTIONS.get(engine_id, f"Calculate {engine_id} consciousness metrics")
    schema = dict(_ENGINE_INPUT_BASE)
    if engine_id == "numerology":
        birth = dict(_BIRTH_DATA_SCHEMA)
        birth["required"] = ["name", "date", "latitude", "longitude", "timezone"]
        schema["properties"] = {**_ENGINE_INPUT_BASE["properties"], "birth_data": birth}
    return {
        "type": "function",
        "function": {
            "name": f"noesis_engine_{engine_id.replace('-', '_')}",
            "description": f"Noesis engine: {description}. Returns a canonical protected Rust result.",
            "parameters": {**schema, "required": []},
            "x-selemene-operation": "calculate",
            "x-selemene-status": "supported",
            "x-selemene-runtime": "protected-rust",
        },
    }


def _workflow_tool(workflow_id: str) -> dict[str, Any]:
    supported = workflow_id in SUPPORTED_WORKFLOWS
    status = "supported" if supported else "unsupported"
    return {
        "type": "function",
        "function": {
            "name": f"noesis_workflow_{workflow_id.replace('-', '_')}",
            "description": (
                f"Noesis workflow: {_WORKFLOW_DESCRIPTIONS[workflow_id]}. "
                "Returns canonical workflow outcome with requested, output and failure records."
            ),
            "parameters": {**_ENGINE_INPUT_BASE, "required": ["birth_data"]},
            "x-selemene-operation": "execute",
            "x-selemene-status": status,
            "x-selemene-runtime": "protected-rust",
        },
    }


def _meta_tools() -> list[dict[str, Any]]:
    return [
        {
            "type": "function",
            "function": {
                "name": "noesis_list_engines",
                "description": f"List all {RUNTIME_ENGINE_COUNT} runtime identities and {PUBLIC_MIRROR_COUNT} public mirrors.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_list_capabilities",
                "description": "List the canonical v1 capability envelope with four availability states.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_list_workflows",
                "description": "List five supported workflows plus Full Spectrum as unsupported.",
                "parameters": {"type": "object", "properties": {}},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_engine_info",
                "description": "Get detailed info for a public engine mirror.",
                "parameters": {"type": "object", "properties": {"engine_id": {"type": "string", "enum": list(ENGINES)}}, "required": ["engine_id"]},
            },
        },
        {
            "type": "function",
            "function": {
                "name": "noesis_workflow_info",
                "description": "Get workflow support metadata, including Full Spectrum unsupported.",
                "parameters": {"type": "object", "properties": {"workflow_id": {"type": "string", "enum": list(WORKFLOWS)}}, "required": ["workflow_id"]},
            },
        },
    ]


def get_noesis_tools(
    include_engines: list[str] | None = None,
    include_workflows: list[str] | None = None,
    include_meta: bool = True,
) -> list[dict[str, Any]]:
    """Return canonical public tool definitions with explicit support metadata."""
    engines = include_engines if include_engines is not None else list(ENGINES)
    workflows = include_workflows if include_workflows is not None else list(WORKFLOWS)
    invalid_engines = set(engines) - set(ENGINES)
    invalid_workflows = set(workflows) - set(WORKFLOWS)
    if invalid_engines or invalid_workflows:
        raise ValueError("Unknown engine or workflow identity")
    tools: list[dict[str, Any]] = []
    if include_meta:
        tools.extend(_meta_tools())
    tools.extend(_engine_tool(engine_id) for engine_id in engines)
    tools.extend(_workflow_tool(workflow_id) for workflow_id in workflows)
    return tools
