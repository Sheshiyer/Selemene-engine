"""Hermes agent bridge for Noesis API.

Exposes all 16 Noesis engines and 6 workflows as OpenAI-compatible
function definitions usable with NousResearch Hermes models
(Hermes-3-Llama-3.1, Hermes-2-Pro, Mistral-Hermes, etc.).

Supports both:
- Standard OpenAI-compatible tool calling (Hermes-3 / Llama-3.1-based)
- Legacy Hermes-2 <tool_call> XML format (auto-detected)
"""
from .tools import get_noesis_tools, ENGINES, WORKFLOWS
from .agent import HermesAgent, HermesAgentConfig

__all__ = ["get_noesis_tools", "HermesAgent", "HermesAgentConfig", "ENGINES", "WORKFLOWS"]
