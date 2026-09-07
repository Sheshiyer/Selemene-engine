"""Hermes agent loop for Noesis tool execution.

Supports:
- Hermes-3 (standard OpenAI tool calling via chat/completions)
- Hermes-2-Pro / legacy (XML <tool_call> tag parsing)
- Any OpenAI-compatible server (llama.cpp, ollama, Together AI, etc.)
"""
from __future__ import annotations

import json
import os
import re
from urllib.parse import quote
from dataclasses import dataclass, field
from typing import Any, Iterator

import httpx

from .tools import (
    get_noesis_tools,
    ENGINES,
    WORKFLOWS,
    SUPPORTED_WORKFLOWS,
    NOESIS_BASE_URL,
)

# --- Configuration -------------------------------------------------------

@dataclass
class HermesAgentConfig:
    """Runtime configuration for the Hermes agent."""

    # Hermes model endpoint (OpenAI-compatible chat completions)
    hermes_base_url: str = field(
        default_factory=lambda: os.environ.get(
            "HERMES_BASE_URL", "http://localhost:11434/v1"  # ollama default
        )
    )
    hermes_api_key: str = field(
        default_factory=lambda: os.environ.get("HERMES_API_KEY", "ollama")
    )
    hermes_model: str = field(
        default_factory=lambda: os.environ.get("HERMES_MODEL", "hermes3")
    )

    # Noesis API
    noesis_base_url: str = field(
        default_factory=lambda: os.environ.get("NOESIS_BASE_URL", NOESIS_BASE_URL)
    )
    noesis_api_key: str = field(
        default_factory=lambda: os.environ.get("NOESIS_API_KEY", "")
    )
    noesis_bearer_token: str = field(
        default_factory=lambda: os.environ.get("NOESIS_BEARER_TOKEN", "")
    )

    # Agent behaviour
    max_iterations: int = 10
    timeout_secs: float = 60.0
    # Force legacy XML format (auto-detected from model name if None)
    force_xml_format: bool | None = None


# --- Noesis tool executor ------------------------------------------------

class NoesisExecutor:
    """Execute Noesis tool calls against the live API."""

    def __init__(self, config: HermesAgentConfig, client_factory: Any = None) -> None:
        self._base = config.noesis_base_url.rstrip("/")
        self._headers: dict[str, str] = {"Content-Type": "application/json"}
        if config.noesis_api_key and config.noesis_bearer_token:
            raise ValueError("Choose either noesis_api_key or noesis_bearer_token")
        if config.noesis_api_key:
            self._headers["X-API-Key"] = config.noesis_api_key
        elif config.noesis_bearer_token:
            self._headers["Authorization"] = f"Bearer {config.noesis_bearer_token}"
        self._timeout = config.timeout_secs
        self._client_factory = client_factory or httpx.Client

    # --- Meta tools ---

    def noesis_list_engines(self, **_: Any) -> Any:
        return self._get("/api/v1/engines")

    def noesis_list_capabilities(self, **_: Any) -> Any:
        return self._get("/api/v1/engines/capabilities")

    def noesis_list_workflows(self, **_: Any) -> Any:
        return self._get("/api/v1/workflows")

    def noesis_engine_info(self, engine_id: str, **_: Any) -> Any:
        engine_id = self._validate_engine_id(engine_id)
        return self._get(f"/api/v1/engines/{quote(engine_id, safe='')}/info")

    def noesis_workflow_info(self, workflow_id: str, **_: Any) -> Any:
        workflow_id = self._validate_workflow_id(workflow_id, allow_unsupported=True)
        return self._get(f"/api/v1/workflows/{quote(workflow_id, safe='')}/info")

    # --- Dynamic dispatch for engine/workflow calls ---

    def _engine_calculate(self, engine_id: str, body: dict[str, Any]) -> Any:
        engine_id = self._validate_engine_id(engine_id)
        return self._post(f"/api/v1/engines/{quote(engine_id, safe='')}/calculate", body)

    def _workflow_execute(self, workflow_id: str, body: dict[str, Any]) -> Any:
        workflow_id = self._validate_workflow_id(workflow_id)
        return self._post(f"/api/v1/workflows/{quote(workflow_id, safe='')}/execute", body)

    @staticmethod
    def _validate_engine_id(engine_id: str) -> str:
        if not isinstance(engine_id, str) or engine_id not in ENGINES:
            raise ValueError("Unknown or unavailable public engine")
        return engine_id

    @staticmethod
    def _validate_workflow_id(workflow_id: str, allow_unsupported: bool = False) -> str:
        if not isinstance(workflow_id, str) or workflow_id not in WORKFLOWS:
            raise ValueError("Unknown workflow")
        if workflow_id == "full-spectrum" and not allow_unsupported:
            raise ValueError("Workflow is unsupported")
        return workflow_id

    def dispatch(self, tool_name: str, arguments: dict[str, Any]) -> Any:
        """Route tool_name → correct Noesis API call."""
        # Meta tools
        if tool_name == "noesis_list_engines":
            return self.noesis_list_engines()
        if tool_name == "noesis_list_capabilities":
            return self.noesis_list_capabilities()
        if tool_name == "noesis_list_workflows":
            return self.noesis_list_workflows()
        if tool_name == "noesis_engine_info":
            return self.noesis_engine_info(**arguments)
        if tool_name == "noesis_workflow_info":
            return self.noesis_workflow_info(**arguments)

        # Engine tools: noesis_engine_<engine_id_underscored>
        if tool_name.startswith("noesis_engine_"):
            engine_slug = tool_name[len("noesis_engine_"):].replace("_", "-")
            if engine_slug in ENGINES:
                supplied_id = arguments.get("engine_id")
                if supplied_id is not None and supplied_id != engine_slug:
                    raise ValueError("Engine argument does not match tool identity")
                body = {key: value for key, value in arguments.items() if key != "engine_id"}
                return self._engine_calculate(engine_slug, body)

        # Workflow tools: noesis_workflow_<workflow_id_underscored>
        if tool_name.startswith("noesis_workflow_"):
            workflow_slug = tool_name[len("noesis_workflow_"):].replace("_", "-")
            if workflow_slug in SUPPORTED_WORKFLOWS:
                body = {key: value for key, value in arguments.items() if key != "workflow_id"}
                return self._workflow_execute(workflow_slug, body)
            if workflow_slug == "full-spectrum":
                raise ValueError("Workflow is unsupported")

        raise ValueError(f"Unknown Noesis tool: {tool_name}")

    # --- HTTP helpers ---

    def _get(self, path: str) -> Any:
        try:
            with self._client_factory(timeout=self._timeout) as client:
                r = client.get(f"{self._base}{path}", headers=self._headers)
                r.raise_for_status()
                return self._decode_response(r)
        except httpx.TimeoutException as exc:
            raise RuntimeError("UPSTREAM_TIMEOUT") from exc
        except httpx.RequestError as exc:
            raise RuntimeError("UPSTREAM_UNAVAILABLE") from exc
        except httpx.HTTPStatusError as exc:
            raise RuntimeError("UPSTREAM_ERROR") from exc

    def _post(self, path: str, body: dict[str, Any]) -> Any:
        try:
            with self._client_factory(timeout=self._timeout) as client:
                r = client.post(f"{self._base}{path}", headers=self._headers, json=body)
                r.raise_for_status()
                return self._decode_response(r)
        except httpx.TimeoutException as exc:
            raise RuntimeError("UPSTREAM_TIMEOUT") from exc
        except httpx.RequestError as exc:
            raise RuntimeError("UPSTREAM_UNAVAILABLE") from exc
        except httpx.HTTPStatusError as exc:
            raise RuntimeError("UPSTREAM_ERROR") from exc

    @staticmethod
    def _decode_response(response: httpx.Response) -> Any:
        try:
            payload = response.json()
        except ValueError as exc:
            raise RuntimeError("UPSTREAM_INVALID_RESPONSE") from exc
        if not isinstance(payload, (dict, list)):
            raise RuntimeError("UPSTREAM_INVALID_RESPONSE")
        return payload


