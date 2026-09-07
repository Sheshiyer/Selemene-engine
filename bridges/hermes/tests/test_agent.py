import json
from pathlib import Path

import httpx
import pytest

from bridges.hermes.agent import HermesAgentConfig, NoesisExecutor


class RecordingClient:
    def __init__(self, transport):
        self.transport = transport

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False

    def get(self, url, **kwargs):
        request = httpx.Request("GET", url, headers=kwargs.get("headers"))
        self.transport.calls.append(request)
        return httpx.Response(200, request=request, json={"ok": True})

    def post(self, url, **kwargs):
        request = httpx.Request("POST", url, headers=kwargs.get("headers"), content=json.dumps(kwargs.get("json", {})))
        self.transport.calls.append(request)
        if self.transport.status_code >= 400:
            return httpx.Response(self.transport.status_code, request=request, text="token=secret raw upstream")
        return httpx.Response(200, request=request, json={"ok": True})


class Transport:
    def __init__(self, status_code=200):
        self.calls = []
        self.status_code = status_code


def factory_for(transport):
    return lambda timeout: RecordingClient(transport)


def test_executor_uses_exact_api_key_and_rust_path():
    transport = Transport()
    config = HermesAgentConfig(noesis_base_url="https://rust.example", noesis_api_key="key")
    executor = NoesisExecutor(config, client_factory=factory_for(transport))
    result = executor.dispatch("noesis_engine_raaga", {"parameters": {}})
    assert result == {"ok": True}
    assert transport.calls[0].url.path == "/api/v1/engines/raaga/calculate"
    assert transport.calls[0].headers["x-api-key"] == "key"
    assert "authorization" not in transport.calls[0].headers


def test_executor_rejects_full_spectrum_and_invalid_names_before_transport():
    transport = Transport()
    executor = NoesisExecutor(HermesAgentConfig(), client_factory=factory_for(transport))
    for name in ["noesis_workflow_full_spectrum", "noesis_engine_raaga_evil"]:
        with pytest.raises(ValueError):
            executor.dispatch(name, {})
    with pytest.raises(ValueError):
        executor.dispatch("noesis_engine_raaga", {"engine_id": "../raaga"})
    assert transport.calls == []


def test_executor_bearer_branch_and_bounded_errors():
    transport = Transport(status_code=502)
    config = HermesAgentConfig(noesis_base_url="https://rust.example", noesis_bearer_token="bearer")
    executor = NoesisExecutor(config, client_factory=factory_for(transport))
    with pytest.raises(RuntimeError, match="UPSTREAM_ERROR"):
        executor.dispatch("noesis_engine_raaga", {})
    assert transport.calls[0].headers["authorization"] == "Bearer bearer"
    assert "x-api-key" not in transport.calls[0].headers


def test_executor_rejects_ambiguous_credentials():
    with pytest.raises(ValueError):
        NoesisExecutor(HermesAgentConfig(noesis_api_key="key", noesis_bearer_token="bearer"))
