import json
from pathlib import Path

import httpx
import pytest
from httpx import ASGITransport

import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import main


@pytest.fixture
def client():
    def factory(*, timeout):
        return httpx.AsyncClient(transport=ASGITransport(app=main.app), base_url="http://rust.test")

    # This factory is replaced per test with a recording upstream client.
    return main.app


class RecordingTransport(httpx.AsyncBaseTransport):
    def __init__(self, status_code=200, payload=None, text=None):
        self.calls = []
        self.status_code = status_code
        self.payload = payload if payload is not None else {"ok": True}
        self.text = text

    async def handle_async_request(self, request):
        self.calls.append(request)
        content = self.text if self.text is not None else json.dumps(self.payload)
        return httpx.Response(self.status_code, headers={"content-type": "application/json"}, content=content.encode())


@pytest.fixture
def recording():
    transport = RecordingTransport()

    def factory(*, timeout):
        return httpx.AsyncClient(transport=transport, base_url="http://rust.test")

    main.app.state.client_factory = factory
    return transport


@pytest.mark.anyio
async def test_catalogue_has_public_mirrors_and_full_spectrum_metadata():
    response = await main.list_tools()
    assert response["runtime_engine_count"] == 19
    assert response["public_engine_count"] == 17
    names = {item["name"] for item in response["tools"]}
    assert "engine_calculate:raaga" in names
    full = next(item for item in response["tools"] if item["name"] == "full_spectrum")
    assert full["status"] == "unsupported"
    assert all(item["server"] == "rust" for item in response["tools"])


@pytest.mark.anyio
async def test_engine_calculate_uses_exact_api_key_and_protected_route(recording, monkeypatch):
    monkeypatch.setattr(main, "API_KEY", "key-1")
    monkeypatch.setattr(main, "BEARER_TOKEN", "")
    result = await main.execute_tool(main.ToolRequest(name="engine_calculate", arguments={"engine_id": "raaga", "parameters": {}}))
    assert result.success
    assert recording.calls[0].url.path == "/api/v1/engines/raaga/calculate"
    assert recording.calls[0].headers["x-api-key"] == "key-1"
    assert "authorization" not in recording.calls[0].headers


@pytest.mark.anyio
async def test_workflow_execute_rejects_full_spectrum_and_invalid_names_before_transport(recording):
    for workflow_id in ["full-spectrum", "unknown", "../workflow", "workflow.name", "a" * 81]:
        with pytest.raises(main.HTTPException):
            await main.execute_tool(main.ToolRequest(name="workflow_execute", arguments={"workflow_id": workflow_id}))
    assert recording.calls == []


@pytest.mark.anyio
async def test_invalid_engine_never_constructs_url_or_calls_upstream(recording):
    for engine_id in ["financial-biosensor", "biofield-capture", "raaga/evil", "raaga.evil", "../raaga", "a" * 81]:
        with pytest.raises(main.HTTPException):
            await main.execute_tool(main.ToolRequest(name="engine_calculate", arguments={"engine_id": engine_id}))
    assert recording.calls == []


@pytest.mark.anyio
async def test_bearer_is_distinct_and_upstream_errors_are_bounded(recording, monkeypatch):
    monkeypatch.setattr(main, "API_KEY", "")
    monkeypatch.setattr(main, "BEARER_TOKEN", "bearer-1")
    recording.status_code = 502
    recording.text = "token=secret raw provider body"
    result = await main.execute_tool(main.ToolRequest(name="engine_calculate", arguments={"engine_id": "raaga"}))
    assert not result.success
    assert result.error == "UPSTREAM_ERROR"
    assert recording.calls[0].headers["authorization"] == "Bearer bearer-1"
    assert "x-api-key" not in recording.calls[0].headers


@pytest.mark.anyio
async def test_ambiguous_auth_fails_before_transport(recording, monkeypatch):
    monkeypatch.setattr(main, "API_KEY", "key")
    monkeypatch.setattr(main, "BEARER_TOKEN", "bearer")
    with pytest.raises(main.HTTPException):
        await main.execute_tool(main.ToolRequest(name="engine_calculate", arguments={"engine_id": "raaga"}))
    assert recording.calls == []