# --- XML tool-call parser (Hermes-2 legacy) ------------------------------

_TOOL_CALL_RE = re.compile(
    r"<tool_call>\s*(.*?)\s*</tool_call>", re.DOTALL | re.IGNORECASE
)


def _parse_xml_tool_calls(content: str) -> list[dict[str, Any]]:
    """Extract tool calls from Hermes-2 <tool_call> XML tags."""
    calls = []
    for match in _TOOL_CALL_RE.finditer(content):
        try:
            obj = json.loads(match.group(1))
            calls.append({"name": obj["name"], "arguments": obj.get("arguments", {})})
        except (json.JSONDecodeError, KeyError):
            continue
    return calls


# --- Hermes system prompt ------------------------------------------------

_HERMES_SYSTEM_PROMPT = """You are a Noesis consciousness engine interface powered by the Hermes agent framework.

Noesis is a reflection-first platform — return patterns for the user to witness, not prescriptions to follow.
You have access to 16 consciousness calculation engines and 6 multi-engine workflows.

When the user asks for calculations, readings, or timing information, call the appropriate tool(s).
Always present results as observations to reflect on, not directives to follow.

Tool calling rules:
- Always call noesis_list_engines or noesis_list_workflows first if you are unsure which engine to use.
- For birth-chart engines (human-design, gene-keys, vimshottari, numerology, panchanga, biofield, face-reading, nadabrahman), birth_data is required.
- For timing/transit engines (vedic-clock, biorhythm, transits), current_time is used; birth_data enriches the response.
- For symbolic engines (tarot, i-ching, sigil-forge), birth_data is optional; pass an intention via options.question for sigil-forge.
- For multi-engine workflows, always pass birth_data."""


# --- Main agent ----------------------------------------------------------

class HermesAgent:
    """Hermes function-calling agent wired to Noesis tools.

    Example usage::

        config = HermesAgentConfig(
            hermes_base_url="http://localhost:11434/v1",
            hermes_model="hermes3",
            noesis_api_key="nk_...",
        )
        agent = HermesAgent(config)
        result = agent.run("Calculate my birth blueprint for 1991-08-13 13:19 Asia/Kolkata 12.97 77.59")
        print(result)

    Streaming example::

        for chunk in agent.stream("What are my biorhythms today?"):
            print(chunk, end="", flush=True)
    """

    def __init__(self, config: HermesAgentConfig | None = None) -> None:
        self.config = config or HermesAgentConfig()
        self.executor = NoesisExecutor(self.config)
        self.tools = get_noesis_tools()
        self._use_xml = self._detect_xml_mode()

    def _detect_xml_mode(self) -> bool:
        """Use XML tool_call format for Hermes-2 models; OpenAI format for Hermes-3."""
        if self.config.force_xml_format is not None:
            return self.config.force_xml_format
        model = self.config.hermes_model.lower()
        # Hermes-3 and later use standard OpenAI tool calling
        return "hermes-2" in model or ("hermes" in model and "3" not in model)

    def run(self, user_message: str) -> str:
        """Run agent loop until a final answer is produced.

        Returns the final assistant message text.
        """
        messages: list[dict[str, Any]] = [
            {"role": "system", "content": _HERMES_SYSTEM_PROMPT},
            {"role": "user", "content": user_message},
        ]

        for _iteration in range(self.config.max_iterations):
            response = self._chat(messages)
            assistant_msg = response["choices"][0]["message"]
            messages.append(assistant_msg)

            if self._use_xml:
                # Hermes-2: parse XML tool calls from content
                tool_calls = _parse_xml_tool_calls(assistant_msg.get("content", ""))
                if not tool_calls:
                    return assistant_msg.get("content", "")
                for call in tool_calls:
                    result = self._execute_tool(call["name"], call["arguments"])
                    messages.append({
                        "role": "tool",
                        "name": call["name"],
                        "content": json.dumps(result),
                    })
            else:
                # Hermes-3 / standard OpenAI: structured tool_calls field
                if not assistant_msg.get("tool_calls"):
                    return assistant_msg.get("content", "")
                for tc in assistant_msg["tool_calls"]:
                    fn = tc["function"]
                    args = json.loads(fn.get("arguments", "{}"))
                    result = self._execute_tool(fn["name"], args)
                    messages.append({
                        "role": "tool",
                        "tool_call_id": tc["id"],
                        "content": json.dumps(result),
                    })

        return "Max iterations reached without a final answer."

    def stream(self, user_message: str) -> Iterator[str]:
        """Stream the agent's final answer token by token.

        Intermediate tool calls are executed silently; only the final text
        response is streamed.
        """
        # Run tool loop non-streaming, then stream the final response
        messages: list[dict[str, Any]] = [
            {"role": "system", "content": _HERMES_SYSTEM_PROMPT},
            {"role": "user", "content": user_message},
        ]

        for _iteration in range(self.config.max_iterations):
            response = self._chat(messages)
            assistant_msg = response["choices"][0]["message"]

            has_tools = (
                bool(_parse_xml_tool_calls(assistant_msg.get("content", "")))
                if self._use_xml
                else bool(assistant_msg.get("tool_calls"))
            )

            if not has_tools:
                # Final answer — yield it token by token
                for token in (assistant_msg.get("content", "") or ""):
                    yield token
                return

            messages.append(assistant_msg)
            if self._use_xml:
                for call in _parse_xml_tool_calls(assistant_msg.get("content", "")):
                    result = self._execute_tool(call["name"], call["arguments"])
                    messages.append({
                        "role": "tool",
                        "name": call["name"],
                        "content": json.dumps(result),
                    })
            else:
                for tc in assistant_msg["tool_calls"]:
                    fn = tc["function"]
                    args = json.loads(fn.get("arguments", "{}"))
                    result = self._execute_tool(fn["name"], args)
                    messages.append({
                        "role": "tool",
                        "tool_call_id": tc["id"],
                        "content": json.dumps(result),
                    })

    def _chat(self, messages: list[dict[str, Any]]) -> dict[str, Any]:
        """Call the Hermes chat completions endpoint."""
        payload: dict[str, Any] = {
            "model": self.config.hermes_model,
            "messages": messages,
        }
        if not self._use_xml:
            payload["tools"] = self.tools
            payload["tool_choice"] = "auto"

        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {self.config.hermes_api_key}",
        }
        with httpx.Client(timeout=self.config.timeout_secs) as client:
            r = client.post(
                f"{self.config.hermes_base_url.rstrip('/')}/chat/completions",
                headers=headers,
                json=payload,
            )
            r.raise_for_status()
            return r.json()

    def _execute_tool(self, name: str, arguments: dict[str, Any]) -> Any:
        """Execute a single tool call and return the result (or error dict)."""
        try:
            return self.executor.dispatch(name, arguments)
        except Exception as e:  # noqa: BLE001
            # Never return URL, response body, provider text or credentials.
            reason = str(e)
            if reason not in {
                "UPSTREAM_TIMEOUT",
                "UPSTREAM_UNAVAILABLE",
                "UPSTREAM_ERROR",
                "UPSTREAM_INVALID_RESPONSE",
            }:
                reason = "TOOL_REJECTED"
            return {"error": reason}
